use crate::AppState;

pub mod player;

pub trait Page {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui);
}
