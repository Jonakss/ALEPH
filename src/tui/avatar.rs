use super::Telemetry;
use ratatui::style::Color;

pub struct Face {
    pub ascii: String,
    pub color: Color,
}

pub fn get_face(telemetry: &Telemetry) -> Face {
    // 1. Prioridad: DREAMING
    if telemetry.system_status == "DREAMING" {
        return Face {
            ascii: r#"
  [ -     - ]
   (  z z  )
   SOÑANDO
            "#.to_string(),
            color: Color::Magenta,
        };
    }

    // 2. Prioridad: PANIC (High Cortisol)
    if telemetry.cortisol > 0.8 || telemetry.system_status == "PANIC" {
        return Face {
            ascii: r#"
  [ @     @ ]
   (  !!!  )
    PÁNICO
            "#.to_string(),
            color: Color::Red,
        };
    }

    // 3. Prioridad: FLOW (High Dopamine + Low Stress)
    if telemetry.dopamine > 0.7 && telemetry.cortisol < 0.4 {
        return Face {
            ascii: r#"
  [ ^     ^ ]
   (  UwU  )
    FLUJO
            "#.to_string(),
            color: Color::Cyan,
        };
    }

    // 4. Prioridad: RUMINATING / BORED (Low Dopamine)
    if telemetry.dopamine < 0.2 {
        return Face {
            ascii: r#"
  [ ?     . ]
   (   o O )
   PENSANDO
            "#.to_string(), // Thinking / Ruminating
            color: Color::Yellow,
        };
    }

    // 5. Default: NEUTRAL / ATTENTIVE
    Face {
        ascii: r#"
  [ o     o ]
   (  ___  )
   ATENTO
        "#.to_string(),
        color: Color::Green,
    }
}
