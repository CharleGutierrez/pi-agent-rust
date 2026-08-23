use pi_agent_rust::agent::{ContextCompactor, ExecutionPlan, ReflectionEngine};
use pi_agent_rust::providers::types::Message;

#[test]
fn test_planner_tree_of_thought() {
    let mut plan = ExecutionPlan::new("Implement persistent memory compaction");
    plan.add_step("Parse JSONL events", "Stream and deserialize memory events");
    plan.add_step("Prune old context", "Keep recent turns and distill middle");

    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.current_step_index, 0);

    plan.mark_current_completed();
    assert_eq!(plan.current_step_index, 1);

    plan.mark_current_completed();
    assert!(plan.is_finished);

    let block = plan.format_prompt_block();
    assert!(block.contains("<active_plan"));
    assert!(block.contains("[x] Step 1"));
    assert!(block.contains("[x] Step 2"));
}

#[test]
fn test_reflection_diagnostics() {
    let diag = ReflectionEngine::diagnose("edit", "Edit #1: oldText not found in src/main.rs");
    assert_eq!(diag.category, pi_agent_rust::agent::ErrorCategory::EditConflict);
    assert!(diag.suggested_remedy.contains("read"));

    let diag2 = ReflectionEngine::diagnose("bash", "error[E0405]: cannot find value `foo` in this scope");
    assert_eq!(diag2.category, pi_agent_rust::agent::ErrorCategory::CompilationError);
    assert!(diag2.should_record_failed_attempt);
}

#[test]
fn test_context_compaction() {
    let mut messages = vec![
        Message::system("System instructions"),
        Message::user("Turn 1 question"),
        Message::assistant("Turn 1 answer"),
        Message::user("Turn 2 question"),
        Message::assistant("Turn 2 answer"),
        Message::user("Turn 3 question"),
        Message::assistant("Turn 3 answer"),
        Message::user("Latest question"),
    ];

    let compacted = ContextCompactor::compact_history(&mut messages, 50);
    assert!(compacted);
    assert!(messages[0].content.contains("System instructions"));
    assert!(messages[1].content.contains("[Context Compaction"));
}
