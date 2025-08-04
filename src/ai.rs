use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

use crate::config;

/// Valid conventional commit types
const VALID_TYPES: [&str; 8] = [
    "feat", "fix", "chore", "docs", "style", "refactor", "test", "perf",
];

/// Emoji mapping for commit types
const EMOJI_MAP: [(&str, &str); 8] = [
    ("feat", "🚀"),
    ("fix", "🐛"),
    ("chore", "🔧"),
    ("docs", "📝"),
    ("style", "🎨"),
    ("refactor", "♻️"),
    ("test", "🧪"),
    ("perf", "⚡"),
];

/// Cleans and validates a commit message to follow conventional commit format
pub fn clean_commit_message(message: &str) -> String {
    clean_commit_message_with_emoji(message, false)
}

/// Cleans and validates a commit message with optional emoji support
pub fn clean_commit_message_with_emoji(message: &str, use_emoji: bool) -> String {
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
        return if use_emoji { "🔧 chore: update code".to_string() } else { "chore: update code".to_string() };
    }

    // Check if it follows type(scope): description format
    if !cleaned.contains(':') {
        return if use_emoji { "🔧 chore: update code".to_string() } else { "chore: update code".to_string() };
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

    // Add emoji if requested and not already present
    if use_emoji && !has_emoji(&cleaned) {
        cleaned = add_emoji_to_commit(&cleaned);
    }

    cleaned
}

/// Check if a commit message already has an emoji
fn has_emoji(message: &str) -> bool {
    // Check if the message starts with an emoji (after any whitespace)
    let trimmed = message.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    
    // Simple emoji detection - look for Unicode emoji characters
    // This covers most common emoji ranges
    let first_char = trimmed.chars().next().unwrap();
    let emoji_ranges = [
        (0x1F600..=0x1F64F), // Emoticons
        (0x1F300..=0x1F5FF), // Miscellaneous Symbols and Pictographs
        (0x1F680..=0x1F6FF), // Transport and Map Symbols
        (0x1F1E0..=0x1F1FF), // Regional Indicator Symbols
        (0x2600..=0x26FF),   // Miscellaneous Symbols
        (0x2700..=0x27BF),   // Dingbats
    ];
    
    let char_code = first_char as u32;
    emoji_ranges.iter().any(|range| range.contains(&char_code))
}

/// Add appropriate emoji to a commit message based on its type
fn add_emoji_to_commit(message: &str) -> String {
    // Extract the commit type
    if let Some(colon_pos) = message.find(':') {
        let type_part = &message[..colon_pos];
        
        // Find the base type (before any scope)
        let base_type = if let Some(paren_pos) = type_part.find('(') {
            &type_part[..paren_pos]
        } else {
            type_part
        };
        
        // Find matching emoji
        for (commit_type, emoji) in EMOJI_MAP {
            if base_type == commit_type {
                return format!("{} {}", emoji, message);
            }
        }
    }
    
    // Default emoji if no match found
    format!("🔧 {}", message)
}

/// Generates a commit message using the configured AI model
pub fn generate_commit_message(prompt: &str, configuration: &config::Config, use_emoji: bool) -> Result<String> {
    let client = Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": configuration.model,
            "prompt": prompt,
            "stream": false
        }))
        .timeout(Duration::from_secs(45))
        .send()?;

    let json: serde_json::Value = response.json()?;
    let raw_msg = json.get("response").and_then(|r| r.as_str()).unwrap_or("");

    Ok(clean_commit_message_with_emoji(raw_msg, use_emoji))
}
