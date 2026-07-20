use core::time::Duration;

use egui::{Key, Modifiers, emath::easing};
use egui_material_icons::icons;

use crate::{
    AppState,
    message::Message,
    ui::{
        pages::Page,
        utils::AnimTimer,
        widgets::seek_bar::{SeekBar, SeekBarPosition, SeekBarState},
    },
};

#[derive(Default)]
pub struct PlayerPage {
    state: State,
    showed_bar_at: AnimTimer,
}

#[derive(Clone, Copy)]
enum State {
    At(SeekBarState),
    Transitioning {
        started: AnimTimer,
        from: SeekBarState,
        to: SeekBarState,
    },
}

impl Default for State {
    fn default() -> Self {
        State::At(SeekBarState::Offscreen)
    }
}

impl PlayerPage {
    fn transition_duration(&self) -> Duration {
        Duration::from_millis(300)
    }

    fn focus(&self) -> SeekBarState {
        match self.state {
            State::At(focus) => focus,
            State::Transitioning { to, .. } => to,
        }
    }

    fn position(&mut self) -> SeekBarPosition {
        match self.state {
            State::At(focus) => SeekBarPosition::Still(focus),
            State::Transitioning { started, from, to } => {
                let t = started.get_t(self.transition_duration(), easing::quadratic_in_out);
                if t >= 1. {
                    self.state = State::At(to);
                    self.showed_bar_at.start();
                    SeekBarPosition::Still(to)
                } else {
                    SeekBarPosition::Transitioning { from, to, t }
                }
            }
        }
    }

    fn opacity(&mut self) -> f32 {
        match self.state {
            State::At(SeekBarState::Offscreen) => 0.,
            State::At(SeekBarState::SeekBar) => {
                let t = self
                    .showed_bar_at
                    .get_t(Duration::from_secs(2), easing::exponential_in);

                if t >= 1. {
                    self.state = State::At(SeekBarState::Offscreen);
                    0.
                } else {
                    1. - t
                }
            }
            State::Transitioning {
                started,
                from: SeekBarState::Offscreen,
                ..
            } => started.get_t(self.transition_duration(), easing::linear),
            State::Transitioning {
                started, to: SeekBarState::Offscreen, ..
            } => 1. - started.get_t(self.transition_duration(), easing::linear),
            _ => 1.,
        }
    }
}

enum Command {
    Transition(SeekBarState, SeekBarState),
    SendMessage(fn() -> Message),
    SendMessageShowBar(fn() -> Message),
}

