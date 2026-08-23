use crate::agent::AgentEngine;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

pub struct TuiDashboard;

impl TuiDashboard {
    pub async fn run(mut agent: AgentEngine, model: &str) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut input_buffer = String::new();
        let mut output_lines: Vec<String> = vec![
            format!("Pi Coding Agent (Rust Edition) - Full TUI Initialized"),
            format!("Active Model: {} | Memory: Persistent JSONL Active", model),
            format!("Press ESC to exit, Enter to submit query, /score for ROI score"),
        ];

        loop {
            let summary_text = agent.memory.get_summary().unwrap_or_else(|_| "No memory".to_string());
            let score_text = agent
                .memory
                .get_score()
                .map(|s| format!("Grade: {} | ROI: ${:.2} | Saved: {:.1}h", s.grade, s.dollars_saved, s.hours_saved))
                .unwrap_or_else(|_| "Score: N/A".to_string());

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // Header
                        Constraint::Min(10),   // Main View (Chat + Memory)
                        Constraint::Length(3),  // Input bar
                    ])
                    .split(f.area());

                // Header
                let header_title = format!(" Pi Coding Agent v0.1.0 ⚡ [{}] | {}", model, score_text);
                let header = Paragraph::new(header_title)
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(header, chunks[0]);

                // Split Main View horizontally (Chat vs Project Memory)
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(65), // Chat Output
                        Constraint::Percentage(35), // Memory & Map
                    ])
                    .split(chunks[1]);

                // Chat Paragraph
                let chat_text: Vec<Line> = output_lines
                    .iter()
                    .rev()
                    .take(30)
                    .rev()
                    .map(|l| Line::from(l.as_str()))
                    .collect();

                let chat_box = Paragraph::new(chat_text)
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL).title("Agent Chat / Stream"));
                f.render_widget(chat_box, main_chunks[0]);

                // Memory Summary Panel
                let mem_box = Paragraph::new(summary_text)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(Color::Green))
                    .block(Block::default().borders(Borders::ALL).title("AI Coding Memory (Persistent)"));
                f.render_widget(mem_box, main_chunks[1]);

                // Input Bar
                let input = Paragraph::new(input_buffer.as_str())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Prompt Input (Press Enter to execute)"));
                f.render_widget(input, chunks[2]);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char(c) => input_buffer.push(c),
                        KeyCode::Backspace => {
                            input_buffer.pop();
                        }
                        KeyCode::Enter => {
                            let prompt = input_buffer.trim().to_string();
                            input_buffer.clear();

                            if !prompt.is_empty() {
                                if prompt == "/exit" || prompt == "exit" {
                                    break;
                                }

                                output_lines.push(format!("❯ {}", prompt));

                                // Run agent turn
                                let res = agent
                                    .run_turn(&prompt, model, |_chunk| {})
                                    .await;

                                match res {
                                    Ok(ans) => {
                                        for line in ans.lines() {
                                            output_lines.push(line.to_string());
                                        }
                                    }
                                    Err(e) => {
                                        output_lines.push(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
