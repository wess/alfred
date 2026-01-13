use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::fmt::Display;

pub fn info<T: Display>(msg: T) {
  println!("{} {}", "i".blue(), msg);
}

pub fn success<T: Display>(msg: T) {
  println!("{} {}", "✓".green(), msg);
}

pub fn warn<T: Display>(msg: T) {
  println!("{} {}", "!".yellow(), msg);
}

pub fn error<T: Display>(msg: T) {
  println!("{} {}", "✗".red(), msg);
}

pub fn heading<T: Display>(msg: T) {
  println!("\n{}", msg.to_string().bold());
}

pub fn dim<T: Display>(msg: T) {
  println!("{}", msg.to_string().dimmed());
}

pub fn separator() {
  println!("{}", "───────────────────────────".dimmed());
}

pub fn prompt(question: &str) -> Option<String> {
  Input::<String>::new()
    .with_prompt(format!("{} {}", "?".cyan(), question))
    .allow_empty(true)
    .interact_text()
    .ok()
    .filter(|s| !s.is_empty())
}

pub fn confirm(question: &str, default: bool) -> bool {
  Confirm::new()
    .with_prompt(format!("{} {}", "?".cyan(), question))
    .default(default)
    .interact()
    .unwrap_or(default)
}

pub fn select<T: ToString + Clone>(question: &str, options: &[T]) -> Option<T> {
  if options.is_empty() {
    return None;
  }

  let items: Vec<String> = options.iter().map(|o| o.to_string()).collect();

  Select::new()
    .with_prompt(format!("{} {}", "?".cyan(), question))
    .items(&items)
    .default(0)
    .interact()
    .ok()
    .and_then(|i| options.get(i).cloned())
}

pub fn list_item<T: Display>(item: T) {
  println!("  {} {}", "•".dimmed(), item);
}

pub fn list_item_colored<T: Display>(marker: &str, color: &str, item: T) {
  let colored_marker = match color {
    "red" => marker.red().to_string(),
    "green" => marker.green().to_string(),
    "yellow" => marker.yellow().to_string(),
    "blue" => marker.blue().to_string(),
    "cyan" => marker.cyan().to_string(),
    _ => marker.dimmed().to_string(),
  };
  println!("  {} {}", colored_marker, item);
}
