#[derive(Default)]
pub struct Properties {
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum PropertyChange {
    TimePos(Option<f64>),
    Duration(Option<f64>),
}

impl Properties {
    pub fn apply(&mut self, change: PropertyChange) {
        match change {
            PropertyChange::TimePos(value) => self.time_pos = value,
            PropertyChange::Duration(value) => self.duration = value,
        }
    }
}
