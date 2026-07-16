use crate::AppState;

pub mod empty;

pub trait Page {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui);
}
