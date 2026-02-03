use sysinfo::{CpuRefreshKind, RefreshKind, System};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::Sender;

pub struct BodyStatus {
    pub cpu_usage: f32, // 0.0 - 100.0 (Global load)
    pub ram_usage: f32, // Used / Total ratio (0.0 - 1.0)
}

pub fn spawn_monitor(tx: Sender<BodyStatus>) {
    thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything())
        );
        
        // Esperar un poco para la primera lectura de CPU (sysinfo requiere 2 lecturas)
        thread::sleep(Duration::from_secs(1));

        loop {
            // Refrescar Métricas
            sys.refresh_cpu();
            sys.refresh_memory();

            let load = sys.global_cpu_info().cpu_usage();
            let ram = sys.used_memory() as f32 / sys.total_memory() as f32;

            let status = BodyStatus {
                cpu_usage: load,
                ram_usage: ram,
            };

            // Enviar (Non-blocking drop if channel full)
            let _ = tx.send(status);

            // Ritmo metabólico (Más lento que el audio, 1Hz)
            thread::sleep(Duration::from_secs(1));
        }
    });
}
