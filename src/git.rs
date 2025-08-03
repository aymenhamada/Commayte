use anyhow::Result;
use std::process::Command;

pub fn get_git_diff() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("Failed to get git diff");

    let diff_output = String::from_utf8_lossy(&output.stdout);

    // Files to ignore (dependency managers, lock files, etc.)
    let ignored_patterns = [
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

    let mut filtered_diff = Vec::new();
    let mut current_file = String::new();
    let mut include_current_file = true;
    let mut total_content_length = 0;
    const MAX_TOTAL_CONTENT: usize = 8000;
    const MAX_FILE_CONTENT: usize = 1000; // Maximum characters per file

    for line in diff_output.lines() {
        // Check if this is a file header (starts with "diff --git")
        if line.starts_with("diff --git") {
            // Process the previous file if it should be included
            if include_current_file && !current_file.is_empty() {
                let mut file_content = current_file.clone();

                // Truncate individual file if it's too large
                if file_content.len() > MAX_FILE_CONTENT {
                    file_content = file_content
                        .chars()
                        .take(MAX_FILE_CONTENT)
                        .collect::<String>();
                    file_content.push_str("\n... (file truncated)");
                }

                let file_size = file_content.len();
                if total_content_length + file_size > MAX_TOTAL_CONTENT {
                    filtered_diff.push("... (diff truncated due to size limit)".to_string());
                    break;
                }
                filtered_diff.push(file_content);
                total_content_length += file_size;
            }

            // Reset for new file
            current_file = line.to_string();
            include_current_file = true;

            // Extract filename from diff header
            if let Some(filename) = extract_filename_from_diff_header(line) {
                // Check if file should be ignored
                let should_ignore = ignored_patterns.iter().any(|pattern| {
                    if let Some(suffix) = pattern.strip_prefix('*') {
                        // Handle wildcard patterns
                        filename.ends_with(suffix)
                    } else if let Some(dir_pattern) = pattern.strip_suffix('/') {
                        // Handle directory patterns
                        filename.starts_with(dir_pattern)
                    } else {
                        // Handle exact patterns
                        filename.contains(pattern)
                    }
                });

                if should_ignore {
                    include_current_file = false;
                }
            }
        } else {
            // Add line to current file if it should be included
            if include_current_file {
                current_file.push('\n');
                current_file.push_str(line);
            }
        }
    }

    // Don't forget the last file
    if include_current_file && !current_file.is_empty() {
        let mut file_content = current_file.clone();

        // Truncate individual file if it's too large
        if file_content.len() > MAX_FILE_CONTENT {
            file_content = file_content
                .chars()
                .take(MAX_FILE_CONTENT)
                .collect::<String>();
            file_content.push_str("\n... (file truncated)");
        }

        let file_size = file_content.len();
        if total_content_length + file_size <= MAX_TOTAL_CONTENT {
            filtered_diff.push(file_content);
        }
    }

    filtered_diff.join("\n\n")
}

pub fn extract_filename_from_diff_header(header: &str) -> Option<&str> {
    // Extract filename from "diff --git a/filename b/filename" format
    if let Some(start) = header.find("a/") {
        if let Some(end) = header[start + 2..].find(" b/") {
            return Some(&header[start + 2..start + 2 + end]);
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
