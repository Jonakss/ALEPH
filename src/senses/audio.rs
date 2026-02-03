use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

/// Inicia la escucha del micrófono y envía el "Estímulo Neural" (0.0 - 1.0)
/// a través del canal.
/// Retorna el Stream de audio para que no se destruya (drop) mientras corre el programa.
pub fn start_listening(tx: Sender<f32>) -> Result<cpal::Stream, anyhow::Error> {
    let host = cpal::default_host();
    
    // 1. Buscar el microfono por defecto
    let device = host.default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
    
    // println!("👂 Oído Conectado: {}", device.name().unwrap_or("Unknown".to_string()));

    // 2. Configuración del stream
    let config = device.default_input_config()?;
    
    // Clonamos el sender para moverlo al thread de audio
    let tx = Arc::new(Mutex::new(tx));

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), tx)?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), tx)?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), tx)?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    Ok(stream)
}

fn run<T>(
    device: &cpal::Device, 
    config: &cpal::StreamConfig, 
    tx: Arc<Mutex<Sender<f32>>>
) -> Result<cpal::Stream, anyhow::Error> 
where
    T: cpal::Sample + cpal::SizedSample, 
    f32: From<T>,
{
    let err_fn = |err| {
        // eprintln!("❌ Error en el oído: {}", err);
    };

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &_| {
            process_audio_data(data, &tx);
        },
        err_fn,
        None
    )?;

    stream.play()?;
    Ok(stream)
}

/// Procesa el buffer de audio crudo y extrae la "Energía" (RMS)
/// Aplicando una transferencia no-lineal (Bio-mimesis)
fn process_audio_data<T>(data: &[T], tx: &Arc<Mutex<Sender<f32>>>)
where
    T: cpal::Sample,
    f32: From<T>,
{
    if data.is_empty() { return; }

    // 1. Calcular RMS (Root Mean Square) del buffer actual
    let mut sum_squares = 0.0;
    for &sample in data {
        let sample_f32: f32 = f32::from(sample);
        sum_squares += sample_f32 * sample_f32;
    }
    let rms = (sum_squares / data.len() as f32).sqrt();

    // 2. Transferencia No-Lineal (Logarítmica / Sigmoide)
    // El oído humano no es lineal. Un susurro es 0.01, un grito es 1.0.
    // Usamos una función de saturación suave.
    // Factor de ganancia: Ajustar según sensibilidad del micro.
    let sensitivity = 5.0; 
    let stimulus = (rms * sensitivity).tanh(); // tanh mapea 0->0, high->1 suavemente

    // 3. Enviar al Cerebro (Reservoir)
    // Ignoramos errores de envío (si el canal se cierra, es que el programa terminó)
    if let Ok(sender) = tx.lock() {
        let _ = sender.send(stimulus);
    }
}
