pub struct PromptTemplates;

impl PromptTemplates {
    pub fn expand_command(input: &str) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.starts_with("/commit") {
            let extra = trimmed.trim_start_matches("/commit").trim();
            Some(format!(
                "Review the current staged/unstaged git diff using the `git` tool, summarize the architectural and code changes concisely, and write a high-quality conventional git commit message. {}",
                extra
            ))
        } else if trimmed.starts_with("/test") {
            let target = trimmed.trim_start_matches("/test").trim();
            Some(format!(
                "Run the test suite using `bash` ('cargo test' or project test command). If any tests fail, log the issue into memory with `log_issue`, precheck the file, diagnose the root cause, and apply a verified fix. {}",
                target
            ))
        } else if trimmed.starts_with("/refactor") {
            let target = trimmed.trim_start_matches("/refactor").trim();
            Some(format!(
                "Precheck target files with `precheck_file`, analyze the architecture for clean design patterns, remove duplication, improve type safety, and verify with tests. Target: {}",
                target
            ))
        } else if trimmed.starts_with("/explain") {
            let target = trimmed.trim_start_matches("/explain").trim();
            Some(format!(
                "Read the code in {} and give a clear, high-level structural explanation of how it works and how it connects with the rest of the project map.",
                target
            ))
        } else {
            None
        }
    }
}
