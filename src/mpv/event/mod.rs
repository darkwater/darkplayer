mod node;
mod property;

use core::ffi::CStr;

use libmpv2::{mpv_format, mpv_log_level};
use libmpv2_sys::{mpv_event, mpv_event_name};
pub use property::Properties;

use self::{node::NodeMap, property::PropertyChange};

#[derive(Debug, Clone)]
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

    PropertyChange(PropertyChange),

    CommandReplyScreenshot {
        width: u32,
        height: u32,
        stride: u32,
        data: Vec<u8>,
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
            libmpv2_sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => {
                let prop = unsafe { &*(event.data as *const libmpv2_sys::mpv_event_property) };

                let name = unsafe { CStr::from_ptr(prop.name) };

                let double = prop.data as *const f64;

                let change = match (name.to_bytes(), prop.format) {
                    (b"time-pos", mpv_format::Double) => {
                        PropertyChange::TimePos(Some(unsafe { *double }))
                    }
                    (b"duration", mpv_format::Double) => {
                        PropertyChange::Duration(Some(unsafe { *double }))
                    }
                    (b"time-pos", mpv_format::None) => PropertyChange::TimePos(None),
                    (b"duration", mpv_format::None) => PropertyChange::Duration(None),
                    _ => {
                        log::warn!(
                            "Unhandled property change: name={:?}, format={:?}",
                            name,
                            prop.format
                        );
                        return MpvEvent::None;
                    }
                };

                Self::PropertyChange(change)
            }
            libmpv2_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => {
                let cmd = unsafe { &*(event.data as *const libmpv2_sys::mpv_event_command) };

                if cmd.result.format == libmpv2_sys::mpv_format_MPV_FORMAT_NONE {
                    Self::None
                } else if cmd.result.format == libmpv2_sys::mpv_format_MPV_FORMAT_NODE_MAP {
                    let map = NodeMap::new(&cmd.result).expect("Expected a node map");
                    log::debug!("Command reply map keys: {:?}", map.keys().collect::<Vec<_>>());

                    // TODO: use userdata instead?
                    if let Some(screenshot) = parse_screenshot_reply(&map) {
                        screenshot
                    } else {
                        log::error!(
                            "Unhandled command reply map: {:?}",
                            map.keys().collect::<Vec<_>>()
                        );
                        Self::None
                    }
                } else {
                    log::error!("Unhandled command reply: {:?}", cmd.result);
                    Self::None
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

fn parse_screenshot_reply(map: &NodeMap) -> Option<MpvEvent> {
    let width = map.get::<i64>(c"w")? as u32;
    let height = map.get::<i64>(c"h")? as u32;
    let stride = map.get::<i64>(c"stride")? as u32;
    let data = map.get::<&[u8]>(c"data")?;

    Some(MpvEvent::CommandReplyScreenshot {
        width,
        height,
        stride,
        data: data.to_vec(),
    })
}
