use std::collections::HashMap;
use std::fs;

fn get_project_info() -> HashMap<String, String> {
    let mut project_info = HashMap::new();

    // Try to read Cargo.toml (Rust project)
    if let Ok(content) = fs::read_to_string("Cargo.toml") {
        if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
            if let Some(package) = toml_value.get("package") {
                if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
                    project_info.insert("project_name".to_string(), name.to_string());
                }
                if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                    project_info.insert("project_version".to_string(), version.to_string());
                }
                if let Some(description) = package.get("description").and_then(|d| d.as_str()) {
                    project_info.insert("project_description".to_string(), description.to_string());
                }
                if let Some(keywords) = package.get("keywords").and_then(|k| k.as_array()) {
                    let keywords_str: Vec<String> = keywords
                        .iter()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect();
                    if !keywords_str.is_empty() {
                        project_info
                            .insert("project_keywords".to_string(), keywords_str.join(", "));
                    }
                }
            }
            project_info.insert("project_type".to_string(), "rust".to_string());
        }
    }
    // Try to read package.json (Node.js project)
    else if let Ok(content) = fs::read_to_string("package.json") {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json_value.get("name").and_then(|n| n.as_str()) {
                project_info.insert("project_name".to_string(), name.to_string());
            }
            if let Some(version) = json_value.get("version").and_then(|v| v.as_str()) {
                project_info.insert("project_version".to_string(), version.to_string());
            }
            if let Some(description) = json_value.get("description").and_then(|d| d.as_str()) {
                project_info.insert("project_description".to_string(), description.to_string());
            }
            if let Some(keywords) = json_value.get("keywords").and_then(|k| k.as_array()) {
                let keywords_str: Vec<String> = keywords
                    .iter()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect();
                if !keywords_str.is_empty() {
                    project_info.insert("project_keywords".to_string(), keywords_str.join(", "));
                }
            }
            project_info.insert("project_type".to_string(), "nodejs".to_string());
        }
    }
    // Try to read build.gradle (Java/Gradle project)
    else if let Ok(content) = fs::read_to_string("build.gradle") {
        // Simple parsing for common Gradle patterns
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("group =") || line.starts_with("group=") {
                if let Some(group) = line.split('=').nth(1) {
                    let group = group.trim().trim_matches('"').trim_matches('\'');
                    project_info.insert("project_group".to_string(), group.to_string());
                }
            } else if line.starts_with("version =") || line.starts_with("version=") {
                if let Some(version) = line.split('=').nth(1) {
                    let version = version.trim().trim_matches('"').trim_matches('\'');
                    project_info.insert("project_version".to_string(), version.to_string());
                }
            } else if line.starts_with("description =") || line.starts_with("description=") {
                if let Some(desc) = line.split('=').nth(1) {
                    let desc = desc.trim().trim_matches('"').trim_matches('\'');
                    project_info.insert("project_description".to_string(), desc.to_string());
                }
            }
        }
        if !project_info.is_empty() {
            project_info.insert("project_type".to_string(), "java".to_string());
        }
    }
    // Try to read pom.xml (Java/Maven project)
    else if let Ok(content) = fs::read_to_string("pom.xml") {
        // Simple XML parsing for Maven POM
        if let Some(name_start) = content.find("<name>") {
            if let Some(name_end) = content[name_start..].find("</name>") {
                let name = &content[name_start + 6..name_start + name_end];
                project_info.insert("project_name".to_string(), name.trim().to_string());
            }
        }
        if let Some(version_start) = content.find("<version>") {
            if let Some(version_end) = content[version_start..].find("</version>") {
                let version = &content[version_start + 9..version_start + version_end];
                project_info.insert("project_version".to_string(), version.trim().to_string());
            }
        }
        if let Some(desc_start) = content.find("<description>") {
            if let Some(desc_end) = content[desc_start..].find("</description>") {
                let desc = &content[desc_start + 13..desc_start + desc_end];
                project_info.insert("project_description".to_string(), desc.trim().to_string());
            }
        }
        if !project_info.is_empty() {
            project_info.insert("project_type".to_string(), "java".to_string());
        }
    }
    // Try to read pyproject.toml (Python project)
    else if let Ok(content) = fs::read_to_string("pyproject.toml") {
        if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
            if let Some(project) = toml_value.get("project") {
                if let Some(name) = project.get("name").and_then(|n| n.as_str()) {
                    project_info.insert("project_name".to_string(), name.to_string());
                }
                if let Some(version) = project.get("version").and_then(|v| v.as_str()) {
                    project_info.insert("project_version".to_string(), version.to_string());
                }
                if let Some(description) = project.get("description").and_then(|d| d.as_str()) {
                    project_info.insert("project_description".to_string(), description.to_string());
                }
            }
            project_info.insert("project_type".to_string(), "python".to_string());
        }
    }
    // Try to read requirements.txt (Python project)
    else if fs::metadata("requirements.txt").is_ok() {
        project_info.insert("project_type".to_string(), "python".to_string());
    }
    // Try to read go.mod (Go project)
    else if let Ok(content) = fs::read_to_string("go.mod") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("module ") {
                if let Some(module) = line.split_whitespace().nth(1) {
                    project_info.insert("project_name".to_string(), module.to_string());
                }
            } else if line.starts_with("go ") {
                if let Some(version) = line.split_whitespace().nth(1) {
                    project_info.insert("go_version".to_string(), version.to_string());
                }
            }
        }
        if !project_info.is_empty() {
            project_info.insert("project_type".to_string(), "go".to_string());
        }
    }
    // Try to read composer.json (PHP project)
    else if let Ok(content) = fs::read_to_string("composer.json") {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json_value.get("name").and_then(|n| n.as_str()) {
                project_info.insert("project_name".to_string(), name.to_string());
            }
            if let Some(description) = json_value.get("description").and_then(|d| d.as_str()) {
                project_info.insert("project_description".to_string(), description.to_string());
            }
            project_info.insert("project_type".to_string(), "php".to_string());
        }
    }
    // Try to read Gemfile (Ruby project)
    else if fs::metadata("Gemfile").is_ok() {
        project_info.insert("project_type".to_string(), "ruby".to_string());
    }

    // Try to read Dockerfile
    if fs::metadata("Dockerfile").is_ok() {
        project_info.insert("has_docker".to_string(), "true".to_string());
    }

    // Try to read docker-compose.yml
    if fs::metadata("docker-compose.yml").is_ok() || fs::metadata("docker-compose.yaml").is_ok() {
        project_info.insert("has_docker_compose".to_string(), "true".to_string());
    }

    // Try to read .github/workflows/ (GitHub Actions)
    if fs::metadata(".github/workflows").is_ok() {
        project_info.insert("has_github_actions".to_string(), "true".to_string());
    }

    // Try to read README.md
    if let Ok(content) = fs::read_to_string("README.md") {
        // Extract first line as potential project name if not already found
        if !project_info.contains_key("project_name") {
            if let Some(first_line) = content.lines().next() {
                let title = first_line.trim_start_matches('#').trim();
                if !title.is_empty() && title.len() < 100 {
                    project_info.insert("project_name".to_string(), title.to_string());
                }
            }
        }
    }

    project_info
}

