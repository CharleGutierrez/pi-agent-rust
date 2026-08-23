use super::auto_tuner::{AutoTuner, TunedProfile};
use super::compaction::ContextCompactor;
use super::planner::ExecutionPlan;
use super::reflection::ReflectionEngine;
use super::session::SessionManager;
use crate::config::AppConfig;
use crate::memory::MemoryEngine;
use crate::providers::types::{Message, StreamChunk, ToolCall};
use crate::providers::ProviderRouter;
use crate::tools::ToolRegistry;
use anyhow::Result;
use futures_util::StreamExt;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AgentEngine {
    pub router: Arc<ProviderRouter>,
    pub tools: Arc<ToolRegistry>,
    pub memory: Arc<MemoryEngine>,
    pub session: SessionManager,
    pub active_plan: Option<ExecutionPlan>,
    pub max_turns: usize,
    pub system_profile: TunedProfile,
}

impl AgentEngine {
    pub fn new(
        work_dir: PathBuf,
        config: &AppConfig,
        default_model: Option<String>,
        max_turns: Option<usize>,
    ) -> Result<Self> {
        let memory = Arc::new(MemoryEngine::new(&work_dir)?);
        let tools = Arc::new(ToolRegistry::init_standard(work_dir.clone(), memory.clone()));
        
        let mut local_config = config.clone();
        if let Some(dm) = default_model {
            local_config.default_model = dm;
        }
        
        let router = Arc::new(ProviderRouter::new(&local_config));
        
        // Auto-tune the system based on hardware & selected model
        let (_, model_id) = router.resolve_alias(&local_config.default_model);
        let active_model = router.list_all_models().into_iter().find(|m| m.id == model_id)
            .unwrap_or_else(|| crate::providers::types::ModelInfo {
                id: model_id.clone(),
                name: model_id,
                provider: "unknown".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            });
            
        let system_profile = AutoTuner::auto_tune_system(&active_model);
        
        let session = SessionManager::new(&work_dir)?;

        Ok(Self {
            router,
            tools,
            memory,
            session,
            active_plan: None,
            max_turns: max_turns.unwrap_or(25),
            system_profile,
        })
    }

    /// Build the comprehensive system prompt including memory context and rules
    pub fn build_system_prompt(&self) -> String {
        let memory_context = self
            .memory
            .get_context(2000, None)
            .unwrap_or_else(|_| "No memory context.".to_string());

        let plan_block = if let Some(plan) = &self.active_plan {
            plan.format_prompt_block()
        } else {
            String::new()
        };

        format!(
            "You are Pi Coding Agent (Rust Edition) — a superpowerful, lightweight, precision coding assistant.\n\n\
             ## MANDATORY WORKFLOW & MEMORY RULES\n\
             1. Persistent Memory is your single source of truth. Always navigate by `PROJECT_MAP.md` and `summary.md`.\n\
             2. BEFORE modifying ANY file, call `precheck_file(file_path)` to check failure history and avoid known dead-ends.\n\
             3. On bug or unexpected behavior discovery -> IMMEDIATELY call `log_issue(summary, location)`.\n\
             4. After EVERY fix attempt -> call `record_attempt(summary, outcome, issue_id, location)` ('worked', 'failed', or 'partial').\n\
             5. When a fix is confirmed with evidence -> call `record_fix(summary, issue_id, location)`.\n\
             6. On architectural or design choice -> call `add_decision(summary, location)`.\n\
             7. On gotcha / environment detail -> call `add_note(summary, location)`.\n\
             8. Execute shell commands with `bash`, read files with `read`, edit files with `edit`, search with `grep`.\n\n\
             {}\n\
             {}\n\
             Always be concise, practical, and tool-aware. Output clean code and clear explanations.",
            memory_context, plan_block
        )
    }

    /// Run single query turn with full autonomous ReAct agent loop
    pub async fn run_turn<F>(&mut self, user_input: &str, model: &str, mut on_stream: F) -> Result<String>
    where
        F: FnMut(StreamChunk),
    {
        // 1. Initialize system message if first turn
        if self.session.messages().is_empty() {
            let sys_prompt = self.build_system_prompt();
            self.session.add_message(Message::system(sys_prompt));
        }

        // 2. Add User Message
        self.session.add_message(Message::user(user_input));

        let tool_definitions = self.tools.definitions();
        let mut turn_count = 0;
        let mut final_response_text = String::new();

        while turn_count < self.max_turns {
            turn_count += 1;

            // Context compaction dynamically tuned to CPU and Model Capacity
            ContextCompactor::compact_history(self.session.messages_mut(), self.system_profile.optimal_context_window);

            // Execute streaming request to LLM provider
            let mut stream = self
                .router
                .stream(model, self.session.messages(), &tool_definitions, Some(0.2), None)
                .await?;

            let mut assistant_content = String::new();
            let mut thinking_content = String::new();
            let mut pending_tool_calls: Vec<ToolCall> = Vec::new();
            let mut tool_args_buffer: std::collections::HashMap<usize, (Option<String>, Option<String>, String)> =
                std::collections::HashMap::new();

            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        match &chunk {
                            StreamChunk::ThinkingDelta(t) => {
                                thinking_content.push_str(t);
                            }
                            StreamChunk::ContentDelta(c) => {
                                assistant_content.push_str(c);
                            }
                            StreamChunk::ToolCallDelta {
                                index,
                                id,
                                name,
                                arguments_delta,
                            } => {
                                let entry = tool_args_buffer.entry(*index).or_insert((None, None, String::new()));
                                if let Some(i) = id {
                                    entry.0 = Some(i.clone());
                                }
                                if let Some(n) = name {
                                    entry.1 = Some(n.clone());
                                }
                                entry.2.push_str(arguments_delta);
                            }
                            _ => {}
                        }
                        on_stream(chunk);
                    }
                    Err(e) => {
                        tracing::error!("Stream chunk error: {}", e);
                    }
                }
            }

            // Assemble accumulated tool calls
            for (idx, (id_opt, name_opt, args_str)) in tool_args_buffer {
                let id = id_opt.unwrap_or_else(|| format!("call_{}", idx));
                let name = name_opt.unwrap_or_else(|| "unknown".to_string());
                let arguments: serde_json::Value =
                    serde_json::from_str(&args_str).unwrap_or_else(|_| serde_json::json!({}));

                pending_tool_calls.push(ToolCall { id, name, arguments });
            }

            // Save assistant message to session
            let mut asst_msg = Message::assistant(&assistant_content);
            if !thinking_content.is_empty() {
                asst_msg.thinking = Some(thinking_content);
            }
            if !pending_tool_calls.is_empty() {
                asst_msg.tool_calls = pending_tool_calls.clone();
            }
            self.session.add_message(asst_msg);

            // If no tool calls were requested, the turn is finished
            if pending_tool_calls.is_empty() {
                final_response_text = assistant_content;
                break;
            }

            // Execute tool calls sequentially
            for tool_call in pending_tool_calls {
                let result = self.tools.execute_call(&tool_call).await;

                // Self-correction reflection if tool returned error
                let response_content = if result.is_error {
                    let diagnosis = ReflectionEngine::diagnose(&tool_call.name, &result.content);
                    format!(
                        "{}\n\n[Agent Self-Correction Hint: Root cause: {}. Suggested action: {}]",
                        result.content, diagnosis.root_cause, diagnosis.suggested_remedy
                    )
                } else {
                    result.content
                };

                self.session
                    .add_message(Message::tool_result(&tool_call.id, response_content));
            }
        }

        // Save session checkpoint
        let _ = self.session.save_current_session(model);

        Ok(final_response_text)
    }
}
