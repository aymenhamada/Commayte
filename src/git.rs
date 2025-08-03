use anyhow::Result;
use std::process::Command;

const IGNORED_PATTERNS: [&str; 28] = [
    ".lock",
    ".lockfile",
    "package-lock.json",
    "yarn.lock",
    "Cargo.lock",
    "Gemfile.lock",
    "composer.lock",
    "poetry.lock",
    "Pipfile.lock",
    "requirements.txt",
    "package.json",
    "node_modules/",
    "vendor/",
    "target/",
    "dist/",
    "build/",
    ".git/",
    ".DS_Store",
    "*.log",
    "*.tmp",
    "*.cache",
    ".env",
    ".env.local",
    ".env.example",
    "models/",
    "*.gguf",
    "*.bin",
    "*.safetensors",
];

pub fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("Failed to get git diff");

    let diff_output = String::from_utf8_lossy(&output.stdout);

    let mut filtered_diff = Vec::new();
    let mut current_file_content = Vec::new();
    let mut current_file_header = String::new();
    let mut include_current_file = true;
    let mut total_content_length = 0;
    const MAX_TOTAL_CONTENT: usize = 8000;
    const MAX_FILE_CONTENT: usize = 1000;

    for line in diff_output.lines() {
        // Check if this is a file header (starts with "diff --git")
        if line.starts_with("diff --git") {
            // Process the previous file if it should be included
            if include_current_file && !current_file_header.is_empty() {
                let file_content = format_file_diff(&current_file_header, &current_file_content);
                
                if file_content.len() <= MAX_FILE_CONTENT {
                    let file_size = file_content.len();
                    if total_content_length + file_size > MAX_TOTAL_CONTENT {
                        filtered_diff.push("... (diff truncated due to size limit)".to_string());
                        break;
                    }
                    filtered_diff.push(file_content);
                    total_content_length += file_size;
                } else {
                    // Truncate the changed lines if too many
                    let truncated_content = format_file_diff_truncated(&current_file_header, &current_file_content, MAX_FILE_CONTENT);
                    let file_size = truncated_content.len();
                    if total_content_length + file_size > MAX_TOTAL_CONTENT {
                        filtered_diff.push("... (diff truncated due to size limit)".to_string());
                        break;
                    }
                    filtered_diff.push(truncated_content);
                    total_content_length += file_size;
                }
            }

            // Reset for new file
            current_file_header.clear();
            current_file_content.clear();
            include_current_file = true;

            // Extract filename from diff header
            if let Some(filename) = extract_filename_from_diff_header(line) {
                // Check if file should be ignored
                let should_ignore = IGNORED_PATTERNS.iter().any(|pattern| {
                    if let Some(suffix) = pattern.strip_prefix('*') {
                        filename.ends_with(suffix)
                    } else if let Some(dir_pattern) = pattern.strip_suffix('/') {
                        filename.starts_with(dir_pattern)
                    } else {
                        filename.contains(pattern)
                    }
                });

                if should_ignore {
                    include_current_file = false;
                }
            }

            if include_current_file {
                current_file_header = line.to_string();
            }
        } else if include_current_file {
            // Only include header lines and actual changes (+ and - lines)
            if is_relevant_diff_line(line) {
                if line.starts_with("index ") || line.starts_with("---") || line.starts_with("+++") || line.starts_with("@@") {
                    current_file_header.push('\n');
                    current_file_header.push_str(line);
                } else if line.starts_with('+') || line.starts_with('-') {
                    current_file_content.push(line.to_string());
                }
            }
        }
    }

    // Don't forget the last file
    if include_current_file && !current_file_header.is_empty() {
        let file_content = format_file_diff(&current_file_header, &current_file_content);
        
        if file_content.len() <= MAX_FILE_CONTENT {
            let file_size = file_content.len();
            if total_content_length + file_size <= MAX_TOTAL_CONTENT {
                filtered_diff.push(file_content);
            }
        } else {
            let truncated_content = format_file_diff_truncated(&current_file_header, &current_file_content, MAX_FILE_CONTENT);
            let file_size = truncated_content.len();
            if total_content_length + file_size <= MAX_TOTAL_CONTENT {
                filtered_diff.push(truncated_content);
            }
        }
    }

    filtered_diff.join("\n\n")
}

fn is_relevant_diff_line(line: &str) -> bool {
    // Include header lines and actual changes
    line.starts_with("index ") ||
    line.starts_with("---") ||
    line.starts_with("+++") ||
    line.starts_with("@@") ||
    line.starts_with('+') ||
    line.starts_with('-')
}

fn format_file_diff(header: &str, changes: &[String]) -> String {
    if changes.is_empty() {
        return header.to_string();
    }
    
    format!("{}\n{}", header, changes.join("\n"))
}

fn format_file_diff_truncated(header: &str, changes: &[String], max_size: usize) -> String {
    let mut result = header.to_string();
    let mut current_size = header.len();
    
    for change in changes {
        let new_size = current_size + change.len() + 1; // +1 for newline
        if new_size > max_size - 30 { // Reserve space for truncation message
            result.push_str("\n... (changes truncated)");
            break;
        }
        result.push('\n');
        result.push_str(change);
        current_size = new_size;
    }
    
    result
}

fn extract_filename_from_diff_header(line: &str) -> Option<String> {
    // Extract filename from "diff --git a/path/file.ext b/path/file.ext"
    if let Some(parts) = line.strip_prefix("diff --git ") {
        if let Some(space_pos) = parts.find(' ') {
            let a_path = &parts[..space_pos];
            if let Some(filename) = a_path.strip_prefix("a/") {
                return Some(filename.to_string());
            }
        }
    }
    None
}

/// Executes a git commit command
pub fn execute_git_commit(message: &str) -> Result<std::process::ExitStatus> {
    let result = Command::new("git")
        .args(["commit", "-am", message])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    Ok(result)
}

pub fn get_git_branch() -> String {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .expect("Failed to get git branch");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
