use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pi-agent",
    about = "Superpowerful Pure-Rust Pi Coding Agent with Persistent AI Memory & Multi-LLM Routing",
    version = "0.1.0"
)]
pub struct Cli {
    /// Target LLM model alias (e.g. sonnet, 4o, r1, flash, groq, ollama, deepseek)
    #[arg(short, long, global = true)]
    pub model: Option<String>,

    /// Project workspace directory (default: current directory)
    #[arg(short, long, global = true)]
    pub dir: Option<PathBuf>,

    /// Launch full-screen interactive TUI dashboard
    #[arg(long, global = true)]
    pub tui: bool,

    /// Start JSON-RPC 2.0 protocol server for IDE/GUI integrations
    #[arg(long, global = true)]
    pub rpc: bool,

    /// Non-interactive one-shot prompt execution
    #[arg(short = 'p', long)]
    pub prompt: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize AI Coding Memory (.pi/memory, PROJECT_MAP.md, plan.md)
    Init,

    /// Inspect or display distilled AI Coding Memory
    Memory {
        /// Show failure-prevention score and ROI
        #[arg(long)]
        score: bool,

        /// Search memory events by query keyword
        #[arg(short, long)]
        search: Option<String>,
    },

    /// List all available LLM models (Free tier & Paid)
    Models,

    /// Add a custom LLM model to config
    ModelAdd {
        /// ID of the model (e.g. 'my-vllm')
        #[arg(long)]
        id: String,
        
        /// Human readable name
        #[arg(long)]
        name: String,
        
        /// Provider type (openai, ollama, anthropic, gemini)
        #[arg(long)]
        provider: String,
        
        /// Base URL (e.g. http://127.0.0.1:8000/v1)
        #[arg(long)]
        base_url: Option<String>,
        
        /// Environment variable name for the API key (e.g. CUSTOM_API_KEY)
        #[arg(long)]
        api_key_env: Option<String>,
        
        /// Context window size
        #[arg(long, default_value = "32000")]
        context_window: u32,
    },

    /// Precheck a file's failure history before editing
    Precheck {
        /// Project-relative path to check
        path: String,
    },

    /// Show or manage project intent and plan
    Plan,

    /// Authenticate via OAuth Web Browser (e.g. 'gemini')
    Login {
        /// The provider to login to (currently 'gemini' is supported)
        provider: String,
    }
}
