use crate::*;

pub struct Manager {
    pub root: Node,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            root: Node::new_root(),
        }
    }

    pub fn write(&mut self, buf_left: &mut [f32], buf_right: &mut [f32]) {
        debug_assert_eq!(buf_left.len() % 8, 0);
        if !buf_right.is_empty() {
            debug_assert_eq!(buf_left.len(), buf_right.len())
        }
        buf_left.fill(0.);
        buf_right.fill(0.);
        let mut buf_left = buf_left;
        let mut buf_right = buf_right;

        while buf_left.len() >= 8 {
            let Some(frame) = self.root.next_frame() else {
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
