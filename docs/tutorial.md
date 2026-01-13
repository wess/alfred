# Alfred Tutorial

This tutorial walks you through a complete development workflow using Alfred. You'll learn how to use AI-powered commit messages, smart branch creation, merge conflict resolution, and more.

## Prerequisites

Before starting, make sure you have:
- Alfred installed and set up (`alfred setup` completed)
- A git repository to work with

If you need a test repository:
```bash
mkdir alfred-tutorial
cd alfred-tutorial
git init
echo "# Tutorial Project" > README.md
git add README.md
git commit -m "Initial commit"
```

## Part 1: AI-Powered Commits

### Scenario

You've made some changes to your project and want to commit them with a well-written message.

### Step 1: Make Some Changes

Let's create a simple file:

```bash
cat > src/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

mkdir -p src
```

### Step 2: Stage Your Changes

```bash
git add src/main.rs
```

### Step 3: Generate Commit Message

```bash
alfred commit
```

Alfred analyzes your staged changes and generates a message:

```
Analyzing staged changes...

Generated commit message:

  feat: add main entry point with hello world

  - Create src/main.rs with basic program structure
  - Print greeting message to stdout

? Use this message? [Y/n]
```

Press Enter to accept, or type `n` to decline and write your own.

### Step 4: Edit Before Committing (Optional)

If you want to modify the message:

```bash
alfred commit --edit
```

This generates the message, then opens your `$EDITOR` so you can tweak it before committing.

### Understanding the Generated Message

Alfred follows the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<optional scope>): <description>

<optional body>
```

**Types Alfred uses:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `style` - Formatting, no code change
- `refactor` - Code restructuring
- `test` - Adding tests
- `chore` - Maintenance tasks

---

## Part 2: Smart Branch Creation

### Scenario

You're about to start working on a new feature and need a branch with a good name.

### Step 1: Describe Your Work

```bash
alfred branch new
```

Alfred prompts you:

```
? What are you working on?
```

Type a description:

```
? What are you working on? Adding user authentication with JWT tokens
```

### Step 2: Review Suggested Name

Alfred suggests a branch name:

```
Suggested branch name: feature/add-user-authentication-jwt

? Use this name? [Y/n]
```

Press Enter to create the branch, or `n` to enter a custom name.

### Step 3: Alternative - Direct Creation

If you already know the name you want:

```bash
alfred branch new feature/my-feature
```

This creates the branch directly without AI suggestions.

### Branch Naming Conventions

Alfred follows these conventions:
- `feature/` - New features
- `bugfix/` - Bug fixes
- `hotfix/` - Urgent production fixes
- `chore/` - Maintenance tasks

---

## Part 3: Managing Branches

### List Branches

See all your local branches:

```bash
alfred branch list
```

Output:
```
Branches:
  • main
  • feature/add-user-authentication-jwt (current)
  • bugfix/fix-login-error
```

Include remote branches:

```bash
alfred branch list --all
```

### Clean Up Merged Branches

After merging feature branches, clean up:

```bash
alfred branch clean
```

Alfred shows merged branches and asks for confirmation:

```
Merged branches to delete:
  • feature/old-feature
  • bugfix/fixed-issue

? Delete these branches? [Y/n]
```

To skip confirmation:

```bash
alfred branch clean --force
```

---

## Part 4: Resolving Merge Conflicts

### Scenario

You're merging a branch and hit a conflict. Alfred can help resolve it.

### Step 1: Create a Conflict (for demonstration)

```bash
# On main branch
git checkout main
echo "version = 1" > config.txt
git add config.txt
git commit -m "Add config"

# Create feature branch
git checkout -b feature/update-config
echo "version = 2" > config.txt
git add config.txt
git commit -m "Update version to 2"

# Back to main, make conflicting change
git checkout main
echo "version = 1.5" > config.txt
git add config.txt
git commit -m "Update version to 1.5"

# Try to merge
git merge feature/update-config
# CONFLICT!
```

### Step 2: Use Alfred to Resolve

```bash
alfred resolve config.txt
```

Alfred analyzes the conflict:

```
Analyzing conflict in config.txt...

OURS (main):
  version = 1.5

THEIRS (feature/update-config):
  version = 2

Suggested resolution:
  version = 2

Reasoning: The feature branch has a newer version (2) which supersedes
both the base (1) and the main branch (1.5).

