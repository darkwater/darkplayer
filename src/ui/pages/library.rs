use super::Page;
use crate::AppState;

#[derive(Default)]
pub struct LibraryPage {}

impl Page for LibraryPage {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui) {
        todo!()
    }
}
