pub fn generate_commit_prompt(diff: &str, project_context: &str, use_emoji: bool) -> String {
    if use_emoji {
        format!(
            "Analyze the git diff below and generate a conventional commit message.\n\n\
            Project context:\n{project_context}\n\n\
            Instructions:\n\
            1. Look at each file name, added lines (+), and removed lines (-)\n\
            2. Determine the type based on changes:\n\
               - feat: new features\n\
               - fix: bug fixes\n\
               - chore: maintenance/config\n\
               - docs: documentation\n\
               - style: formatting\n\
               - refactor: code restructuring\n\
               - test: adding/updating tests\n\
               - perf: performance improvements\n\
            3. Format: type description\n\
            4. Keep description concise\n\
            5. Return ONLY the commit message\n\n\
            RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
            RETURN ONLY THE COMMIT MESSAGE.\n\n\
            RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
            Git diff:\n{diff}\n\n\
            Commit message:"
        )
    } else {
        format!(
            "Analyze the git diff below and generate a conventional commit message.\n\n\
            Project context:\n{project_context}\n\n\
            Instructions:\n\
            1. Look at each file name, added lines (+), and removed lines (-)\n\
            2. Determine the type based on changes:\n\
               - feat: new features\n\
               - fix: bug fixes\n\
               - chore: maintenance/config\n\
               - docs: documentation\n\
               - style: formatting\n\
               - refactor: code restructuring\n\
               - test: adding/updating tests\n\
               - perf: performance improvements\n\
            3. Determine scope from file path (e.g., client, server, ui)\n\
            4. Write description of what changed\n\
            5. Format: type(scope): description\n\
            6. Keep description concise\n\
            7. Return ONLY the commit message\n\n\
            RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
            RETURN ONLY THE COMMIT MESSAGE.\n\n\
            RESPECT CONVENTIONAL COMMIT SPECIFICATION.\n\n\
            Git diff:\n{diff}\n\n\
            Commit message:"
        )
    }
}
