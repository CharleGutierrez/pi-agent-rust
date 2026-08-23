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
*Ready to hunt some bugs? Download, build, and deploy.* ⚡
