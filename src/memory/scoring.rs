use super::events::{AttemptOutcome, EventType, MemoryEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePreventionScore {
    pub grade: String,
    pub score_percentage: f32,
    pub total_events: usize,
    pub total_issues: usize,
    pub resolved_issues: usize,
    pub failed_attempts_prevented: usize,
    pub hours_saved: f32,
    pub dollars_saved: f32,
    pub tokens_prevented: u64,
}

impl FailurePreventionScore {
    pub fn compute(events: &[MemoryEvent]) -> Self {
        let total_events = events.len();
        let mut total_issues = 0;
        let mut resolved_issues = 0;
        let mut failed_attempts = 0;
        let mut worked_attempts = 0;
        let mut tokens_prevented: u64 = 0;

        for evt in events {
            match evt.event_type {
                EventType::Issue => total_issues += 1,
                EventType::Fix => resolved_issues += 1,
                EventType::Attempt => match evt.outcome {
                    Some(AttemptOutcome::Failed) => failed_attempts += 1,
                    Some(AttemptOutcome::Worked) => worked_attempts += 1,
                    _ => {}
                },
                _ => {}
            }

            if let Some(tok) = evt.tokens_prevented {
                tokens_prevented += tok;
            }
        }

        // Each logged failed attempt prevents at least 2 repeat cycles in future AI runs
        let prevented_cycles = failed_attempts * 2;
        // Default token saving heuristic if not manually specified: 3,500 tokens per avoided dead-end
        if tokens_prevented == 0 && prevented_cycles > 0 {
            tokens_prevented = (prevented_cycles as u64) * 3500;
        }

        // Standard developer time saved (20 minutes per avoided rabbit hole + 45 min per tracked fix)
        let hours_saved = (prevented_cycles as f32 * 0.35) + (resolved_issues as f32 * 0.75);
        // Average senior engineer rate $120/hr
        let dollars_saved = hours_saved * 120.0;

        // Calculate score & grade
        let resolution_rate = if total_issues > 0 {
            resolved_issues as f32 / total_issues as f32
        } else {
            1.0
        };

        let attempts_logged_ratio = if total_issues > 0 {
            ((failed_attempts + worked_attempts) as f32 / total_issues as f32).min(1.0)
        } else {
            1.0
        };

        let raw_score = (resolution_rate * 60.0) + (attempts_logged_ratio * 40.0);
        let score_percentage = raw_score.clamp(0.0, 100.0);

        let grade = match score_percentage as u32 {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "A-",
            80..=84 => "B+",
            75..=79 => "B",
            70..=74 => "B-",
            60..=69 => "C",
            50..=59 => "D",
            _ => "F",
        }
        .to_string();

        Self {
            grade,
            score_percentage,
            total_events,
            total_issues,
            resolved_issues,
            failed_attempts_prevented: prevented_cycles,
            hours_saved,
            dollars_saved,
            tokens_prevented,
        }
    }

    pub fn formatted_report(&self) -> String {
        format!(
            "🏆 Failure-Prevention Score: {} ({:.1}%)\n\
             ├─ Events tracked: {}\n\
             ├─ Issues resolved: {} / {}\n\
             ├─ Dead-ends prevented: {}\n\
             ├─ Dev hours saved: {:.1} hrs\n\
             ├─ Value protected: ${:.2}\n\
             └─ Tokens saved: {} tokens",
            self.grade,
            self.score_percentage,
            self.total_events,
            self.resolved_issues,
            self.total_issues,
            self.failed_attempts_prevented,
            self.hours_saved,
            self.dollars_saved,
            self.tokens_prevented
        )
    }
}
