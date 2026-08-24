use core::time::Duration;

use egui::{Key, Modifiers, emath::easing};

use super::{Page, library::LibraryPage};
use crate::{
    AppState,
    message::Message,
    ui::{
        utils::AnimTimer,
        widgets::seek_bar::{SeekBar, SeekBarPosition, SeekBarState},
    },
    utils::ResponseExt,
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
        for (path, hash) in &app.db.index {
            ui.label(format!("{}: {}", path.display(), hash.0));
        }

        let focus = self.focus();
        let cmds = match focus {
            SeekBarState::Offscreen => &[
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
                (Key::Enter, Command::SendMessage(|| Message::TogglePause)),
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
                ui.button("foo").autofocus();
                // ui.button("bar");
                // ui.button(icons::ICON_CAMERA_ALT.rich_text().size(32.));
            },

            rmenu: |ui| {
                if ui.button("Library").clicked() {
                    Message::SetPage(Box::new(LibraryPage::default())).send();
                }
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
