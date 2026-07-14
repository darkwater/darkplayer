use egui::{vec2, Stroke};

#[derive(Default)]
pub struct Dpad {}

impl Dpad {
    pub fn render(&self, ui: &mut egui::Ui) {
        let radius = ui.content_rect().width().min(ui.content_rect().height()) * 0.1;
        let center = ui.content_rect().right_center() + vec2(radius * -1.5, 0.0);

        let stroke = Stroke::new(radius * 0.04, egui::Color32::WHITE);
        let shadow_stroke = Stroke::new(radius * 0.04 + 2., egui::Color32::from_black_alpha(127));

        ui.painter()
            .circle_stroke(center, radius - 1., shadow_stroke);

        ui.painter().circle_stroke(center, radius, stroke);
    }
}