? Apply this resolution? [Y/n]
```

### Step 3: Accept or Modify

Press Enter to apply the suggestion. The conflict markers are replaced with the resolved content.

### Step 4: Complete the Merge

```bash
git add config.txt
git commit -m "Merge feature/update-config"
```

### Resolving All Conflicts at Once

If you have multiple conflicting files:

```bash
alfred resolve
```

Alfred processes each file and asks for confirmation on each resolution.

---

## Part 5: Rebase with AI Suggestions

### Scenario

You have several commits on a feature branch and want to clean them up before merging.

### Step 1: View Your Commits

```bash
git log --oneline main..HEAD
```

```
a1b2c3d WIP: trying something
d4e5f6g Fix typo
g7h8i9j Add user authentication
j0k1l2m WIP: authentication started
```

### Step 2: Get AI Suggestions

```bash
alfred rebase main --ai
```

Alfred analyzes your commits:

```
Commits to rebase onto main:
  1. j0k1l2m WIP: authentication started
  2. g7h8i9j Add user authentication
  3. d4e5f6g Fix typo
  4. a1b2c3d WIP: trying something

AI Suggestions:
  • Squash commits 1 and 2 (both related to authentication)
  • Squash commit 3 into commit 2 (typo fix belongs with main change)
  • Consider dropping commit 4 or squashing if it contains useful changes

Recommended final history:
  1. feat(auth): add user authentication
     (combines commits 1, 2, 3)

? Proceed with interactive rebase? [Y/n]
```

### Step 3: Interactive Rebase

If you proceed, Alfred starts an interactive rebase with the suggested actions pre-filled.

---

## Part 6: Using the Daemon

### Why Use the Daemon?

Without the daemon, each Alfred command loads the AI model from disk (~2-3 seconds). The daemon keeps it in memory for instant responses.

### Start the Daemon

```bash
alfred daemon start
```

```
i Starting alfred daemon...
✓ Daemon started (PID: 12345)
Model will be loaded on first request
```

### Check Status

```bash
alfred daemon status
```

```
Daemon Status

  Status: Running
  PID: 12345
  Port: 7654
  Idle timeout: 30 minutes
  Service: Not installed
```

### Test the Speed Difference

```bash
# First command loads the model
time alfred branch new --help

# Subsequent commands are instant
time alfred branch new --help
```

### Install as Service

For automatic startup at login:

```bash
alfred daemon install
```

### Stop When Done

```bash
alfred daemon stop
```

---

## Part 7: Git Passthrough

Alfred acts as a transparent wrapper around git. Any command it doesn't recognize gets passed to git:

```bash
# These work exactly like regular git commands
alfred status
alfred log --oneline -5
alfred diff
alfred push origin main
alfred pull
alfred fetch --all
alfred stash
alfred cherry-pick abc123
```

This means you can use `alfred` as your primary git command, and get AI features when you need them.

---

## Part 8: Complete Workflow Example

Here's a realistic workflow using all of Alfred's features:

```bash
# 1. Start the daemon for fast responses
alfred daemon start

# 2. Create a feature branch
alfred branch new
# ? What are you working on? Implement password reset feature
# Suggested: feature/implement-password-reset

# 3. Do your work...
# (write code, make changes)

# 4. Commit with AI-generated messages
git add src/auth/reset.rs
alfred commit
# Generated: feat(auth): add password reset functionality

git add src/email/templates/reset.html
alfred commit
# Generated: feat(email): add password reset email template

# 5. More work, more commits...
git add .
alfred commit

# 6. Clean up commits before PR
alfred rebase main --ai
# Suggests squashing related commits

# 7. Push and create PR
alfred push -u origin feature/implement-password-reset

# 8. After PR is merged, clean up
alfred checkout main
alfred pull
alfred branch clean

# 9. Stop daemon when done for the day
alfred daemon stop
```

---

## Tips and Best Practices

### Commit Messages

- Stage related changes together for better commit messages
- Use `--edit` if you want to add more context
- Alfred works best with focused, single-purpose commits

### Branch Names

- Be descriptive when asked "What are you working on?"
- Include ticket/issue numbers in your description if applicable

### Merge Conflicts

- Review AI suggestions carefully before accepting
- For complex conflicts, use `--edit` or resolve manually

### Daemon

- Start the daemon at the beginning of your work session
- Install as a service if you use Alfred daily
- Reduce idle timeout on memory-constrained systems

### General

- Use `alfred --help` or `alfred <command> --help` for quick reference
- Remember that unrecognized commands pass through to git
- Check `alfred daemon status` if commands seem slow

---

## Next Steps

- Read the [Command Reference](./commands.md) for all options
- Configure Alfred with the [Configuration Guide](./configuration.md)
- Set up the [Daemon](./daemon.md) for best performance
