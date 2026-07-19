use egui::{Stroke, vec2};

use crate::ui::widgets::Fadeable;

pub struct SeekBar {
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,
}

impl Fadeable for SeekBar {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let radius = ui.content_rect().width().min(ui.content_rect().height()) * 0.1;

        let fg_stroke = Stroke::new(radius * 0.04, egui::Color32::WHITE);
        let shadow_stroke = Stroke::new(radius * 0.04 + 2., egui::Color32::from_black_alpha(127));

        let padded = ui.content_rect().shrink(radius * 0.8);

        ui.painter().line_segment(
            [padded.left_bottom() - vec2(1., 0.), padded.right_bottom() + vec2(1., 0.)],
            shadow_stroke,
        );

        if let (Some(time_pos), Some(duration)) = (self.time_pos, self.duration) {
            let fraction = (time_pos / duration).clamp(0., 1.) as f32;

            ui.painter().line_segment(
                [padded.left_bottom(), padded.left_bottom().lerp(padded.right_bottom(), fraction)],
                fg_stroke,
            );
        }
    }

    fn logic(self, _ui: &mut egui::Ui, _timer: Option<&mut super::FadeTimer>) {}
}
