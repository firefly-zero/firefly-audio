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

    pub fn write(&mut self, buf: &mut [f32]) {
        debug_assert_eq!(buf.len() % 16, 0);
        let mut buf = buf;
        while buf.len() >= 8 {
            let Some(frame) = self.root.next_frame() else {
                // fill the remainder of the buffer with zeros
                // to avoid playing old values
                buf.fill(0.);
                break;
            };

            // write left
            let left = &frame.left.as_array_ref()[..];
            buf[..8].copy_from_slice(left);
            buf = &mut buf[8..];

            // write right
            if let Some(right) = &frame.right {
                let right = &right.as_array_ref()[..];
                buf[..8].copy_from_slice(right);
            } else {
                buf[..8].copy_from_slice(left);
            }
            buf = &mut buf[8..];
        }
    }
}
