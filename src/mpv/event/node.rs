use core::ffi::CStr;

use libmpv2_sys::{mpv_node, mpv_node_list};

pub struct NodeMap<'a>(&'a mpv_node_list);
impl NodeMap<'_> {
    pub fn new(node: &mpv_node) -> Option<Self> {
        if node.format == libmpv2_sys::mpv_format_MPV_FORMAT_NODE_MAP {
            Some(Self(unsafe { &*node.u.list }))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.0.num as usize
    }

    pub fn keys(&self) -> impl Iterator<Item = &CStr> {
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

    pub fn get<T>(&self, key: &CStr) -> Option<T>
    where
        T: FromMpvNode,
    {
        let node = self.get_raw(key)?;
        assert_eq!(node.format, T::FORMAT);
        Some(T::from_mpv_node(node))
    }
}
pub trait FromMpvNode: Sized {
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
