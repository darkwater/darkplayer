use crate::AppState;

pub mod library;
pub mod player;

pub trait Page: Send {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui);
}
