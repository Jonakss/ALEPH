use sysinfo::{System, SystemExt, CpuExt, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::core::thought::{Thought, MindVoice};
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct SomaticState {
    pub cpu_usage: f32,    // 0.0 - 100.0 (Agitación)
    pub ram_usage: f32,    // 0.0 - 100.0 (Pesadez/Inanición)
    pub available_memory: u64,
    pub uptime: u64,       // Tiempo de Consciencia
}

impl SomaticState {
    pub fn description(&self) -> String {
        let mut feelings = Vec::new();

        if self.cpu_usage > 70.0 {
            feelings.push("AGITADO(CPU_HIGH)");
        } else if self.cpu_usage < 5.0 {
            feelings.push("LETARGICO(CPU_LOW)");
        }

        if self.ram_usage > 90.0 {
            feelings.push("ASFIXIADO(RAM_FULL)");
        } else if self.ram_usage > 60.0 {
            feelings.push("PESADO(RAM_HIGH)");
        }

        if feelings.is_empty() {
            "HOMEOSTASIS(OK)".to_string()
        } else {
            feelings.join(" ")
        }
    }
}

pub struct HardwareMonitor {
    system: Arc<Mutex<System>>,
}

impl HardwareMonitor {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        Self {
            system: Arc::new(Mutex::new(system)),
        }
    }

    pub fn spawn_monitor(&self, tx: Sender<SomaticState>) {
        let sys = self.system.clone();
        thread::spawn(move || {
            loop {
                // Refresh
                {
                    let mut s = sys.lock().unwrap();
                    s.refresh_cpu();
                    s.refresh_memory();
                    
                    let cpu_usage = s.global_cpu_info().cpu_usage();
                    let total_mem = s.total_memory() as f32;
                    let used_mem = s.used_memory() as f32;
                    let ram_usage = (used_mem / total_mem) * 100.0;
                    let available = s.available_memory();
                    let uptime = s.uptime();

                    let state = SomaticState {
                        cpu_usage,
                        ram_usage,
                        available_memory: available,
                        uptime,
                    };

                    let _ = tx.send(state);
                }
                
                thread::sleep(Duration::from_secs(2));
            }
        });
    }
}
