use log::info;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, System, SystemExt};
use toml::Value;

const BYTES_TO_KB: u64 = 1024;

#[derive(Default, Serialize, Clone)]
pub struct CargoMetadata {
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Default, Serialize, Clone)]
pub struct ZkMetrics {
    pub cycles: Option<u64>,                 // Number of VM cycles executed
    pub num_segments: Option<usize>,         // Number of segments/shards
    pub core_proof_size: Option<usize>,      // Size of the core proof in bytes
    pub recursive_proof_size: Option<usize>, // Size of the recursive/compressed proof in bytes
    pub execution_speed: Option<f64>,        // Cycles per second during proof generation
}

#[derive(Default, Serialize, Clone)]
pub struct TimingMetrics {
    pub workspace_setup_duration: Option<Duration>,
    pub compilation_duration: Option<Duration>,
    pub proof_generation_duration: Option<Duration>,
    pub core_prove_duration: Option<Duration>, // Time to generate initial proof
    pub core_verify_duration: Option<Duration>, // Time to verify initial proof
    pub compress_prove_duration: Option<Duration>, // Time to generate compressed/recursive proof
    pub compress_verify_duration: Option<Duration>, // Time to verify compressed/recursive proof
    pub total_duration: Option<Duration>,
}

#[derive(Default, Serialize, Clone)]
pub struct ProgramInfo {
    pub file_path: String,
    pub file_name: String,
    pub absolute_path: Option<String>,
    pub cargo_metadata: CargoMetadata,
}

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
pub struct TelemetryData {
    pub timing: TimingMetrics,
    pub resources: ResourceMetrics,
    pub proving_system: String,
    pub precompiles_enabled: bool,
    pub program: ProgramInfo,
    pub zk_metrics: ZkMetrics,
}

pub struct TelemetryCollector {
    start_time: Instant,
    system: System,
    metrics: Arc<Mutex<TelemetryData>>,
    enabled: bool,
    resource_samples: Arc<Mutex<Vec<(u64, f32)>>>, // (memory_kb, cpu_percent)
}

impl TelemetryCollector {
    pub fn new(
        proving_system: &str,
        precompiles_enabled: bool,
        enabled: bool,
        guest_path: &str,
    ) -> Self {
        let mut system = System::new();
        system.refresh_all();

        let path = PathBuf::from(guest_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let cargo_metadata = Self::extract_cargo_metadata(guest_path);

        let metrics = TelemetryData {
            proving_system: proving_system.to_string(),
            precompiles_enabled,
            program: ProgramInfo {
                file_path: guest_path.to_string(),
                file_name,
                absolute_path: std::fs::canonicalize(guest_path)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                cargo_metadata,
            },
            ..Default::default()
        };

        Self {
            start_time: Instant::now(),
            system,
            metrics: Arc::new(Mutex::new(metrics)),
            enabled,
            resource_samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn extract_cargo_metadata(guest_path: &str) -> CargoMetadata {
        let cargo_path = PathBuf::from(guest_path).join("Cargo.toml");

        if let Ok(contents) = fs::read_to_string(cargo_path) {
            if let Ok(cargo_toml) = contents.parse::<Value>() {
                let mut metadata = CargoMetadata::default();

                if let Some(package) = cargo_toml.get("package") {
                    if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                        metadata.package_name = Some(name.to_string());
                    }
                    if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                        metadata.version = Some(version.to_string());
                    }
                    if let Some(authors) = package.get("authors").and_then(|v| v.as_array()) {
                        metadata.authors = Some(
                            authors
                                .iter()
                                .filter_map(|a| a.as_str())
                                .map(String::from)
                                .collect(),
                        );
                    }
                    if let Some(edition) = package.get("edition").and_then(|v| v.as_str()) {
                        metadata.edition = Some(edition.to_string());
                    }
                }

                if let Some(deps) = cargo_toml.get("dependencies") {
                    if let Some(table) = deps.as_table() {
                        metadata.dependencies = Some(table.keys().map(|k| k.to_string()).collect());
                    }
                }

                return metadata;
            }
        }
        CargoMetadata::default()
    }

    pub fn record_workspace_setup(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.workspace_setup_duration = Some(duration);
        }
    }

