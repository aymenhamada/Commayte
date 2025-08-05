use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;
use lazy_static::lazy_static;

use crate::config;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Valid conventional commit types
const VALID_TYPES: [&str; 8] = [
    "feat", "fix", "chore", "docs", "style", "refactor", "test", "perf",
];

lazy_static! {
    static ref GITMOJI_MAP: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        
        // New features
        map.insert("feat", vec![
            "🚀", "🎉", "💫", "🌟"
        ]);
        
        // Bug fixes
        map.insert("fix", vec![
            "🐛", "🚑️", "🩹", "🔧", "🔨"
        ]);
        
        // Maintenance and config
        map.insert("chore", vec![
            "🔧", "🔨", "⚙️", "🔧", "📦️"
        ]);
        
        // Documentation
        map.insert("docs", vec![
            "📝", "📚", "📖", "📄", "📋"
        ]);
        
        // Code style and formatting
        map.insert("style", vec![
            "🎨", "💄", "🎭", "✨", "💅"
        ]);
        
        // Code refactoring
        map.insert("refactor", vec![
            "♻️", "🔄", "🛠️", "🔨", "⚡"
        ]);
        
        // Tests
        map.insert("test", vec![
            "🧪", "✅", "🔬", "🧪", "🎯"
        ]);
        
        // Performance improvements
        map.insert("perf", vec![
            "⚡", "🚀", "💨", "🔥", "⚡️"
        ]);
        
        // Additional types for better coverage
        map.insert("security", vec![
            "🔒️", "🔐", "🛡️", "🔒", "🔐"
        ]);
        
        map.insert("ci", vec![
            "👷", "🚧", "🔧", "⚙️", "🔨"
        ]);
        
        map.insert("build", vec![
            "📦️", "🔨", "⚙️", "🔧", "🏗️"
        ]);
        
        map.insert("deps", vec![
            "⬆️", "⬇️", "📌", "➕", "➖"
        ]);
        
        map.insert("revert", vec![
            "⏪️", "↩️", "🔄", "⏮️", "↪️"
        ]);
        
        map.insert("breaking", vec![
            "💥", "🚨", "⚠️", "💣", "🔥"
        ]);
        
        map
    };
}


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

/// Cleans and validates AI-generated commit messages with optional gitmoji support
pub fn clean_commit_message_from_ai(message: &str, use_gitmoji: bool) -> String {

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
        return if use_gitmoji { "🔧 chore: update code".to_string() } else { "chore: update code".to_string() };
    }

    // Check if it follows type(scope): description format
    if !cleaned.contains(':') {
        return if use_gitmoji { "🔧 chore: update code".to_string() } else { "chore: update code".to_string() };
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

    // Add gitmoji if requested and not already present
    if use_gitmoji && !has_emoji(&cleaned) {
        cleaned = add_gitmoji_to_commit(&cleaned);
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

/// Add appropriate gitmoji to a commit message based on its type
fn add_gitmoji_to_commit(message: &str) -> String {
    // Extract the commit type
    if let Some(colon_pos) = message.find(':') {
        let type_part = &message[..colon_pos];
        
        // Find the base type (before any scope)
        let base_type = if let Some(paren_pos) = type_part.find('(') {
            &type_part[..paren_pos]
        } else {
            type_part
        };
        
        // Find matching gitmoji options
        if let Some(emoji_options) = GITMOJI_MAP.get(base_type) {
            // Use a simple hash of the message to select emoji consistently
            let mut hasher = DefaultHasher::new();
            message.hash(&mut hasher);
            let hash = hasher.finish();
            let emoji_index = (hash % emoji_options.len() as u64) as usize;
            let selected_emoji = emoji_options[emoji_index];
            
            return format!("{} {}", selected_emoji, message);
        }
    }
    
    // Default emoji if no match found
    format!("🔧 {}", message)
}

/// Generates a commit message using the configured AI model
pub fn generate_commit_message(
    prompt: &str,
    configuration: &config::Config,
    use_gitmoji: bool,
) -> Result<String> {
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

    Ok(clean_commit_message_from_ai(raw_msg, use_gitmoji))
}
