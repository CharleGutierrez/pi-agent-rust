use colored::*;

pub struct Banner;

impl Banner {
    pub fn print_startup(model: &str, provider: &str, session_id: &str, project_name: &str) {
        println!("{}", r#"
 ██████╗ ██╗     ██████╗ ██████╗ ███████╗
 ██╔══██╗██║    ██╔════╝██╔═══██╗██╔════╝
 ██████╔╝██║    ██║     ██║   ██║███████╗
 ██╔═══╝ ██║    ██║     ██║   ██║██╔════╝
 ██║     ██║    ╚██████╗╚██████╔╝███████╗
 ╚═╝     ╚═╝     ╚═════╝ ╚═════╝ ╚══════╝
  Superpowerful Pure-Rust AI Coding Agent & Persistent Memory
        "#.bright_cyan().bold());

        println!(" {}", "═".repeat(64).bright_black());
        println!("  {} {} {}", "⚡ Target Model:".bright_yellow().bold(), model.bright_white().bold(), format!("({})", provider).bright_black());
        println!("  {} {}", "🧠 AI Memory:   ".bright_green().bold(), "Persistent Event Log & Semantic Map Active".bright_white());
        println!("  {} {}", "📁 Project:     ".bright_blue().bold(), project_name.bright_white());
        println!("  {} {}", "🆔 Session:     ".bright_magenta().bold(), session_id.bright_black());
        println!(" {}", "═".repeat(64).bright_black());
        println!("  Type your prompt or commands: {} {} {} {} {} {}",
            "/model".bright_cyan(),
            "/memory".bright_cyan(),
            "/plan".bright_cyan(),
            "/score".bright_cyan(),
            "/undo".bright_cyan(),
            "/exit".bright_cyan()
        );
        println!(" {}\n", "─".repeat(64).bright_black());
    }
}
