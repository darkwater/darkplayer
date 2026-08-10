use egui::Response;

pub trait ResponseExt {
    fn autofocus(self) -> Self;
}

impl ResponseExt for Response {
    fn autofocus(self) -> Self {
        if self.ctx.memory(|m| m.focused().is_none()) {
            self.request_focus();
        }
        self
    }
}
