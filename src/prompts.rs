/// Generates the main prompt for commit message generation
pub fn generate_commit_prompt(diff: &str) -> String {
    format!(
        "Generate conventional commit from diff:\n\
        Types: feat|fix|chore|docs|style|refactor|test|perf\n\
        Format: type(scope): description\n\
        Focus on + and - lines only. Keep description <50 chars.\n\
        ---\n\
        {diff}\n\
        ---\n\
        Commit:"
    )
}
