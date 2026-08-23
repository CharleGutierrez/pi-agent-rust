use pi_agent_rust::memory::MemoryEngine;
use pi_agent_rust::providers::types::ToolCall;
use pi_agent_rust::tools::ToolRegistry;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_tools_suite() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    let mem_engine = Arc::new(MemoryEngine::new(&root).unwrap());
    let registry = ToolRegistry::init_standard(root.clone(), mem_engine.clone());

    // 1. Test Write Tool
    let write_call = ToolCall {
        id: "call_1".to_string(),
        name: "write".to_string(),
        arguments: json!({
            "path": "src/calc.rs",
            "content": "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"
        }),
    };
    let res = registry.execute_call(&write_call).await;
    assert!(!res.is_error);
    assert!(root.join("src/calc.rs").exists());

    // 2. Test Read Tool
    let read_call = ToolCall {
        id: "call_2".to_string(),
        name: "read".to_string(),
        arguments: json!({
            "path": "src/calc.rs",
            "offset": 1,
            "limit": 10
        }),
    };
    let res = registry.execute_call(&read_call).await;
    assert!(!res.is_error);
    assert!(res.content.contains("pub fn add"));

    // 3. Test Edit Tool (Precision replacement)
    let edit_call = ToolCall {
        id: "call_3".to_string(),
        name: "edit".to_string(),
        arguments: json!({
            "path": "src/calc.rs",
            "edits": [
                {
                    "oldText": "    a + b\n",
                    "newText": "    // Fast addition\n    a.saturating_add(b)\n"
                }
            ]
        }),
    };
    let res = registry.execute_call(&edit_call).await;
    assert!(!res.is_error);
    let edited_content = fs::read_to_string(root.join("src/calc.rs")).unwrap();
    assert!(edited_content.contains("saturating_add"));

    // 4. Test Grep Tool
    let grep_call = ToolCall {
        id: "call_4".to_string(),
        name: "grep".to_string(),
        arguments: json!({
            "pattern": "saturating_add",
            "path": "."
        }),
    };
    let res = registry.execute_call(&grep_call).await;
    assert!(!res.is_error);
    assert!(res.content.contains("src/calc.rs"));

    // 5. Test Find Files Tool
    let find_call = ToolCall {
        id: "call_5".to_string(),
        name: "find_files".to_string(),
        arguments: json!({
            "pattern": "*.rs"
        }),
    };
    let res = registry.execute_call(&find_call).await;
    assert!(!res.is_error);
    assert!(res.content.contains("src/calc.rs"));

    // 6. Test Memory MCP Tool (add_decision)
    let dec_call = ToolCall {
        id: "call_6".to_string(),
        name: "add_decision".to_string(),
        arguments: json!({
            "summary": "Use saturating arithmetic for calculator functions",
            "location": "src/calc.rs"
        }),
    };
    let res = registry.execute_call(&dec_call).await;
    assert!(!res.is_error);
    assert!(res.content.contains("Recorded decision"));
}
