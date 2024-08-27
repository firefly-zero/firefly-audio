use crate::*;
use alloc::boxed::Box;

pub enum Sink {
    Default,
    Headphones,
    Speakers,
}

impl Sink {
    pub fn from_id(id: u32) -> Option<Sink> {
        match id {
            DEFAULT_SINK => Some(Self::Default),
            HEADPHONES_SINK => Some(Self::Headphones),
            SPEAKERS_SINK => Some(Self::Speakers),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Sink::Default => DEFAULT_SINK,
            Sink::Headphones => HEADPHONES_SINK,
            Sink::Speakers => SPEAKERS_SINK,
        }
    }
}

const DEFAULT_SINK: u32 = 1;
const HEADPHONES_SINK: u32 = 2u32.pow(30);
const SPEAKERS_SINK: u32 = 2u32.pow(31) - 1;

pub struct Manager {
    pub root: Node,
}

impl Manager {
    pub fn new() -> Self {
        let mut root = Node::new_root();
        debug_assert_eq!(root.children.len(), 0);
        for _ in 0..3 {
            let added = root.add(Box::new(Empty::new()));
            debug_assert!(added);
        }
        debug_assert_eq!(root.children.len(), 3);
        debug_assert!(root.children[0].id < root.children[1].id);
        debug_assert!(root.children[1].id < root.children[2].id);
        Self { root }
    }

    pub fn write(&mut self, sink: &Sink, buf_left: &mut [f32], buf_right: &mut [f32]) {
        debug_assert_eq!(buf_left.len() % 8, 0);
        if !buf_right.is_empty() {
            debug_assert_eq!(buf_left.len(), buf_right.len())
        }
        buf_left.fill(0.);
        buf_right.fill(0.);
        let mut buf_left = buf_left;
        let mut buf_right = buf_right;

        let node_idx = match sink {
            Sink::Default => 0,
            Sink::Headphones => 1,
            Sink::Speakers => 2,
        };
        let node = &mut self.root.children[node_idx];

        while buf_left.len() >= 8 {
            let Some(frame) = node.next_frame() else {
                break;
            };
            buf_left[..8].copy_from_slice(&frame.left.as_array_ref()[..]);
            buf_left = &mut buf_left[8..];
            if buf_right.len() >= 8 {
                if let Some(right) = &frame.right {
                    buf_right[..8].copy_from_slice(&right.as_array_ref()[..]);
                }
                buf_right = &mut buf_right[8..];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sink_ids() {
        let m = Manager::new();
        assert_eq!(m.root.children[0].id, DEFAULT_SINK);
        assert_eq!(m.root.children[1].id, HEADPHONES_SINK);
        assert_eq!(m.root.children[2].id, SPEAKERS_SINK);
    }
}