fn build_context(project_info: HashMap<String, String>) -> String {
    let mut project_context = String::new();
    if !project_info.is_empty() {
        if let Some(name) = project_info.get("project_name") {
            project_context.push_str(&format!("- Name: {name}\n"));
        }
        if let Some(version) = project_info.get("project_version") {
            project_context.push_str(&format!("- Version: {version}\n"));
        }
        if let Some(description) = project_info.get("project_description") {
            project_context.push_str(&format!("- Description: {description}\n"));
        }
        if let Some(project_type) = project_info.get("project_type") {
            project_context.push_str(&format!("- Type: {project_type}\n"));
        }
        if let Some(keywords) = project_info.get("project_keywords") {
            project_context.push_str(&format!("- Keywords: {keywords}\n"));
        }
        if let Some(group) = project_info.get("project_group") {
            project_context.push_str(&format!("- Group: {group}\n"));
        }
        if let Some(go_version) = project_info.get("go_version") {
            project_context.push_str(&format!("- Go Version: {go_version}\n"));
        }
        if project_info.contains_key("has_docker") {
            project_context.push_str("- Has Docker: true\n");
        }
        if project_info.contains_key("has_docker_compose") {
            project_context.push_str("- Has Docker Compose: true\n");
        }
        if project_info.contains_key("has_github_actions") {
            project_context.push_str("- Has GitHub Actions: true\n");
        }
        project_context.push('\n');
    }

    project_context
}

pub fn get_project_context() -> String {
    let project_info = get_project_info();
    build_context(project_info)
}
