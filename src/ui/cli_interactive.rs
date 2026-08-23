use crate::agent::AgentEngine;
use crate::providers::types::StreamChunk;
use crate::skills::PromptTemplates;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, BufRead, Write};
use std::time::Duration;

pub struct CliInteractive;

impl CliInteractive {
    pub async fn run(mut agent: AgentEngine, initial_model: &str) -> Result<()> {
        let mut current_model = initial_model.to_string();
        let (provider, _) = agent.router.resolve_alias(&current_model);
        let project_name = agent.memory.storage.root_dir().file_name().unwrap_or_default().to_string_lossy().to_string();

        super::banner::Banner::print_startup(&current_model, &provider, agent.session.session_id(), &project_name);

        let stdin = io::stdin();
        let mut handle = stdin.lock();

        loop {
            print!("{} ", "❯".bright_green().bold());
            io::stdout().flush()?;

            let mut line = String::new();
            if handle.read_line(&mut line)? == 0 {
                break;
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            // Handle commands
            if input == "/exit" || input == "/quit" || input == "exit" {
                println!("{}", "Goodbye! Memory saved.".bright_green());
                break;
            }

            if input == "/score" {
                let score = agent.memory.get_score()?;
                println!("\n{}\n", score.formatted_report().bright_cyan());
                continue;
            }

            if input == "/memory" || input == "/summary" {
                let summary = agent.memory.get_summary()?;
                println!("\n{}\n", summary.bright_white());
                continue;
            }

            if input == "/map" {
                let map = agent.memory.storage.memory_dir().join(crate::memory::storage::MAP_FILE);
                if map.exists() {
                    let content = std::fs::read_to_string(map)?;
                    println!("\n{}\n", content.bright_blue());
                } else {
                    println!("{}", "No map file yet.".bright_yellow());
                }
                continue;
            }

            if input == "/plan" {
                let plan = agent.memory.storage.memory_dir().join(crate::memory::storage::PLAN_FILE);
                if plan.exists() {
                    let content = std::fs::read_to_string(plan)?;
                    println!("\n{}\n", content.bright_yellow());
                } else {
                    println!("{}", "No plan file yet.".bright_yellow());
                }
                continue;
            }

            if input.starts_with("/model") {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() > 1 {
                    current_model = parts[1].to_string();
                    let (p, m) = agent.router.resolve_alias(&current_model);
                    println!("Switched model to: {} (provider: {})", m.bright_yellow().bold(), p.bright_cyan());
                } else {
                    println!("\nAvailable Models:");
                    for m in agent.router.list_all_models() {
                        let cost_badge = if m.is_free { "[FREE]".bright_green() } else { "[PAID]".bright_red() };
                        println!("  - {:<30} {:<15} {}", m.id.bright_yellow(), m.provider.bright_blue(), cost_badge);
                    }
                    println!();
                }
                continue;
            }

            if input == "/undo" {
                if agent.session.rollback_last_turn() {
                    println!("{}", "Rolled back last turn.".bright_yellow());
                } else {
                    println!("{}", "Nothing to rollback.".bright_red());
                }
                continue;
            }

            // Expand prompt shortcuts if any (/commit, /test, /refactor, /explain)
            let prompt_to_run = PromptTemplates::expand_command(input).unwrap_or_else(|| input.to_string());

            println!();
            
            // Start Spinner
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template("{spinner:.cyan} {msg}")
                    .unwrap(),
            );
            spinner.enable_steady_tick(Duration::from_millis(80));
            spinner.set_message(format!("{} is thinking...", current_model.bright_yellow()));

            let mut first_chunk = false;

            // Use tokio::select! to allow cancelling the LLM stream with Ctrl+C
            let cancel_signal = tokio::signal::ctrl_c();
            
            let turn_future = agent.run_turn(&prompt_to_run, &current_model, |chunk| {
                if !first_chunk {
                    spinner.finish_and_clear();
                    first_chunk = true;
                }

                match chunk {
                    StreamChunk::ThinkingDelta(t) => {
                        print!("{}", t.bright_black());
                        let _ = io::stdout().flush();
                    }
                    StreamChunk::ContentDelta(c) => {
                        print!("{}", c);
                        let _ = io::stdout().flush();
                    }
                    StreamChunk::ToolCallDelta { name, .. } => {
                        if let Some(n) = name {
                            println!("\n{} {}", "🔧 [Tool Calling]".bright_magenta().bold(), n.bright_white().bold());
                        }
                    }
                    _ => {}
                }
            });

            tokio::select! {
                res = turn_future => {
                    if !first_chunk {
                        spinner.finish_and_clear();
                    }
                    println!("\n");
                    if let Err(e) = res {
                        println!("{} {}", "❌ Error:".bright_red().bold(), e);
                    }
                }
                _ = cancel_signal => {
                    spinner.finish_and_clear();
                    println!("\n{} {}", "🛑".bright_red(), "Generation cancelled by user (Ctrl+C).".bright_yellow());
                }
            }
        }

        Ok(())
    }
}
