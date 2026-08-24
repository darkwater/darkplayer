use egui::{
    Align, Align2, Color32, Direction, FontId, Layout, Pos2, Rect, Stroke, UiBuilder,
    emath::GuiRounding as _, pos2, vec2,
};
use egui_material_icons::icons;

use crate::ui::painter::PainterExt;

pub struct SeekBar {
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,

    pub menu: fn(&mut egui::Ui),
    pub rmenu: fn(&mut egui::Ui),
}

const PADDING: f32 = 60.;
const TEXT_PADDING: f32 = 12.;
const MENU_HEIGHT: f32 = 60.;
const OFFSCREEN_OFFSET: f32 = 20.;

#[derive(Clone, Copy)]
struct PositionsValues {
    text_bottom: f32,
    bar: f32,
    down_icon: f32,
    menu_bottom: f32,
}

#[derive(Clone, Copy)]
pub enum SeekBarState {
    Offscreen,
    SeekBar,
    Menu,
}

pub enum SeekBarPosition {
    Still(SeekBarState),
    Transitioning {
        from: SeekBarState,
        to: SeekBarState,
        t: f32,
    },
}

impl SeekBarState {
    fn values(self, ui: &egui::Ui) -> PositionsValues {
        let bottom = ui.viewport_rect().bottom();
        let offscreen = bottom + OFFSCREEN_OFFSET + MENU_HEIGHT;
        let first_height = bottom - PADDING;
        let second_height = first_height - MENU_HEIGHT - PADDING / 2.;

        match self {
            Self::Offscreen => PositionsValues {
                text_bottom: offscreen,
                bar: offscreen,
                down_icon: offscreen,
                menu_bottom: offscreen,
            },
            Self::SeekBar => PositionsValues {
                text_bottom: first_height - TEXT_PADDING,
                bar: first_height,
                down_icon: (first_height + bottom) / 2.,
                menu_bottom: offscreen,
            },
            Self::Menu => PositionsValues {
                text_bottom: second_height - TEXT_PADDING,
                bar: second_height,
                down_icon: (second_height + bottom) / 2.,
                menu_bottom: first_height,
            },
        }
    }
}

impl PositionsValues {
    fn show_menu(&self, ui: &egui::Ui) -> bool {
        self.menu_bottom - MENU_HEIGHT < ui.viewport_rect().bottom()
    }

    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            text_bottom: self.text_bottom + (other.text_bottom - self.text_bottom) * t,
            bar: self.bar + (other.bar - self.bar) * t,
            down_icon: self.down_icon + (other.down_icon - self.down_icon) * t,
            menu_bottom: self.menu_bottom + (other.menu_bottom - self.menu_bottom) * t,
        }
    }

    fn bar_left(&self, ui: &egui::Ui) -> Pos2 {
        Pos2::new(ui.content_rect().left() + PADDING, self.bar)
    }

    fn bar_right(&self, ui: &egui::Ui) -> Pos2 {
        Pos2::new(ui.content_rect().right() - PADDING, self.bar)
    }

    fn text_left(&self, ui: &egui::Ui) -> Pos2 {
        Pos2::new(ui.content_rect().left() + PADDING, self.text_bottom)
    }

    fn text_right(&self, ui: &egui::Ui) -> Pos2 {
        Pos2::new(ui.content_rect().right() - PADDING, self.text_bottom)
    }

    fn down_icon(&self, ui: &egui::Ui) -> Pos2 {
        pos2(ui.content_rect().center().x, self.down_icon)
    }

    fn menu_rect(&self, ui: &egui::Ui) -> Rect {
        Rect::from_min_max(
            pos2(self.bar_left(ui).x, self.menu_bottom - MENU_HEIGHT),
            pos2(self.bar_right(ui).x, self.menu_bottom),
        )
    }
}

impl SeekBar {
    fn fg_color(&self) -> Color32 {
        Color32::WHITE
    }

    fn background(&self, ui: &mut egui::Ui) {
        ui.painter().gradient(
            Direction::TopDown,
            ui.content_rect().split_top_bottom_at_fraction(0.5).1,
            Color32::from_black_alpha(0),
            &[
                (0.1, Color32::from_black_alpha(10)),
                (0.2, Color32::from_black_alpha(32)),
                (0.7, Color32::from_black_alpha(96)),
            ],
            Color32::from_black_alpha(192),
        );
    }