    pub fn record_compilation(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.compilation_duration = Some(duration);
        }
    }

    pub fn record_proof_generation(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.proof_generation_duration = Some(duration);
        }
    }

    pub fn sample_resources(&mut self) {
        if !self.enabled {
            return;
        }

        self.system.refresh_all();

        let memory_used = self.system.used_memory() / BYTES_TO_KB;
        let cpu_usage: f32 = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.system.cpus().len() as f32;

        if let Ok(mut samples) = self.resource_samples.lock() {
            samples.push((memory_used, cpu_usage));
        }
    }

    pub fn start_resource_monitoring(&self) -> std::sync::mpsc::Sender<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let samples = self.resource_samples.clone();
        let enabled = self.enabled;

        std::thread::spawn(move || {
            if !enabled {
                return;
            }
            let mut system = System::new();

            while rx.try_recv().is_err() {
                system.refresh_all();
                let memory_used = system.used_memory() / BYTES_TO_KB;
                let cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
                    / system.cpus().len() as f32;

                if let Ok(mut samples) = samples.lock() {
                    samples.push((memory_used, cpu_usage));
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        tx
    }

    pub fn record_zk_metrics(
        &self,
        cycles: Option<u64>,
        num_segments: Option<usize>,
        core_proof_size: Option<usize>,
        recursive_proof_size: Option<usize>,
    ) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            let proof_duration = metrics
                .timing
                .proof_generation_duration
                .unwrap_or(Duration::from_secs(0));
            let execution_speed = cycles.map(|c| c as f64 / proof_duration.as_secs_f64());

            metrics.zk_metrics = ZkMetrics {
                cycles,
                num_segments,
                core_proof_size,
                recursive_proof_size,
                execution_speed,
            };
        }
    }

    pub fn record_proof_timings(
        &self,
        core_prove: Duration,
        core_verify: Duration,
        compress_prove: Option<Duration>,
        compress_verify: Option<Duration>,
    ) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.core_prove_duration = Some(core_prove);
            metrics.timing.core_verify_duration = Some(core_verify);
            metrics.timing.compress_prove_duration = compress_prove;
            metrics.timing.compress_verify_duration = compress_verify;
        }
    }

    pub fn finalize(self) -> Option<TelemetryData> {
        if !self.enabled {
            return None;
        }

        let mut final_metrics = self.metrics.lock().ok()?.clone();
        final_metrics.timing.total_duration = Some(self.start_time.elapsed());

        // Calculate resource statistics
        if let Ok(samples) = self.resource_samples.lock() {
            if !samples.is_empty() {
                let memory_stats = samples
                    .iter()
                    .map(|(mem, _)| *mem)
                    .fold((u64::MAX, 0u64, 0u64), |(min, max, sum), val| {
                        (min.min(val), max.max(val), sum + val)
                    });

                let cpu_stats = samples
                    .iter()
                    .map(|(_, cpu)| *cpu)
                    .fold((f32::MAX, 0f32, 0f32), |(min, max, sum), val| {
                        (min.min(val), max.max(val), sum + val)
                    });

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
        info!(
            "Program: {} ({})",
            final_metrics.program.file_name, final_metrics.program.file_path
        );
        if let Some(abs_path) = &final_metrics.program.absolute_path {
            info!("Absolute Path: {}", abs_path);
        }

        // Log Cargo metadata
        let metadata = &final_metrics.program.cargo_metadata;
        if let Some(name) = &metadata.package_name {
            info!("Package Name: {}", name);
            if let Some(version) = &metadata.version {
                info!("Version: {}", version);
            }
        }
        if let Some(authors) = &metadata.authors {
            info!("Authors: {}", authors.join(", "));
        }
        if let Some(edition) = &metadata.edition {
            info!("Rust Edition: {}", edition);
        }
        if let Some(deps) = &metadata.dependencies {
            info!("Dependencies: {}", deps.join(", "));
        }

        // Log ZK metrics
        let zk = &final_metrics.zk_metrics;
        if let Some(cycles) = zk.cycles {
            info!("VM Cycles: {}", cycles);
            if let Some(speed) = zk.execution_speed {
                info!("Execution Speed: {:.2} cycles/second", speed);
            }
        }
        if let Some(segments) = zk.num_segments {
            info!("Number of Segments/Shards: {}", segments);
        }
        if let Some(size) = zk.core_proof_size {
            info!("Core Proof Size: {} bytes", size);
        }
        if let Some(size) = zk.recursive_proof_size {
            info!("Recursive Proof Size: {} bytes", size);
        }

        // Log timings
        let timing = &final_metrics.timing;
        info!("Total Duration: {:?}", timing.total_duration.unwrap());
        if let Some(d) = timing.workspace_setup_duration {
            info!("Workspace Setup: {:?}", d);
        }
        if let Some(d) = timing.compilation_duration {
            info!("Compilation: {:?}", d);
        }
        if let Some(d) = timing.proof_generation_duration {
            info!("Total Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.core_prove_duration {
            info!("Core Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.core_verify_duration {
            info!("Core Proof Verification: {:?}", d);
        }
        if let Some(d) = timing.compress_prove_duration {
            info!("Recursive Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.compress_verify_duration {
            info!("Recursive Proof Verification: {:?}", d);
        }

        // Log resource usage
        info!(
            "Memory Usage - Max: {} KB, Min: {} KB, Avg: {} KB",
            final_metrics.resources.max_memory_kb,
            final_metrics.resources.min_memory_kb,
            final_metrics.resources.avg_memory_kb
        );
        info!(
            "CPU Usage - Max: {:.1}%, Min: {:.1}%, Avg: {:.1}%",
            final_metrics.resources.max_cpu_percent,
            final_metrics.resources.min_cpu_percent,
            final_metrics.resources.avg_cpu_percent
        );

        Some(final_metrics)
    }
}
