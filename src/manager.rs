use crate::*;
use alloc::boxed::Box;

pub enum Sink {
    Default,
    Headphones,
    Speakers,
}

pub struct Manager {
    root: Node,
}

impl Manager {
    pub fn new() -> Self {
        let mut root = Node::new_root();
        for _ in 0..3 {
            root.add(Box::new(Empty::new()));
        }
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
