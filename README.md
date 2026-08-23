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

## 📖 The Official Tutorial: How to Wield Pi Agent

Welcome to your lightning-fast, autonomous pair programmer. Here is how to wield it effectively.

### Step 1: Arming the System (Installation)
Because it’s written in pure Rust, installing it is incredibly simple. 
1. Open your terminal in the project folder.
2. Compile the highly-optimized binary:
   ```bash
   cargo build --release
   ```
3. *(Optional)* Move it to your path so you can use it anywhere:
   ```bash
   sudo cp ./target/release/pi-agent-rust /usr/local/bin/pi-agent
   ```

### Step 2: Loading Your Ammunition (Connecting AI Models)
Pi Agent supports nearly every AI model on Earth. You can use it **100% for free** or plug in premium models.

**Option A: The Zero-Config Web Login (Free Gemini)**  
Don’t want to deal with API keys? Just use your browser!
```bash
pi-agent login gemini
```
*A browser window will pop up. Click "Allow," and you are instantly connected to Google Gemini's massive 1M-token API for free.*

**Option B: Using API Keys (Claude, OpenAI, DeepSeek, xAI)**  
Create a `.env` file in your project folder and paste your keys:
```env
ANTHROPIC_API_KEY="sk-ant-..."
OPENAI_API_KEY="sk-proj-..."
GROQ_API_KEY="gsk_..."
```
*Pi Agent automatically detects them when it starts.*

### Step 3: Calibrating the Radar (Project Memory)
Pi Agent has **Persistent Memory**. Before you ask it to code, let it scan your project so it understands your architecture.
Navigate to any coding project on your computer and run:
```bash
pi-agent init
```
**What happens:** It creates a `.pi/` folder, builds a `PROJECT_MAP.md` (a radar map of all your files), and sets up the safety backup directories.

### Step 4: Engaging the Target (Using the Agent)

**Mode 1: The Interactive Terminal (Standard)**  
Start chatting by simply typing:
```bash
pi-agent
```
Just talk to it naturally! 
> **You:** *"Find the bug in `src/auth.rs` where the password validation fails, and fix it."*

Pi Agent will spin up, use its `grep` tool to read the file, use its `edit` tool to fix the code, and save an atomic `.bak` backup just in case!

**Mode 2: The Command Center (TUI Mode)**  
If you want to feel like a hacker in a movie, launch the Fullscreen Dashboard:
```bash
pi-agent --tui
```
*This splits your screen: Chat on the left, live Project Memory and ROI scores on the right!*

### Step 5: The Master Commands (Shortcuts)
While chatting with the agent, type these `/` commands to instantly control it:

- **`/model <name>`** — Instantly swap the AI’s brain mid-conversation! 
  - *Type `/model sonnet` to use Claude 3.7.*
  - *Type `/model r1` to switch to DeepSeek R1 for heavy math.*
  - *Type `/model ollama` for free local offline coding.*
- **`/undo`** — If the AI makes a mistake, roll back the conversation.
- **`/memory`** — Prints a summary of every bug you've fixed and decision you've made.
- **`/score`** — Displays your **Failure Prevention Score** (shows you how many developer hours and API tokens the agent has saved you!).

---

## 🛠️ Built-In Tools & Tech Stack
- **Rust Native Tools:** `read`, `write`, `edit` (Fuzzy matching), `bash` (streamed execution), `grep` (ripgrep-fast), `find_files`.
- **Async Engine:** Powered by `Tokio`.
- **Parallelism:** Powered by `Rayon` (Dynamic hardware auto-tuning).
- **TUI & REPL:** Powered by `Ratatui`, `Crossterm`, and `Indicatif`.

---
*Ready to hunt some bugs? Download, build, and deploy.* ⚡
