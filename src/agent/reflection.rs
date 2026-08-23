use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    CompilationError,
    TestFailure,
    RuntimeCrash,
    FileNotFound,
    EditConflict,
    CommandTimeout,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionDiagnosis {
    pub category: ErrorCategory,
    pub root_cause: String,
    pub suggested_remedy: String,
    pub should_record_failed_attempt: bool,
}

pub struct ReflectionEngine;

impl ReflectionEngine {
    /// Analyze tool error output and generate diagnostic hints for the agent
    pub fn diagnose(tool_name: &str, raw_error: &str) -> ReflectionDiagnosis {
        let err_lower = raw_error.to_lowercase();

        if tool_name == "edit" && err_lower.contains("oldtext not found") {
            ReflectionDiagnosis {
                category: ErrorCategory::EditConflict,
                root_cause: "The exact oldText did not match file content. The file may have changed or line endings differ.".to_string(),
                suggested_remedy: "Call `read` on the file around the target lines first to inspect the exact current text, then re-issue `edit`.".to_string(),
                should_record_failed_attempt: false,
            }
        } else if tool_name == "bash" && (err_lower.contains("error[e") || err_lower.contains("cannot find") || err_lower.contains("mismatched types")) {
            ReflectionDiagnosis {
                category: ErrorCategory::CompilationError,
                root_cause: "Compiler error encountered in build/check step.".to_string(),
                suggested_remedy: "Inspect compiler error message and location, precheck the file, and apply targeted code correction.".to_string(),
                should_record_failed_attempt: true,
            }
        } else if tool_name == "bash" && (err_lower.contains("test failed") || err_lower.contains("assertion failed") || err_lower.contains("panicked at")) {
            ReflectionDiagnosis {
                category: ErrorCategory::TestFailure,
                root_cause: "Test assertion or panic occurred.".to_string(),
                suggested_remedy: "Read the test case and the asserted condition. Formulate a fix hypothesis before modifying code.".to_string(),
                should_record_failed_attempt: true,
            }
        } else if tool_name == "bash" && err_lower.contains("timed out") {
            ReflectionDiagnosis {
                category: ErrorCategory::CommandTimeout,
                root_cause: "Command took longer than timeout threshold.".to_string(),
                suggested_remedy: "Check for infinite loops or long-running interactive processes. Pass --no-interactive or timeout flags.".to_string(),
                should_record_failed_attempt: false,
            }
        } else if err_lower.contains("not found") || err_lower.contains("no such file") {
            ReflectionDiagnosis {
                category: ErrorCategory::FileNotFound,
                root_cause: "Target file does not exist at requested path.".to_string(),
                suggested_remedy: "Use `find_files` or `get_project_map` to verify correct relative path.".to_string(),
                should_record_failed_attempt: false,
            }
        } else {
            ReflectionDiagnosis {
                category: ErrorCategory::Unknown,
                root_cause: "Tool returned non-zero or error status.".to_string(),
                suggested_remedy: "Inspect tool error text and try an alternative approach.".to_string(),
                should_record_failed_attempt: false,
            }
        }
    }
}
