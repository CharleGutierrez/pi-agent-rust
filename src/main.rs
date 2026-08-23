use anyhow::Result;
use clap::Parser;
use colored::*;
use pi_agent_rust::agent::AgentEngine;
use pi_agent_rust::auth::GoogleAuth;
use pi_agent_rust::cli::{Cli, Commands};
use pi_agent_rust::config::config::{AppConfig, CustomModelConfig};
use pi_agent_rust::providers::types::StreamChunk;
use pi_agent_rust::ui::{CliInteractive, RpcServer, TuiDashboard};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Environment Variables (e.g. .env)
    let _ = dotenvy::dotenv();

    // Parse CLI arguments
    let cli = Cli::parse();
    let work_dir = cli
        .dir
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // 2. Initialize Production File Logger
    let log_dir = work_dir.join(".pi/logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = rolling::daily(&log_dir, "agent.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    tracing::info!("Starting Pi Agent Rust...");

    let config = AppConfig::load_or_default(&work_dir);
    let target_model = cli.model.unwrap_or_else(|| config.default_model.clone());

    // Initialize Agent Engine
    let mut agent = AgentEngine::new(work_dir.clone(), &config, Some(target_model.clone()), Some(config.max_turns))?;

    // Handle Subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Init => {
                agent.memory.auto_sync()?;
                println!("{}", "✨ Successfully initialized Pi Agent Coding Memory!".bright_green().bold());
                println!("  ├── Memory Dir:    {}", agent.memory.storage.memory_dir().display().to_string().bright_cyan());
                println!("  ├── Project Map:   {}", agent.memory.storage.memory_dir().join(pi_agent_rust::memory::storage::MAP_FILE).display().to_string().bright_cyan());
                println!("  ├── Plan Intent:   {}", agent.memory.storage.memory_dir().join(pi_agent_rust::memory::storage::PLAN_FILE).display().to_string().bright_cyan());
                println!("  └── Event Log:     {}", agent.memory.storage.events_path().display().to_string().bright_cyan());
                return Ok(());
            }
            Commands::Memory { score, search } => {
                if score {
                    let s = agent.memory.get_score()?;
                    println!("\n{}\n", s.formatted_report().bright_cyan());
                    return Ok(());
                }
                if let Some(q) = search {
                    let results = agent.memory.search(&q, 10)?;
                    if results.is_empty() {
                        println!("No events found for query: '{}'", q);
                    } else {
                        println!("Found {} events for '{}':", results.len(), q);
                        for r in results {
                            println!("- (score {:.1}) {}", r.score, r.snippet);
                        }
                    }
                    return Ok(());
                }
                let summary = agent.memory.get_summary()?;
                println!("\n{}\n", summary.bright_white());
                return Ok(());
            }
            Commands::Models => {
                println!("\n{}", "=== Available Models (Free & Paid) ===".bright_cyan().bold());
                for m in agent.router.list_all_models() {
                    let badge = if m.is_free {
                        "[FREE]".bright_green().bold()
                    } else {
                        "[PAID]".bright_red()
                    };
                    println!(
                        "  {:<32} {:<12} {} (Context: {}k tok)",
                        m.id.bright_yellow(),
                        m.provider.bright_blue(),
                        badge,
                        m.context_window / 1000
                    );
                }
                println!();
                return Ok(());
            }
            Commands::ModelAdd { id, name, provider, base_url, api_key_env, context_window } => {
                let mut cfg = config.clone();
                cfg.custom_models.push(CustomModelConfig {
                    id: id.clone(),
                    name: name.clone(),
                    provider_type: provider.clone(),
                    base_url,
                    api_key_env,
                    context_window,
                    supports_tools: true,
                    supports_thinking: false,
                });
                
                if let Err(e) = cfg.save_global() {
                    println!("{} Failed to save config: {}", "❌".bright_red(), e);
                } else {
                    println!("{} Successfully added custom model: {}", "✅".bright_green(), id.bright_yellow());
                }
                return Ok(());
            }
            Commands::Precheck { path } => {
                let report = agent.memory.precheck_file(&path)?;
                if report.has_warnings {
                    println!("{}", report.guidance.bright_yellow());
                } else {
                    println!("{}", report.guidance.bright_green());
                }
                return Ok(());
            }
            Commands::Plan => {
                let plan_path = agent.memory.storage.memory_dir().join(pi_agent_rust::memory::storage::PLAN_FILE);
                if plan_path.exists() {
                    let content = std::fs::read_to_string(plan_path)?;
                    println!("\n{}\n", content.bright_yellow());
                } else {
                    println!("No plan.md found. Run `pi-agent init` first.");
                }
                return Ok(());
            }
            Commands::Login { provider } => {
                if provider.to_lowercase() == "gemini" || provider.to_lowercase() == "google" {
                    GoogleAuth::authenticate_via_browser().await?;
                    println!("{}", "Successfully authenticated with Google. You can now use Gemini models without setting GEMINI_API_KEY.".bright_green());
                } else {
                    println!("{} Unsupported OAuth provider: {}. Try 'gemini'.", "❌".bright_red(), provider);
                }
                return Ok(());
            }
        }
    }

    // Handle RPC Mode
    if cli.rpc {
        return RpcServer::run(agent, &target_model).await;
    }

    // Handle Fullscreen TUI Mode
    if cli.tui {
        return TuiDashboard::run(agent, &target_model).await;
    }

    // Handle One-Shot Non-Interactive Prompt
    if let Some(prompt) = cli.prompt {
        let (p, m) = agent.router.resolve_alias(&target_model);
        eprintln!("⚡ Executing on {} ({})", m.bright_yellow(), p.bright_cyan());

        let _res = agent
            .run_turn(&prompt, &target_model, |chunk| match chunk {
                StreamChunk::ContentDelta(c) => {
                    print!("{}", c);
                    let _ = io::stdout().flush();
                }
                _ => {}
            })
            .await?;

        println!();
        return Ok(());
    }

    // Default: Run Terminal Interactive REPL
    CliInteractive::run(agent, &target_model).await
}
