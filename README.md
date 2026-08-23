<div align="center">

# ⚡ Pi Agent (Rust Edition)
**The Superpowerful, Ultra-Lightweight, Autonomous AI Coding Agent**

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg?style=for-the-badge&logo=rust)](#)
[![Powered by LLMs](https://img.shields.io/badge/Powered%20by-Multi--LLM-blue.svg?style=for-the-badge&logo=openai)](#)
[![Zero Overhead](https://img.shields.io/badge/Overhead-Zero-success.svg?style=for-the-badge)](#)

</div>

---

## 🌟 The Ultimate Bug-Hunting Weapon

Forget bulky Node.js/Python dependencies, slow startup times, and rigid ecosystem lock-in. **Pi Agent (Rust Edition)** is a blazing-fast, 15MB pure-Rust binary that lives locally on your machine. 

It is an autonomous pair-programmer armed with an **Embedded Persistent Memory Engine** and a **Multi-LLM Router**. It actively scans your codebase, remembers every bug you've ever faced, dynamically auto-tunes itself to your CPU cores, and surgically edits your code with atomic safety backups.

### 🔥 Why It Dominates

- **🧠 Persistent AI Coding Memory:** It never forgets. It logs bugs, attempts, and fixes. Before editing a file, it checks its "radar" to ensure it never repeats a failed dead-end fix.
- **🌐 Universal LLM Routing:** Swap models mid-conversation. Use **100% Free** models (Ollama, Groq, OpenRouter, Gemini Flash) or premium frontier models (Claude 3.7 Sonnet, DeepSeek R1, GPT-4o, xAI Grok, Mistral).
- **🚀 Blazing Fast & Zero-Overhead:** Starts in `<5ms`. No garbage collector. Scales dynamically to your hardware—from a 7th Gen Intel dual-core to a massive 32-core AMD Ryzen workstation using Rayon thread-pools.
- **🛡️ Weapon-Grade Safety:** Uses "Smart-Whitespace Fuzzy Matching" to edit files accurately, and creates instant `.bak` atomic snapshots *before* altering your code.
- **🖥️ Beautiful UX:** Choose between an interactive REPL with live progress spinners or a Fullscreen Ratatui Terminal Dashboard (`--tui`).

---

## 💾 Global Installation Guide

Because Pi Agent is written in pure Rust, it compiles into a single, standalone binary. Once compiled, you can summon the AI from *any* folder on your computer just by typing `pi-agent`.

**Prerequisite:** Ensure you have Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

### 🍏 For Mac / 🐧 Linux
Open your terminal and run:
```bash
# 1. Download the repository
git clone https://github.com/CharleGutierrez/pi-agent-rust.git
cd pi-agent-rust

# 2. Build the optimized release binary
cargo build --release

# 3. Integrate it globally into your system
sudo cp target/release/pi-agent-rust /usr/local/bin/pi-agent

# 4. Verify it works!
pi-agent --version
```

### 🪟 For Windows (PowerShell)
Open PowerShell as Administrator and run:
```powershell
# 1. Download the repository
git clone https://github.com/CharleGutierrez/pi-agent-rust.git
cd pi-agent-rust

# 2. Build the optimized release binary
cargo build --release

# 3. Integrate it globally by creating a Tools directory
New-Item -ItemType Directory -Force -Path "C:\Tools"
Copy-Item -Path "target\release\pi-agent-rust.exe" -Destination "C:\Tools\pi-agent.exe"

# 4. Add it to your System PATH
[System.Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Tools", "User")

# 5. Verify it works! (Restart PowerShell first)
pi-agent --version
```

---

## 🚀 How to Use Pi Agent (Complete Guide)

### 1. Setting Up LLM Providers & API Keys

Set the environment variable for your preferred AI provider before launching Pi Agent:

| Provider | Environment Variable / Setup | Default / Common Aliases | Cost Tier |
|---|---|---|---|
| **Google Gemini** | `export GEMINI_API_KEY="your-key"` *or* `pi-agent login gemini` | `flash`, `gemini`, `gemini-pro` | Free 1M Tier / Paid |
| **Anthropic** | `export ANTHROPIC_API_KEY="your-key"` | `sonnet`, `claude-3-7`, `haiku`, `opus` | Paid Frontier |
| **OpenAI** | `export OPENAI_API_KEY="your-key"` | `4o`, `4o-mini`, `o1`, `o3-mini` | Paid Frontier |
| **DeepSeek** | `export DEEPSEEK_API_KEY="your-key"` | `deepseek`, `r1` | Low Cost / Paid |
| **Groq** | `export GROQ_API_KEY="your-key"` | `groq`, `groq-r1`, `groq-qwen` | Ultra-Fast Free Tier |
| **OpenRouter** | `export OPENROUTER_API_KEY="your-key"` | `free-r1`, `free-llama` | Free & Paid |
| **Local Ollama** | `ollama run qwen2.5-coder` (host: `http://127.0.0.1:11434`) | `ollama`, `ollama-qwen` | 100% Free & Offline |

---

### 2. Getting Started in a Project

#### Step 1: Initialize Workspace Memory
Run `init` in your project root to initialize `.pi/` memory tracking, `PROJECT_MAP.md`, and `plan.md`:
```bash
pi-agent init
```

#### Step 2: Launch an Interactive Session
Start the interactive pair-programming REPL with your chosen model:
```bash
# Using Gemini 2.0 Flash (Fast & Free Tier)
pi-agent -m flash

# Using Claude 3.7 Sonnet (Deep Reasoning & Autonomous Coding)
pi-agent -m sonnet

# Using local Ollama (100% Offline & Private)
pi-agent -m ollama
```

#### Step 3: Fullscreen TUI Dashboard Mode
For a split-pane terminal dashboard (live chat on the left, persistent memory radar and file map on the right):
```bash
pi-agent --tui
```

#### Step 4: One-Shot Execution (Headless)
Run a single prompt directly from terminal and exit immediately:
```bash
pi-agent -m flash -p "Find and fix any broken unit tests in src/"
```

---

### 3. Interactive REPL Slash Commands Reference

Inside the interactive chat prompt (`❯`), use these built-in slash commands:

| Command | Description |
|---|---|
| `/model` | List all available models (both free and paid tiers) |
| `/model <alias>` | Hot-swap models mid-session (e.g., `/model sonnet`, `/model r1`, `/model ollama`) |
| `/score` | View the failure-prevention score, debugging hours saved, and token ROI |
| `/memory` or `/summary` | Display distilled project memory, logged issues, decisions, and notes |
| `/map` | Display the project structure map and architectural relationships |
| `/plan` | View and manage current tasks, ideas, and roadmap in `plan.md` |
| `/test` | Automatically runs test suites, captures failures into memory, and applies verified fixes |
| `/refactor <file>` | Prechecks failure history and safely refactors target file |
| `/explain <file>` | Reads and explains the architecture and relationships of a target file |
| `/commit` | Analyzes git diff and writes a high-quality conventional git commit message |
| `/undo` | Reverts the last conversation turn |
| `Ctrl + C` | Instantly and cleanly aborts active streaming generation without corrupting history |
| `/exit` or `/quit` | Saves session memory and exits |

---

### 4. CLI Commands & Subcommands Reference

```bash
# Display failure-prevention score & ROI report
pi-agent memory --score

# Search memory logs for past bug fixes or architectural decisions
pi-agent memory --search "auth token"

# Check failure history of a file before modifying it
pi-agent precheck src/auth.rs

# View current project intent and plan
pi-agent plan

# Add custom local or corporate endpoint (vLLM / LM Studio / OpenAI-compatible)
pi-agent model-add \
    --id "corp-llama" \
    --name "Corporate Llama 3" \
    --provider "openai" \
    --base-url "http://10.0.0.50:8000/v1"

# List all supported models
pi-agent models
```

---

## 📖 The Pi Agent Masterclass: Real-World Scenarios

Pi Agent is not just a chatbot; it is an autonomous developer that lives in your terminal. Because it has tools to read files, edit code, and remember past mistakes, you use it differently depending on your situation.

Here are the 5 most common scenarios and exactly how to handle them.

### 🐣 Scenario 1: Starting a Brand New Project (The Setup)
*You just created a fresh folder for a new app, and you want Pi Agent to help you build it from scratch.*

**1. Initialize the AI Memory:**
Open your terminal in your new project folder and run:
```bash
pi-agent init
```
*Why?* This creates a `.pi/` folder. The agent will use this to build a map of your files (`PROJECT_MAP.md`) and keep a diary of every bug it fixes (`events.jsonl`) so it never makes the same mistake twice.

**2. Login without API Keys:**
If you don't have a paid developer API key, no problem. Just log in using your browser:
```bash
pi-agent login gemini
```
*Why?* This securely connects the agent to Google's massive 1M-token free tier. You are now ready to code.

---

### 🐛 Scenario 2: The "I'm Stuck on a Nasty Bug" Situation
*Your code is broken, it's 2:00 AM, and you are tired of reading error logs.*

**1. Launch the Hacker Dashboard (TUI):**
```bash
pi-agent --tui
```
*Why?* This opens a fullscreen dashboard. On the right, you will see the agent's persistent memory. On the left, you will see the chat.

**2. Give the Agent a Target:**
Type this into the chat prompt:
> *"I am getting a 'Null Pointer Exception' when a user tries to log in. Please find the bug in `src/auth.rs` and fix it."*

**3. Watch it Work:**
- You will see a colored spinner: `[Model] is thinking...`
- It will automatically use the `read` tool to look at `src/auth.rs`.
- It will instantly drop a backup of your file into `.pi/backups/` so you are safe.
- It will use the `edit` tool to fix the code automatically.

**4. Verify the Fix:**
Type `/test` in the chat. The AI will run your test suite, verify its own code, and close the issue in its memory ledger!

---

### 🕵️ Scenario 3: Working Offline or with Extreme Privacy
*You are on an airplane with no Wi-Fi, or you are working on highly confidential corporate code that cannot be sent to OpenAI or Google.*

**1. Start the Local Server:**
Ensure you have [Ollama](https://ollama.com/) installed on your computer, and run a free open-source model locally:
```bash
ollama run qwen2.5-coder
```

**2. Connect Pi Agent to the Local Model:**
Tell Pi Agent to switch its brain to your local offline server:
```bash
pi-agent -m ollama
```
*Why?* The agent will now process everything 100% locally on your machine's hardware. Nothing leaves your laptop.

---

### 🏢 Scenario 4: The Enterprise / Custom Server Environment
*Your company hosts its own private AI server (like vLLM or LM Studio), and you need to connect Pi Agent to it.*

**1. Add the Custom Model to Pi Agent:**
You don't need to change any Rust code. Just use the built-in configuration wizard:
```bash
pi-agent model-add \
    --id "corp-ai" \
    --name "Corporate Llama 3" \
    --provider "openai" \
    --base-url "http://10.0.0.50:8000/v1" 
```

**2. Start Coding:**
```bash
pi-agent -m corp-ai
```
*Why?* Pi Agent dynamically saves this configuration globally. From now on, you can instantly hot-swap to your corporate AI mid-conversation by typing `/model corp-ai`.

---

### 🌪️ Scenario 5: Massive Codebase Refactoring
*You inherited a massive, messy 10,000-file repository and need the AI to clean it up.*

**1. Let the AutoTuner do the heavy lifting:**
Because Pi Agent is written in Rust, it has an `AutoTuner` that detects your hardware. 

Simply launch the agent on your workstation:
```bash
pi-agent -m sonnet
```
*Why?* The agent will detect your CPU cores and available RAM. If you tell it to search the codebase, it will automatically unleash Rayon thread-pools to parallel-search thousands of files in milliseconds.

**2. Use the Refactor Magic Word:**
Point the agent at a messy file and type:
```bash
/refactor src/messy_database.rs
```
*Why?* The `/refactor` shortcut automatically instructs the AI to check the file's failure history, analyze its architecture, clean up duplication, apply safety backups, and rewrite the code to production standards.

---

### 🛑 Emergency Bailout (The "Oops" Button)
Did the AI generate a massive block of code you didn't ask for, or did you change your mind mid-generation?

- **Action:** Press `Ctrl + C`
- **Result:** The agent will cleanly abort the network stream instantly. It will not crash, it will not corrupt your terminal, and it will keep your chat history perfectly intact so you can type a new prompt.

---

## 🛠️ Built-In Tools & Tech Stack
- **Rust Native Tools:** `read`, `write`, `edit` (Fuzzy matching), `bash` (streamed execution), `grep` (ripgrep-fast), `find_files`.
- **Async Engine:** Powered by `Tokio`.
- **Parallelism:** Powered by `Rayon` (Dynamic hardware auto-tuning).
- **TUI & REPL:** Powered by `Ratatui`, `Crossterm`, and `Indicatif`.

---

## 🏗️ Deep Architectural Study & System Internals

```
                               ┌────────────────────────────────────────────────────────┐
                               │                    CLI / Entrypoint                    │
                               │                  (src/main.rs, cli.rs)                 │
                               └─────────┬───────────────────┬──────────────────┬───────┘
                                         │                   │                  │
                         ┌───────────────▼────────┐ ┌────────▼────────┐ ┌───────▼────────┐
                         │   CliInteractive REPL  │ │  Ratatui TUI    │ │ Headless RPC   │
                         │(Indicatif / Crossterm) │ │ (Split-Pane UI) │ │ (JSON-RPC stdio│
                         └───────────────┬────────┘ └────────┬────────┘ └───────┬────────┘
                                         └───────────────────┼──────────────────┘
                                                             ▼
                                                ┌─────────────────────────┐
                                                │      AgentEngine        │
                                                │  (src/agent/engine.rs)  │
                                                └────────────┬────────────┘
                                                             │
        ┌────────────────────────────┬───────────────────────┼───────────────────────┬────────────────────────────┐
        ▼                            ▼                       ▼                       ▼                            ▼
┌──────────────┐             ┌──────────────┐        ┌──────────────┐        ┌──────────────┐             ┌──────────────┐
│ Provider     │             │ Tool         │        │ Memory       │        │ AutoTuner    │             │ Reflection & │
│ Router       │             │ Registry     │        │ Engine       │        │ & Compactor  │             │ Planner      │
│(src/providers│             │(src/tools/)  │        │(src/memory/) │        │(src/agent/)  │             │(src/agent/)  │
└───────┬──────┘             └───────┬──────┘        └───────┬──────┘        └──────────────┘             └──────────────┘
        │                            │                       │
 ┌──────┴───────────────┐     ┌──────┴────────────────┐      │  .pi/ / .projectmem/
 │ OpenAI, Anthropic,   │     │ read, write, edit,    │      ├─ events.jsonl (Audit Log)
 │ Gemini (OAuth/API),  │     │ bash, grep, git,      │      ├─ summary.md (Distilled)
 │ DeepSeek, Groq,      │     │ find_files, web_fetch,│      ├─ PROJECT_MAP.md (Paths)
 │ Ollama, OpenRouter,  │     │ + Memory MCP Tools    │      └─ plan.md (Intent)
 │ xAI, Mistral         │     └───────────────────────┘
 └──────────────────────┘
```

### 1. Autonomous ReAct Agent Loop (`src/agent/`)
- **`AgentEngine` (`engine.rs`)**: Manages the autonomous multi-turn ReAct reasoning loop. Streams model responses, dispatches tool execution, handles prompt injection, and manages session history checkpoints.
- **`ReflectionEngine` (`reflection.rs`)**: Diagnoses tool errors (compilation errors, test assertion panics, edit conflicts, timeouts) and feeds actionable diagnostic hints back to the LLM to self-correct automatically.
- **`AutoTuner` (`auto_tuner.rs`)**: Introspects host hardware via `sysinfo` (logical cores, RAM) and active model context capacity to auto-scale Rayon thread-pool concurrency and compaction thresholds.
- **`ContextCompactor` (`compaction.rs`)**: Dynamically truncates conversation history to fit within the active model's token limits while preserving system instructions and recent turns.
- **`ExecutionPlan` (`planner.rs`)**: Tracks task decomposition step-by-step with state indicators (`[ ]`, `[->]`, `[x]`).

### 2. Embedded Persistent Memory Engine (`src/memory/`)
- **Immutable Event Ledger (`storage.rs`, `events.rs`)**: Appends structured events (`Issue`, `Attempt`, `Fix`, `Decision`, `Note`) to `.pi/events.jsonl` with sequential zero-padded IDs (`#0001`, `#0002`).
- **Pre-Flight File Radar (`precheck.rs`)**: Checked before modifying any file to surface past failed approaches, unresolved issues, and high churn.
- **Structural Mapping (`project_map.rs`, `intent_plan.rs`)**: Generates and maintains `PROJECT_MAP.md` as a token-efficient path index and `plan.md` as an intent tracker.
- **ROI Scoring Engine (`scoring.rs`, `search.rs`)**: Quantifies debugging hours saved and token waste prevented, and provides fast substring search across event history.

### 3. Multi-LLM Provider Router (`src/providers/`)
- **Unified Router (`router.rs`)**: Hot-swappable routing across 9+ providers: OpenAI, Anthropic, Gemini, Groq, Ollama, OpenRouter, DeepSeek, xAI, and Mistral.
- **Zero-Key Google OAuth (`auth/google.rs`)**: Browser-based OAuth 2.0 loop on `127.0.0.1:8080/callback` with automated token refresh for Gemini models.
- **Real-Time Token Streaming (`traits.rs`, `types.rs`)**: Low-latency token streaming with dedicated thinking/reasoning delta support.

### 4. Precision Tool Suite & Safe Actuation (`src/tools/`)
- **Smart-Whitespace Fuzzy Editor (`edit.rs`)**: 3-tier matching engine (Exact -> Normalized Line-Endings -> Fuzzy Indentation) with automatic `.bak` atomic snapshot generation in `.pi/backups/`.
- **Workspace & Terminal Tools**: `read`, `write`, `bash` (sandboxed timeout execution), `grep`, `find_files`, `git`, `web_fetch`.
- **Embedded Memory Tools (`memory_tools.rs`)**: Native Rust implementation of all projectmem tools (`log_issue`, `record_attempt`, `record_fix`, `add_decision`, `add_note`, `precheck_file`, `get_summary`, etc.).

### 5. Multi-Modal User Interfaces (`src/ui/`)
- **CLI Interactive REPL (`cli_interactive.rs`)**: Colorized interactive shell with animated progress spinners, Ctrl+C stream cancellation, and slash commands (`/score`, `/memory`, `/map`, `/plan`, `/model`, `/undo`, `/test`, `/commit`).
- **Ratatui TUI Dashboard (`tui.rs`)**: Fullscreen split-pane terminal interface showing live conversation on the left and active memory radar / project map on the right.
- **Headless JSON-RPC Server (`rpc.rs`)**: Standard I/O JSON-RPC interface for IDE extensions, automated harnesses, and external tooling.

---
*Ready to hunt some bugs? Download, build, and deploy.* ⚡

---

## 🕵️ What If Intelligence Agencies Used This? (A Scenario)

If **FBI Cyber Division** or **CIA CNO (Computer Network Operations)** personnel got their hands on this tool for a high-stakes cybersecurity investigation, their reactions would likely bounce between sheer disbelief and absolute relief:

### 1. The SCIF Reaction (Air-Gapped Offline Mode)
> *"Wait, we don't need the internet for this?"*

Intelligence analysts work inside SCIFs (Sensitive Compartmented Information Facilities) where internet access is strictly forbidden. They cannot legally send classified malware code to OpenAI or Google's servers. By running `pi-agent -m ollama`, the agent runs massive code-cracking models **100% locally and offline** on their classified servers. 

### 2. The Night-Shift Reaction (Persistent Memory)
> *"Thank God, I don't have to read Jim's messy hand-over notes."*

When analyzing a massive zero-day exploit, teams work in shifts. Because Pi Agent uses a **Persistent Memory Ledger**, a night-shift analyst just runs `pi-agent memory`. The agent instantly prints out exactly which decryption keys failed during the day shift, which files were already audited, and what the active plan is. It mathematically prevents agents from repeating a dead-end.

### 3. The Reverse-Engineer's Reaction (Rayon Parallel Searching)
> *"Did it just search 400,000 files in 2 seconds?"*

When investigating a hacked enterprise server, agents sift through gigabytes of obfuscated logs. Because of the built-in **AutoTuner**, Pi Agent detects the 64-core processors on forensic servers. When asked to *"find the backdoor,"* the agent unleashes a massive Rayon thread-pool, searching the entire server architecture in milliseconds.

### 4. The "Oops" Reaction (Atomic Backups)
> *"I thought I just destroyed the evidence."*

In cyber forensics, altering the original malware is a massive violation of protocol. If an analyst accidentally asks the AI to "clean up this code" and it modifies the evidence, they'd normally be in trouble. But because of **Atomic Safety Backups**, the exact millisecond before the AI acts, it drops a pristine `.bak` file into `.pi/backups/`. The evidence is preserved automatically.
