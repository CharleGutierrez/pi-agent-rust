use crate::providers::types::ModelInfo;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct TunedProfile {
    pub logical_cores: usize,
    pub available_ram_gb: u64,
    pub max_parallel_workers: usize,
    pub optimal_context_window: usize,
    pub hardware_grade: String,
}

pub struct AutoTuner;

impl AutoTuner {
    /// Detects hardware and active LLM model to auto-tune internal processing and concurrency
    pub fn auto_tune_system(active_model: &ModelInfo) -> TunedProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let logical_cores = sys.cpus().len().max(1);
        let total_ram_bytes = sys.total_memory();
        let available_ram_gb = total_ram_bytes / (1024 * 1024 * 1024);

        // Auto-tune concurrency limits
        // If older CPU (e.g. 2-4 cores), limit workers so we don't freeze the OS.
        // If modern CPU (e.g. 16-32 cores), blast it with heavy parallelization.
        let max_parallel_workers = if logical_cores <= 4 {
            logical_cores // Safe mode
        } else {
            logical_cores * 2 // Aggressive I/O parallel mode
        };

        // Hardware Grade for debugging/UI
        let hardware_grade = if logical_cores <= 4 {
            "Standard (Dual/Quad Core)".to_string()
        } else if logical_cores <= 12 {
            "Performance (Octa Core+)".to_string()
        } else {
            "Ultra-Threaded (Workstation/Server)".to_string()
        };

        // Auto-tune context compaction threshold based on model memory
        let context_cap = active_model.context_window as usize;
        let optimal_context_window = if context_cap > 1_000_000 {
            // e.g. Gemini 1.5 Pro / Flash
            800_000
        } else if context_cap >= 128_000 {
            // e.g. GPT-4o, Claude 3.5, Llama 3.3
            100_000
        } else {
            // e.g. DeepSeek R1, older models
            (context_cap as f64 * 0.8) as usize
        };

        // Configure global thread pool for Rayon (used by fast search tools)
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(max_parallel_workers)
            .build_global();

        TunedProfile {
            logical_cores,
            available_ram_gb,
            max_parallel_workers,
            optimal_context_window,
            hardware_grade,
        }
    }
}
