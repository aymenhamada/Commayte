/// Generates the main prompt for commit message generation
pub fn generate_commit_prompt(diff: &str, branch: &str) -> String {
    format!(
        "Analyze the git diff below and generate a conventional commit message. The branch is {branch}.\n\n\
        Instructions:\n\n\
        1. Look at each file name, added lines (+), and removed lines (-)\n\
        2. Determine the type based on the changes:\n\
            Types: feat, fix, chore, docs, style, refactor, test, perf\n\
                - feat: new features or functionality\n\
                - fix: bug fixes or error corrections\n\
                - chore: maintenance, dependencies, config changes\n\
                - docs: documentation updates\n\
                - style: formatting, whitespace, code style (Do not use style unless the change is purely formatting)\n\
                - refactor: code restructuring without changing behavior\n\
                - test: adding or updating tests\n\
        3. Determine scope from the file path\n\
        4. Write a short, concise description under 30 characters based on what was actually changed\n\
        Use Format: type(scope): description\n\
        Git diff:\n{diff}\n\n\
        Commit message:"
    )
}
