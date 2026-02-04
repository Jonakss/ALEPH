use crate::tui::Telemetry;
use ratatui::style::Color;

pub struct Face {
    pub ascii: String,
    pub color: Color,
}

pub fn get_face(telemetry: &Telemetry) -> Face {
    // 1. Prioridad: DREAMING
    let status = telemetry.system_status.to_uppercase();
    if status.contains("DREAMING") || status.contains("SOÑANDO") {
        return Face {
            ascii: r#"  [ -     - ]
   (  z z  )
   SOÑANDO"#.to_string(),
            color: Color::Magenta,
        };
    }

    // 2. Prioridad: PANIC (High Cortisol)
    if telemetry.cortisol > 0.8 || status.contains("PANIC") || status.contains("PÁNICO") || status.contains("COLAPSO") {
        return Face {
            ascii: r#"  [ @     @ ]
   (  !!!  )
    PÁNICO"#.to_string(),
            color: Color::Red,
        };
    }

    // 3. Prioridad: FLOW (High Dopamine + Low Stress)
    if (telemetry.dopamine > 0.7 && telemetry.cortisol < 0.4) || status.contains("FLOW") || status.contains("FLUJO") || status.contains("CREATIVO") {
        return Face {
            ascii: r#"  [ ^     ^ ]
   (  UwU  )
    FLUJO"#.to_string(),
            color: Color::Cyan,
        };
    }

    // 4. Prioridad: RUMINATING / BORED (Low Dopamine)
    if telemetry.dopamine < 0.2 || status.contains("RUMINAT") || status.contains("PENSANDO") || status.contains("ABURRIDO") {
        return Face {
            ascii: r#"  [ ?     . ]
   (   o O )
   PENSANDO"#.to_string(), // Thinking / Ruminating
            color: Color::Yellow,
        };
    }

    // 5. Default: NEUTRAL / ATTENTIVE
    Face {
        ascii: r#"  [ o     o ]
   (  ___  )
   ATENTO"#.to_string(),
        color: Color::Green,
    }
}
