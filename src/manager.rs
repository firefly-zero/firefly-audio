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

    pub fn write(&mut self, buf: &mut [i16]) {
        debug_assert_eq!(buf.len() % 16, 0);
        let mut buf = buf;
        while buf.len() >= 8 {
            let Some(frame) = self.root.next_frame() else {
                // fill the remainder of the buffer with zeros
                // to avoid playing old values
                buf.fill(0);
                break;
            };

            // write left
            let left = frame.left.as_array_ref();
            fill_buf(buf, left);
            buf = &mut buf[8..];

            // write right
            if let Some(right) = &frame.right {
                let right = right.as_array_ref();
                fill_buf(buf, right);
            } else {
                fill_buf(buf, left);
            }
            buf = &mut buf[8..];
        }
    }
}

fn fill_buf(buf: &mut [i16], src: &[f32; 8]) {
    // buf[..8].copy_from_slice(src)
    // TODO: use SIMD operations
    for (t, s) in buf.iter_mut().zip(src) {
        *t = (s * i16::MAX as f32) as i16
    }
}
