use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};
use log::info;
use serde::Serialize;

const BYTES_TO_KB: u64 = 1024;

#[derive(Default, Serialize, Clone)]
pub struct ResourceMetrics {
    max_memory_kb: u64,
    min_memory_kb: u64,
    avg_memory_kb: u64,
    max_cpu_percent: f32,
    min_cpu_percent: f32,
    avg_cpu_percent: f32,
    samples: usize,
}

#[derive(Default, Serialize, Clone)]
pub struct TimingMetrics {
    pub workspace_setup_duration: Option<Duration>,
    pub compilation_duration: Option<Duration>,
    pub proof_generation_duration: Option<Duration>,
    pub total_duration: Option<Duration>,
}

#[derive(Default, Serialize, Clone)]
pub struct TelemetryData {
    pub timing: TimingMetrics,
    pub resources: ResourceMetrics,
    pub proving_system: String,
    pub precompiles_enabled: bool,
}

pub struct TelemetryCollector {
    start_time: Instant,
    system: System,
    metrics: Arc<Mutex<TelemetryData>>,
    enabled: bool,
    resource_samples: Arc<Mutex<Vec<(u64, f32)>>>, // (memory_kb, cpu_percent)
}

impl TelemetryCollector {
    pub fn new(proving_system: &str, precompiles_enabled: bool, enabled: bool) -> Self {
        let mut system = System::new();
        system.refresh_all();
        
        let mut metrics = TelemetryData::default();
        metrics.proving_system = proving_system.to_string();
        metrics.precompiles_enabled = precompiles_enabled;

        Self {
            start_time: Instant::now(),
            system,
            metrics: Arc::new(Mutex::new(metrics)),
            enabled,
            resource_samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_workspace_setup(&self, duration: Duration) {
        if !self.enabled { return; }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.workspace_setup_duration = Some(duration);
        }
    }

    pub fn record_compilation(&self, duration: Duration) {
        if !self.enabled { return; }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.compilation_duration = Some(duration);
        }
    }

    pub fn record_proof_generation(&self, duration: Duration) {
        if !self.enabled { return; }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.proof_generation_duration = Some(duration);
        }
    }

    pub fn sample_resources(&mut self) {
        if !self.enabled { return; }
        
        self.system.refresh_all();
        
        let memory_used = self.system.used_memory() / BYTES_TO_KB;
        let cpu_usage: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / 
            self.system.cpus().len() as f32;
        
        if let Ok(mut samples) = self.resource_samples.lock() {
            samples.push((memory_used, cpu_usage));
        }
    }

    pub fn start_resource_monitoring(&self) -> std::sync::mpsc::Sender<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let samples = self.resource_samples.clone();
        let enabled = self.enabled;

        std::thread::spawn(move || {
            if !enabled { return; }
            let mut system = System::new();
            
            while rx.try_recv().is_err() {
                system.refresh_all();
                let memory_used = system.used_memory() / BYTES_TO_KB;
                let cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / 
                    system.cpus().len() as f32;
                
                if let Ok(mut samples) = samples.lock() {
                    samples.push((memory_used, cpu_usage));
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        tx
    }

    pub fn finalize(self) -> Option<TelemetryData> {
        if !self.enabled { return None; }

        let mut final_metrics = self.metrics.lock().ok()?.clone();
        final_metrics.timing.total_duration = Some(self.start_time.elapsed());

        // Calculate resource statistics
        if let Ok(samples) = self.resource_samples.lock() {
            if !samples.is_empty() {
                let memory_stats = samples.iter()
                    .map(|(mem, _)| *mem)
                    .fold((u64::MAX, 0u64, 0u64), |(min, max, sum), val| 
                        (min.min(val), max.max(val), sum + val));

                let cpu_stats = samples.iter()
                    .map(|(_, cpu)| *cpu)
                    .fold((f32::MAX, 0f32, 0f32), |(min, max, sum), val| 
                        (min.min(val), max.max(val), sum + val));

                final_metrics.resources = ResourceMetrics {
                    max_memory_kb: memory_stats.1,
                    min_memory_kb: memory_stats.0,
                    avg_memory_kb: memory_stats.2 / samples.len() as u64,
                    max_cpu_percent: cpu_stats.1,
                    min_cpu_percent: cpu_stats.0,
                    avg_cpu_percent: cpu_stats.2 / samples.len() as f32,
                    samples: samples.len(),
                };
            }
        }

        // Log summary
        info!("Telemetry Summary:");
        info!("Total Duration: {:?}", final_metrics.timing.total_duration.unwrap());
        if let Some(d) = final_metrics.timing.workspace_setup_duration {
            info!("Workspace Setup: {:?}", d);
        }
        if let Some(d) = final_metrics.timing.compilation_duration {
            info!("Compilation: {:?}", d);
        }
        if let Some(d) = final_metrics.timing.proof_generation_duration {
            info!("Proof Generation: {:?}", d);
        }
        info!("Memory Usage - Max: {} KB, Min: {} KB, Avg: {} KB", 
            final_metrics.resources.max_memory_kb,
            final_metrics.resources.min_memory_kb,
            final_metrics.resources.avg_memory_kb);
        info!("CPU Usage - Max: {:.1}%, Min: {:.1}%, Avg: {:.1}%",
            final_metrics.resources.max_cpu_percent,
            final_metrics.resources.min_cpu_percent,
            final_metrics.resources.avg_cpu_percent);

        Some(final_metrics)
    }
} 