impl Page for PlayerPage {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui) {
        let focus = self.focus();
        let cmds = match focus {
            SeekBarState::Offscreen => &[
                (
                    Key::Q,
                    Command::SendMessage(|| {
                        Message::MpvCommand("cycle-values".to_owned(), vec![
                            "tscale".to_owned(),
                            "jinc".to_owned(),
                            "sphinx".to_owned(),
                            "blackman".to_owned(),
                            "kaiser".to_owned(),
                            "welch".to_owned(),
                            "quadric".to_owned(),
                            "hamming".to_owned(),
                            "tukey".to_owned(),
                            "hanning".to_owned(),
                            "cosine".to_owned(),
                            "bartlett".to_owned(),
                            "gaussian".to_owned(),
                            "triangle".to_owned(),
                            "nearest".to_owned(),
                            "box".to_owned(),
                            "robidouxsharp".to_owned(),
                            "robidoux".to_owned(),
                            "mitchell".to_owned(),
                            "catmull_rom".to_owned(),
                            "hermite".to_owned(),
                            "bicubic".to_owned(),
                            "ginseng".to_owned(),
                            "lanczos".to_owned(),
                            "sinc".to_owned(),
                            "spline64".to_owned(),
                            "spline36".to_owned(),
                            "spline16".to_owned(),
                            "linear".to_owned(),
                            "oversample".to_owned(),
                        ])
                    }),
                ),
                (
                    Key::W,
                    Command::SendMessage(|| {
                        Message::MpvCommand("cycle-values".to_owned(), vec![
                            "tscale".to_owned(),
                            "oversample".to_owned(),
                            "linear".to_owned(),
                            "spline16".to_owned(),
                            "spline36".to_owned(),
                            "spline64".to_owned(),
                            "sinc".to_owned(),
                            "lanczos".to_owned(),
                            "ginseng".to_owned(),
                            "bicubic".to_owned(),
                            "hermite".to_owned(),
                            "catmull_rom".to_owned(),
                            "mitchell".to_owned(),
                            "robidoux".to_owned(),
                            "robidouxsharp".to_owned(),
                            "box".to_owned(),
                            "nearest".to_owned(),
                            "triangle".to_owned(),
                            "gaussian".to_owned(),
                            "bartlett".to_owned(),
                            "cosine".to_owned(),
                            "hanning".to_owned(),
                            "tukey".to_owned(),
                            "hamming".to_owned(),
                            "quadric".to_owned(),
                            "welch".to_owned(),
                            "kaiser".to_owned(),
                            "blackman".to_owned(),
                            "sphinx".to_owned(),
                            "jinc".to_owned(),
                        ])
                    }),
                ),
                (
                    Key::E,
                    Command::SendMessage(|| {
                        Message::MpvCommand("cycle".to_owned(), vec!["interpolation".to_owned()])
                    }),
                ),
                (
                    Key::R,
                    Command::SendMessage(|| {
                        Message::MpvCommand("cycle-values".to_owned(), vec![
                            "video-sync".to_owned(),
                            "audio".to_owned(),
                            "display-resample".to_owned(),
                        ])
                    }),
                ),
                (Key::ArrowLeft, Command::SendMessageShowBar(|| Message::SeekBackward)),
                (Key::ArrowRight, Command::SendMessageShowBar(|| Message::SeekForward)),
                (Key::ArrowDown, Command::Transition(focus, SeekBarState::Menu)),
                (Key::Enter, Command::SendMessage(|| Message::TogglePause)),
            ][..],
            SeekBarState::SeekBar => &[
                (Key::ArrowUp, Command::Transition(focus, SeekBarState::Offscreen)),
                (Key::ArrowLeft, Command::SendMessageShowBar(|| Message::SeekBackward)),
                (Key::ArrowRight, Command::SendMessageShowBar(|| Message::SeekForward)),
                (Key::ArrowDown, Command::Transition(focus, SeekBarState::Menu)),
                (Key::Escape, Command::Transition(focus, SeekBarState::Offscreen)),
            ],
            SeekBarState::Menu => &[
                (Key::ArrowUp, Command::Transition(focus, SeekBarState::SeekBar)),
                (Key::Escape, Command::Transition(focus, SeekBarState::Offscreen)),
            ],
        };

        for (key, cmd) in cmds {
            // TODO: optimize
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, *key)) {
                match cmd {
                    Command::SendMessage(msg_fn) => msg_fn().send(),
                    Command::SendMessageShowBar(msg_fn) => {
                        msg_fn().send();
                        self.state = State::At(SeekBarState::SeekBar);
                        self.showed_bar_at.start();
                    }
                    Command::Transition(from, to) => {
                        self.state = State::Transitioning {
                            from: *from,
                            to: *to,
                            started: AnimTimer::now(),
                        };
                    }
                }
            }
        }

        SeekBar {
            time_pos: app.properties.time_pos,
            duration: app.properties.duration,

            menu: |ui| {
                ui.spacing_mut().button_padding = egui::vec2(20., 10.);
                ui.spacing_mut().item_spacing = egui::vec2(30., 0.);

                ui.button("foo");
                ui.button("bar");
                ui.button(icons::ICON_CAMERA_ALT.rich_text().size(32.));
            },
        }
        .ui(ui, self.opacity(), self.position());

        if let SeekBarPosition::Transitioning { .. } = self.position() {
            ui.request_repaint();
        }

        // match self.state {
        //     State::At(Focus::None) => {}
        //     State::At(Focus::SeekBar) => seekbar.ui(ui),
        //     State::At(Focus::Menu) => seekbar.ui(ui),
        //
        //     State::Transitioning { started, from, to } => todo!(),
        // }
    }
}
