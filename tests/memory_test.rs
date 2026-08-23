use pi_agent_rust::memory::{AttemptOutcome, MemoryEngine, ProjectMap};
use tempfile::tempdir;

#[tokio::test]
async fn test_memory_event_lifecycle() {
    let tmp = tempdir().unwrap();
    let memory = MemoryEngine::new(tmp.path()).unwrap();

    // 1. Log Issue
    let issue_id = memory.log_issue("Buffer overflow in socket parser", Some("src/socket.rs")).unwrap();
    assert_eq!(issue_id, "0001");

    // 2. Record Failed Attempt
    let fail_id = memory
        .record_attempt(
            "Tried increasing static buffer to 1024 bytes — still panics on large packet",
            AttemptOutcome::Failed,
            Some(&issue_id),
            Some("src/socket.rs"),
        )
        .unwrap();
    assert!(fail_id.starts_with("evt_"));

    // 3. Record Worked Attempt
    let work_id = memory
        .record_attempt(
            "Switched to dynamic vector allocation with 64KB chunking",
            AttemptOutcome::Worked,
            Some(&issue_id),
            Some("src/socket.rs"),
        )
        .unwrap();
    assert!(work_id.starts_with("evt_"));

    // 4. Precheck File
    let precheck = memory.precheck_file("src/socket.rs").unwrap();
    assert!(precheck.has_warnings);
    assert_eq!(precheck.open_issues.len(), 1);
    assert_eq!(precheck.failed_attempts.len(), 1);
    assert!(precheck.guidance.contains("DO NOT REPEAT"));

    // 5. Record Fix & Close Issue
    let fix_id = memory
        .record_fix(
            "Fixed buffer overflow with dynamic chunked vector and unit test suite",
            Some(&issue_id),
            Some("src/socket.rs"),
        )
        .unwrap();
    assert!(fix_id.starts_with("evt_"));

    // 6. Precheck File After Fix (no more open issues)
    let precheck_after = memory.precheck_file("src/socket.rs").unwrap();
    assert_eq!(precheck_after.open_issues.len(), 0);

    // 7. Verify Score
    let score = memory.get_score().unwrap();
    assert_eq!(score.total_issues, 1);
    assert_eq!(score.resolved_issues, 1);
    assert!(score.dollars_saved > 0.0);
    assert!(score.score_percentage >= 90.0);

    // 8. Verify Search
    let results = memory.search("socket", 5).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_project_map_parse_and_generate() {
    let raw = r#"# Project Map

## Project purpose
Lightweight high-performance network proxy.

## Stack
- Rust 2021
- Tokio

## Structure
- `src/` — Core source
  - `src/main.rs` — CLI Entry
  - `src/proxy.rs` — SOCKS5 server

## Relationships
- `src/main.rs` starts `src/proxy.rs`
"#;

    let map = ProjectMap::parse(raw);
    assert_eq!(map.purpose, "Lightweight high-performance network proxy.");
    assert_eq!(map.stack.len(), 2);
    assert_eq!(map.structure.len(), 3);
    assert_eq!(map.relationships.len(), 1);

    let md = map.to_markdown();
    assert!(md.contains("## Project purpose"));
    assert!(md.contains("Lightweight high-performance network proxy."));
}
