use anyhow::{anyhow, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::sampling::LlamaSampler;
use once_cell::sync::OnceCell;
use std::num::NonZeroU32;

use crate::config;
use crate::daemon_client;

static BACKEND: OnceCell<LlamaBackend> = OnceCell::new();
static MODEL: OnceCell<LlamaModel> = OnceCell::new();

fn get_backend() -> &'static LlamaBackend {
    BACKEND.get_or_init(|| {
        LlamaBackend::init().expect("Failed to initialize llama backend")
    })
}

pub fn load_model() -> Result<()> {
    if MODEL.get().is_some() {
        return Ok(());
    }

    let model_path = config::get_model_path();
    if !model_path.exists() {
        return Err(anyhow!(
            "Model not found at {}\nRun 'alfred setup' to download it.",
            model_path.display()
        ));
    }

    let backend = get_backend();
    let model_params = LlamaModelParams::default();

    let model = LlamaModel::load_from_file(backend, &model_path, &model_params)
        .with_context(|| format!("Failed to load model from {}", model_path.display()))?;

    MODEL
        .set(model)
        .map_err(|_| anyhow!("Model already loaded"))?;

    Ok(())
}

pub fn is_loaded() -> bool {
    MODEL.get().is_some()
}

/// Generate text locally (used by daemon)
pub fn generate_local(prompt: &str, max_tokens: u32) -> Result<String> {
    if !is_loaded() {
        load_model()?;
    }

    let model = MODEL.get().ok_or_else(|| anyhow!("Model not loaded"))?;
    let backend = get_backend();

    // Create context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(2048));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .with_context(|| "Failed to create context")?;

    // Tokenize prompt
    let tokens = model
        .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
        .with_context(|| "Failed to tokenize prompt")?;

    let n_tokens = tokens.len();

    // Create batch and add prompt tokens
    let mut batch = LlamaBatch::new(n_tokens.max(1), 1);

    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == n_tokens - 1;
        batch.add(*token, i as i32, &[0], is_last)?;
    }

    // Decode prompt
    ctx.decode(&mut batch)
        .with_context(|| "Failed to decode prompt")?;

    // Setup sampler - chain temperature, top-k, top-p, and distribution samplers
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(0.7),
        LlamaSampler::top_k(40),
        LlamaSampler::top_p(0.9, 1),
        LlamaSampler::dist(42),
    ]);

    // Generate tokens
    let mut output = String::new();
    let mut n_generated = 0;
    let mut n_cur = n_tokens;
    let eos_token = model.token_eos();

    while n_generated < max_tokens {
        // Sample next token
        let token = sampler.sample(&ctx, -1);
        sampler.accept(token);

        // Check for EOS
        if token == eos_token {
            break;
        }

        // Convert token to string
        if let Ok(piece) = model.token_to_str(token, llama_cpp_2::model::Special::Tokenize) {
            output.push_str(&piece);
        }

        // Prepare next batch
        batch.clear();
        batch.add(token, n_cur as i32, &[0], true)?;
        n_cur += 1;

        // Decode
        if ctx.decode(&mut batch).is_err() {
            break;
        }

        n_generated += 1;
    }

    Ok(output.trim().to_string())
}

/// Generate text - tries daemon first, falls back to local
pub fn generate(prompt: &str, max_tokens: u32) -> Result<String> {
    // Try daemon first
    if let Ok(mut client) = daemon_client::connect() {
        return client.generate(prompt, max_tokens);
    }

    // Fallback to local generation
    generate_local(prompt, max_tokens)
}

pub fn generate_commit_message(diff: &str) -> Result<String> {
    // Try daemon first
    if let Ok(mut client) = daemon_client::connect() {
        return client.generate_commit_message(diff);
    }

    // Fallback to local
    let prompt = format!(
        r#"<|system|>
You are a helpful assistant that generates concise, conventional git commit messages.
Follow the conventional commits format: type(scope): description
Types: feat, fix, docs, style, refactor, test, chore
Keep the first line under 72 characters.
Only output the commit message, nothing else.<|end|>
<|user|>
Generate a commit message for this diff:

{}<|end|>
<|assistant|>"#,
        &diff[..diff.len().min(4000)]
    );

    let response = generate_local(&prompt, 100)?;
    Ok(response.lines().next().unwrap_or(&response).trim().to_string())
}

pub fn suggest_conflict_resolution(
    file: &str,
    ours: &str,
    theirs: &str,
    base: &str,
) -> Result<String> {
    // Try daemon first
    if let Ok(mut client) = daemon_client::connect() {
        return client.suggest_conflict_resolution(file, ours, theirs, base);
    }

    // Fallback to local
    let prompt = format!(
        r#"<|system|>
You are a helpful assistant that resolves git merge conflicts.
Analyze the conflict and provide a merged result that preserves the intent of both changes.
Only output the resolved code, no explanations.<|end|>
<|user|>
Resolve this merge conflict in {}:

BASE (original):
{}

OURS (current branch):
{}

THEIRS (incoming branch):
{}

Provide the merged result:<|end|>
<|assistant|>"#,
        file,
        &base[..base.len().min(2000)],
        &ours[..ours.len().min(2000)],
        &theirs[..theirs.len().min(2000)]
    );

    generate_local(&prompt, 500)
}

pub fn suggest_rebase_strategy(commits: &[String], onto: &str) -> Result<String> {
    // Try daemon first
    if let Ok(mut client) = daemon_client::connect() {
        return client.suggest_rebase_strategy(commits, onto);
    }

    // Fallback to local
    let prompt = format!(
        r#"<|system|>
You are a helpful assistant that suggests git rebase strategies.
Analyze the commits and suggest which ones to squash, reorder, or reword.
Be concise and provide actionable suggestions.<|end|>
<|user|>
I'm rebasing these commits onto {}:

{}

Suggest a rebase strategy (squash, reorder, reword):<|end|>
<|assistant|>"#,
        onto,
        commits.join("\n")
    );

    generate_local(&prompt, 200)
}

pub fn suggest_branch_name(description: &str) -> Result<String> {
    // Try daemon first
    if let Ok(mut client) = daemon_client::connect() {
        return client.suggest_branch_name(description);
    }

    // Fallback to local
    let prompt = format!(
        r#"<|system|>
You are a helpful assistant that suggests git branch names.
Follow conventions: feature/, bugfix/, hotfix/, chore/
Use kebab-case, keep it short but descriptive.
Only output the branch name, nothing else.<|end|>
<|user|>
Suggest a branch name for: {}<|end|>
<|assistant|>"#,
        description
    );

    let response = generate_local(&prompt, 30)?;
    let name = response
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    Ok(name)
}

pub fn unload() {
    // OnceCell doesn't support removal, but the values will be dropped when the program exits
    // In a more complex application, we might use a Mutex<Option<T>> instead
}
