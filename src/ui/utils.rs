use core::time::Duration;
use std::time::Instant;

#[derive(Default, Clone, Copy)]
pub struct AnimTimer {
    start: Option<Instant>,
}

impl AnimTimer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn now() -> Self {
        let mut timer = Self::new();
        timer.start();
        timer
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn get_t(&self, period: Duration, easing: fn(f32) -> f32) -> f32 {
        if let Some(start) = self.start {
            let elapsed = start.elapsed();
            if elapsed >= period {
                1.0
            } else {
                easing(elapsed.as_secs_f32() / period.as_secs_f32())
            }
        } else {
            0.0
        }
    }
}
