use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub goal: String,
    pub steps: Vec<TaskStep>,
    pub current_step_index: usize,
    pub is_finished: bool,
}

impl ExecutionPlan {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            steps: Vec::new(),
            current_step_index: 0,
            is_finished: false,
        }
    }

    pub fn add_step(&mut self, title: impl Into<String>, description: impl Into<String>) {
        let step_number = self.steps.len() + 1;
        self.steps.push(TaskStep {
            step_number,
            title: title.into(),
            description: description.into(),
            completed: false,
            error: None,
        });
    }

    pub fn mark_current_completed(&mut self) {
        if self.current_step_index < self.steps.len() {
            self.steps[self.current_step_index].completed = true;
            self.current_step_index += 1;
            if self.current_step_index >= self.steps.len() {
                self.is_finished = true;
            }
        }
    }

    pub fn mark_current_failed(&mut self, err: impl Into<String>) {
        if self.current_step_index < self.steps.len() {
            self.steps[self.current_step_index].error = Some(err.into());
        }
    }

    pub fn format_prompt_block(&self) -> String {
        if self.steps.is_empty() {
            return String::new();
        }

        let mut out = format!("\n<active_plan goal=\"{}\">\n", self.goal);
        for (i, step) in self.steps.iter().enumerate() {
            let mark = if step.completed {
                "[x]"
            } else if i == self.current_step_index {
                "[->]"
            } else {
                "[ ]"
            };
            out.push_str(&format!("  {} Step {}: {}\n", mark, step.step_number, step.title));
            if let Some(err) = &step.error {
                out.push_str(&format!("      ⚠️ Error: {}\n", err));
            }
        }
        out.push_str("</active_plan>\n");
        out
    }
}
