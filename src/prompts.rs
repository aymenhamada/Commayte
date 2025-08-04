pub fn generate_commit_prompt(diff: &str, project_context: &str) -> String {
    format!(
        "Analyze the git diff below and generate a conventional commit message.\n\n\
        Here some context about the project:\n\
        {project_context}\n\n\
        Instructions:\n\
        1. Look at each file name, added lines (+), and removed lines (-)\n\
        2. Determine the type based on the changes:\n\
           - feat: new features or functionality\n\
           - fix: bug fixes or error corrections\n\
           - chore: maintenance, dependencies, config changes\n\
           - docs: documentation updates\n\
           - style: formatting, whitespace, code style\n\
           - refactor: code restructuring without changing behavior\n\
           - test: adding or updating tests\n\
           - perf: performance improvements\n\
        3. Determine scope from the file path (e.g., client, server, config, ui)\n\
        4. Write a description based on what was actually changed\n\
        5. Use format: type(scope): description\n\
        6. Keep description short concise and under 30 characters\n\
        7. Return ONLY the commit message\n\n\
        RETURN ONLY THE COMMIT MESSAGE, NOTHING ELSE.\n\n\
        RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
        RETURN ONLY THE COMMIT MESSAGE, NOTHING ELSE.\n\n\
        Git diff:\n{diff}\n\n\
        Commit message:"
    )
}
