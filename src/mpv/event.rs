use core::ffi::CStr;

use libmpv2::mpv_log_level;
use libmpv2_sys::{mpv_event, mpv_event_name, mpv_node, mpv_node_list};

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
            libmpv2_sys::mpv_event_id_MPV_EVENT_COMMAND_REPLY => {
                let cmd = unsafe { &*(event.data as *const libmpv2_sys::mpv_event_command) };

                if cmd.result.format == libmpv2_sys::mpv_format_MPV_FORMAT_NODE_MAP {
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

struct NodeMap<'a>(&'a mpv_node_list);
impl NodeMap<'_> {
    fn new(node: &mpv_node) -> Option<Self> {
        if node.format == libmpv2_sys::mpv_format_MPV_FORMAT_NODE_MAP {
            Some(Self(unsafe { &*node.u.list }))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.0.num as usize
    }

    fn keys(&self) -> impl Iterator<Item = &CStr> {
        let keys = unsafe { std::slice::from_raw_parts(self.0.keys, self.len()) };
        keys.iter().map(|&k| unsafe { CStr::from_ptr(k) })
    }

    fn get_raw(&self, key: &CStr) -> Option<&mpv_node> {
        let keys = unsafe { std::slice::from_raw_parts(self.0.keys, self.len()) };
        let values = unsafe { std::slice::from_raw_parts(self.0.values, self.len()) };

        for (k, v) in keys.iter().zip(values.iter()) {
            if unsafe { CStr::from_ptr(*k) } == key {
                return Some(v);
            }
        }

        None
    }

    fn get<T>(&self, key: &CStr) -> Option<T>
    where
        T: FromMpvNode,
    {
        let node = self.get_raw(key)?;
        assert_eq!(node.format, T::FORMAT);
        Some(T::from_mpv_node(node))
    }
}
trait FromMpvNode: Sized {
    const FORMAT: libmpv2_sys::mpv_format;
    fn from_mpv_node(node: &mpv_node) -> Self;
}
impl FromMpvNode for i64 {
    const FORMAT: libmpv2_sys::mpv_format = libmpv2_sys::mpv_format_MPV_FORMAT_INT64;
    fn from_mpv_node(node: &mpv_node) -> Self {
        unsafe { node.u.int64 }
    }
}
impl FromMpvNode for &[u8] {
    const FORMAT: libmpv2_sys::mpv_format = libmpv2_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY;
    fn from_mpv_node(node: &mpv_node) -> Self {
        let ba = unsafe { &*node.u.ba };
        unsafe { std::slice::from_raw_parts(ba.data as *const u8, ba.size) }
    }
}