    fn seek_line(&self, ui: &mut egui::Ui, positions: PositionsValues) {
        let fg_stroke = Stroke::new(4.0.round_to_pixels(ui.pixels_per_point()), self.fg_color());
        let shadow_stroke = Stroke::new(fg_stroke.width + 4., ui.painter().shadow_color());

        let bar_left = positions.bar_left(ui);
        let bar_right = positions.bar_right(ui);

        ui.painter()
            .line_segment([bar_left - vec2(1., 0.), bar_right + vec2(1., 0.)], shadow_stroke);

        if let (Some(time_pos), Some(duration)) = (self.time_pos, self.duration) {
            let fraction = (time_pos / duration).clamp(0., 1.) as f32;

            ui.painter().line_segment(
                [
                    bar_left,
                    (bar_left + vec2(fg_stroke.width, 0.)).max(bar_left.lerp(bar_right, fraction)),
                ],
                fg_stroke,
            );
        }
    }

    fn position_text(&self, ui: &mut egui::Ui, positions: PositionsValues) {
        if let Some(time_pos) = self.time_pos {
            ui.painter().shadow_text(
                positions.text_left(ui),
                Align2::LEFT_BOTTOM,
                TimeFormat::MinutesSeconds.format(time_pos),
                FontId::proportional(26.),
                self.fg_color(),
            );
        }

        if let Some(duration) = self.duration {
            ui.painter().shadow_text(
                positions.text_right(ui),
                Align2::RIGHT_BOTTOM,
                TimeFormat::MinutesSeconds.format(duration),
                FontId::proportional(26.),
                self.fg_color(),
            );
        }
    }

    fn down_icon(&self, ui: &mut egui::Ui, positions: PositionsValues, opacity: f32) {
        ui.painter().shadow_icon(
            positions.down_icon(ui),
            icons::ICON_KEYBOARD_DOUBLE_ARROW_DOWN,
            24.,
            Color32::from_white_alpha((opacity * 255.) as u8),
        );
    }

    pub fn ui(self, ui: &mut egui::Ui, opacity: f32, position: SeekBarPosition) {
        let positions = match position {
            SeekBarPosition::Still(pos) => pos.values(ui),
            SeekBarPosition::Transitioning { from, to, t } => {
                from.values(ui).lerp(to.values(ui), t)
            }
        };

        let down_opacity = match position {
            SeekBarPosition::Still(SeekBarState::Menu) => 0.,
            SeekBarPosition::Still(_) => 1.,
            SeekBarPosition::Transitioning {
                from: SeekBarState::SeekBar,
                to: SeekBarState::Menu,
                t,
            } => 1. - t,
            SeekBarPosition::Transitioning {
                from: SeekBarState::Menu,
                to: SeekBarState::SeekBar,
                t,
            } => t,
            SeekBarPosition::Transitioning { from: SeekBarState::Menu, .. }
            | SeekBarPosition::Transitioning { to: SeekBarState::Menu, .. } => 0.,
            _ => 1.,
        };

        ui.scope(|ui| {
            ui.set_opacity(opacity);
            self.background(ui);
            self.seek_line(ui, positions);
            self.position_text(ui, positions);
            self.down_icon(ui, positions, down_opacity);

            if positions.show_menu(ui) {
                let mut lui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(positions.menu_rect(ui))
                        .layout(Layout::left_to_right(Align::Center).with_cross_justify(true)),
                );
                lui.spacing_mut().button_padding = egui::vec2(20., 10.);
                lui.spacing_mut().item_spacing = egui::vec2(30., 0.);

                (self.menu)(&mut lui);

                let mut rui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(positions.menu_rect(ui))
                        .layout(Layout::right_to_left(Align::Center).with_cross_justify(true)),
                );
                rui.spacing_mut().button_padding = egui::vec2(20., 10.);
                rui.spacing_mut().item_spacing = egui::vec2(30., 0.);

                (self.rmenu)(&mut rui);
            }
        });
    }
}

pub enum TimeFormat {
    /// MM:SS
    MinutesSeconds,
}

impl TimeFormat {
    pub fn format(&self, secs: f64) -> String {
        match self {
            TimeFormat::MinutesSeconds => {
                let minutes = (secs / 60.).floor() as u64;
                let seconds = (secs % 60.).floor() as u64;
                format!("{minutes}:{seconds:02}")
            }
        }
    }
}
