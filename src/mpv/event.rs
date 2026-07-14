use core::ffi::CStr;

use libmpv2::mpv_log_level;
use libmpv2_sys::{mpv_event, mpv_event_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MpvEvent {
    /// Nothing happened. Happens on timeouts or sporadic wakeups.
    None,

    /// Happens when the player quits. The player enters a state where it tries to disconnect all
    /// clients. Most requests to the player will fail, and the client should react to this and
    /// quit with mpv_destroy() as soon as possible.
    Shutdown,

    LogMessage {
        prefix: String,
        msg: String,
        level: log::Level,
    },

    UnknownNamed(String),
    Unknown(u32),
}

impl From<&mpv_event> for MpvEvent {
    fn from(event: &mpv_event) -> Self {
        match event.event_id {
            libmpv2_sys::mpv_event_id_MPV_EVENT_NONE => Self::None,
            libmpv2_sys::mpv_event_id_MPV_EVENT_SHUTDOWN => Self::Shutdown,
            libmpv2_sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => {
                let msg = unsafe { &*(event.data as *const libmpv2_sys::mpv_event_log_message) };

                Self::LogMessage {
                    prefix: unsafe { CStr::from_ptr(msg.prefix) }
                        .to_string_lossy()
                        .into_owned(),

                    msg: unsafe { CStr::from_ptr(msg.text) }
                        .to_string_lossy()
                        .trim_end()
                        .to_owned(),

                    level: match msg.log_level {
                        mpv_log_level::Trace.. => log::Level::Trace,
                        mpv_log_level::Debug.. => log::Level::Debug,
                        mpv_log_level::Info.. => log::Level::Info,
                        mpv_log_level::Warn.. => log::Level::Warn,
                        _ => log::Level::Error,
                    },
                }
            }
            event_id => {
                let event_name = unsafe { mpv_event_name(event_id) };

                if event_name.is_null() {
                    Self::Unknown(event_id)
                } else {
                    Self::UnknownNamed(
                        unsafe { CStr::from_ptr::<'static>(event_name) }
                            .to_string_lossy()
                            .into_owned(),
                    )
                }
            }
        }
    }
}
