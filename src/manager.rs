use crate::*;

pub struct Manager {
    pub root: Node,
    prev: Option<Frame>,
    consumed: usize,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            root: Node::new_root(),
            prev: None,
            consumed: 0,
        }
    }

    pub fn write(&mut self, buf: &mut [i16]) {
        // If there is partially emitted frame left from the previous write iteration,
        // write it to the buffer.
        let mut buf = self.write_prev(buf);

        while buf.len() >= 16 {
            let Some(frame) = self.root.next_frame() else {
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

        // If the buffer is not completely filled yet, partially write
        // the next frame into it. The rest of the frame will be written
        // on the next write loop.
        if !buf.is_empty() {
            if let Some(frame) = self.root.next_frame() {
                self.prev = Some(frame);
                self.consumed = 0;
                buf = self.write_prev(buf);
                debug_assert!(buf.is_empty())
            }
            // fill the remainder of the buffer with zeros
            // to avoid playing old values
            buf.fill(0);
        }
    }

    #[must_use]
    fn write_prev<'a>(&mut self, buf: &'a mut [i16]) -> &'a mut [i16] {
        debug_assert!(self.consumed < 16);
        let Some(frame) = &mut self.prev else {
            return buf;
        };
        let left = frame.left.as_array_ref();
        let mut consumed = self.consumed;
        let mut buf = buf;

        // If the left channel is not fully consumed, write it to the buffer.
        if consumed < 8 {
            let written = fill_buf_s(buf, &left[consumed..]);
            consumed += written;
            buf = &mut buf[written..];
        }

        if (8..16).contains(&consumed) {
            let right = match frame.right {
                Some(right) => right,
                None => frame.left,
            };
            let right = right.as_array_ref();
            let written = fill_buf_s(buf, &right[(consumed - 8)..]);
            consumed += written;
            buf = &mut buf[written..];
        }

        if consumed >= 16 {
            debug_assert_eq!(consumed, 16);
            self.prev = None;
            self.consumed = 0;
        } else {
            self.consumed = consumed;
        }
        &mut buf[..]
    }
}

fn fill_buf(buf: &mut [i16], src: &[f32; 8]) {
    // TODO: use SIMD operations
    for (t, s) in buf.iter_mut().zip(src) {
        *t = (s * i16::MAX as f32) as i16
    }
}

fn fill_buf_s(buf: &mut [i16], src: &[f32]) -> usize {
    for (t, s) in buf.iter_mut().zip(src) {
        *t = (s * i16::MAX as f32) as i16;
    }
    buf.len().min(src.len())
}
