use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

use crate::config;

/// Valid conventional commit types
const VALID_TYPES: [&str; 8] = [
    "feat", "fix", "chore", "docs", "style", "refactor", "test", "perf",
];

/// Cleans and validates a commit message to follow conventional commit format
pub fn clean_commit_message(message: &str) -> String {
    let first_line = message.lines().next().unwrap_or("").trim();
    let mut cleaned = first_line.to_string();

    // Remove common prefixes and quotes
    for prefix in [
        "commit",
        "Commit:",
        "Commit message:",
        "\"",
        "'",
        "```",
        "`",
    ] {
        cleaned = cleaned.replace(prefix, "");
    }
    cleaned = cleaned.trim().to_string();

    // Validate conventional commit format
    if cleaned.is_empty() {
        return "chore: update code".to_string();
    }

    // Check if it follows type(scope): description format
    if !cleaned.contains(':') {
        return "chore: update code".to_string();
    }

    // Ensure it starts with a valid type
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() < 2 {
        return "".to_string();
    }

    let type_part = parts[0];
    let has_valid_type = VALID_TYPES.iter().any(|&t| type_part.starts_with(t));

    if !has_valid_type {
        return "".to_string();
    }

    cleaned
}

/// Generates a commit message using the configured AI model
pub fn generate_commit_message(prompt: &str, project_context: &str) -> Result<String> {
    let configuration = config::load_config();
    let client = Client::new();

    let system_prompt = format!("You are a concise AI assistant that only returns single-line Git commit messages and that follows the conventional commit format https://www.conventionalcommits.org/en/v1.0.0/. Never include explanations. Project context: \n\n{project_context}");

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": configuration.model,
            "prompt": prompt,
            "system": system_prompt,
            "stream": false
        }))
        .timeout(Duration::from_secs(45))
        .send()?;

    let json: serde_json::Value = response.json()?;
    let raw_msg = json.get("response").and_then(|r| r.as_str()).unwrap_or("");

    Ok(clean_commit_message(raw_msg))
}
