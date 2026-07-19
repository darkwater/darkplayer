use core::time::Duration;
use std::time::Instant;

use egui::emath::easing;

pub mod dpad;
pub mod seek_bar;

pub trait Fadeable {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn logic(self, ui: &mut egui::Ui, timer: Option<&mut FadeTimer>);
}

pub struct Faded<'a, T: Fadeable> {
    pub inner: T,
    pub fade_timer: Option<&'a mut FadeTimer>,
    pub period: Duration,
}

#[derive(Default, Clone)]
pub struct FadeTimer {
    last_input: Option<Instant>,
}
impl FadeTimer {
    pub fn reset(&mut self) {
        self.last_input = Some(Instant::now());
    }
}

impl<'a, T: Fadeable> Faded<'a, T> {
    pub fn new(inner: T, fade_timer: &'a mut FadeTimer, period: Duration) -> Self {
        Self {
            inner,
            fade_timer: Some(fade_timer),
            period,
        }
    }

    pub fn permanent(inner: T) -> Self {
        Self {
            inner,
            fade_timer: None,
            period: Duration::from_secs(0),
        }
    }

    pub fn render(self, ui: &mut egui::Ui) {
        let Self { mut inner, fade_timer, period } = self;

        if let Some(timer) = &fade_timer {
            if let Some(last_input) = timer.last_input {
                let elapsed = last_input.elapsed();
                if elapsed >= period {
                    // hidden
                } else {
                    // shown

                    let opacity =
                        easing::cubic_out(1.0 - (elapsed.as_secs_f32() / period.as_secs_f32()));

                    ui.request_repaint();
                    ui.scope(|ui| {
                        ui.set_opacity(opacity);
                        inner.ui(ui);
                    });
                }
            } else {
                // hidden
            }
        } else {
            // shown
            inner.ui(ui);
        };

        inner.logic(ui, fade_timer);
    }
}
