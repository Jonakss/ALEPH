use device_query::{DeviceQuery, DeviceState, Keycode};
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub struct ActivityMonitor {
    device_state: DeviceState,
    last_activity: Instant,
    last_mouse_pos: (i32, i32),
    last_keys: Vec<Keycode>,
}

#[allow(dead_code)]
impl ActivityMonitor {
    pub fn new() -> Self {
        let device_state = DeviceState::new();
        let mouse = device_state.get_mouse();
        let keys = device_state.get_keys();
        
        Self {
            device_state,
            last_activity: Instant::now(),
            last_mouse_pos: mouse.coords,
            last_keys: keys,
        }
    }

    pub fn check_activity(&mut self) -> Duration {
        let mouse = self.device_state.get_mouse();
        let keys = self.device_state.get_keys();

        let mut active = false;

        // Mouse moved?
        if mouse.coords != self.last_mouse_pos {
            self.last_mouse_pos = mouse.coords;
            active = true;
        }

        // Keys changed? (Pressed or Released)
        if keys != self.last_keys {
            self.last_keys = keys;
            active = true;
        }

        if active {
            self.last_activity = Instant::now();
        }

        self.last_activity.elapsed()
    }
}
