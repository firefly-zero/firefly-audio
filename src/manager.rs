use crate::*;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub struct Manager {
    root: Node,
    paths: Vec<Box<[u8]>>,
    prev: Option<Frame>,
    consumed: usize,
}

impl Manager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Node::new_root(),
            paths: vec![Box::new([])],
            prev: None,
            consumed: 0,
        }
    }

    /// Find the node with the given ID in the graph.
    ///
    /// ## Errors
    ///
    /// If the node is not present, returns [`NodeError::UnknownID`].
    pub fn get_node(&mut self, id: u32) -> Result<&mut Node, NodeError> {
        let Some(path) = self.paths.get(id as usize) else {
            return Err(NodeError::UnknownID(id));
        };
        Ok(self.root.get_node(path))
    }

    /// Find a child node for the node with the given ID. Returns the new node ID.
    ///
    /// ## Errors
    ///
    /// If the parent node is not present, returns [`NodeError::UnknownID`].
    /// If there are too many nodes, returns [`NodeError::TooManyChildren`]
    /// or [`NodeError::TooManyNodes`].
    #[allow(clippy::cast_possible_truncation)]
    pub fn add_node(&mut self, parent_id: u32, b: Box<dyn Processor>) -> Result<u32, NodeError> {
        const MAX_NODES: usize = 32;
        if self.paths.len() >= MAX_NODES {
            return Err(NodeError::TooManyNodes);
        };
        let Some(parent_path) = self.paths.get(parent_id as usize) else {
            return Err(NodeError::UnknownID(parent_id));
        };
        let parent_node = self.root.get_node(parent_path);
        let sub_id = parent_node.add(b)?;
        let id = self.paths.len() as u32;
        let mut path = Vec::new();
        path.extend_from_slice(parent_path);
        path.push(sub_id);
        self.paths.push(path.into_boxed_slice());
        Ok(id)
    }

    /// Remove all child nodes from the node with the given ID.
    ///
    /// ## Errors
    ///
    /// If the node is not present, returns [`NodeError::UnknownID`].
    pub fn clear(&mut self, id: u32) -> Result<(), NodeError> {
        let Some(path) = self.paths.get(id as usize) else {
            return Err(NodeError::UnknownID(id));
        };
        let node = self.root.get_node(path);
        node.clear();
        let mut paths = Vec::new();
        for p in &self.paths {
            if p.len() > path.len() && p.starts_with(path) {
                continue;
            }
            paths.push(p.clone());
        }
        self.paths = paths;
        Ok(())
    }

    /// Fill the given buffer with PCM sound values.
    pub fn write(&mut self, buf: &mut [i16]) {
        // If there is partially emitted frame left from the previous write iteration,
        // write it to the buffer.
        let mut buf = self.write_prev(buf);

        while buf.len() >= 16 {
            let Some(frame) = self.root.next_frame() else {
                break;
            };
            let written = fill_buf(buf, &frame, 0);
            buf = &mut buf[written..];
        }

        // If the buffer is not completely filled yet, partially write
        // the next frame into it. The rest of the frame will be written
        // on the next write loop.
        if !buf.is_empty() {
            if let Some(frame) = self.root.next_frame() {
                self.prev = Some(frame);
                self.consumed = 0;
                buf = self.write_prev(buf);
                debug_assert!(buf.is_empty());
            } else {
                // fill the remainder of the buffer with zeros
                // to avoid playing old values
                buf.fill(0);
            }
        }
    }

    #[must_use]
    fn write_prev<'a>(&mut self, buf: &'a mut [i16]) -> &'a mut [i16] {
        debug_assert!(self.consumed < 16);
        let Some(frame) = &mut self.prev else {
            return buf;
        };
        let written = fill_buf(buf, frame, self.consumed);
        let consumed = self.consumed + written;
        if consumed >= 16 {
            debug_assert_eq!(consumed, 16);
            self.prev = None;
            self.consumed = 0;
        } else {
            self.consumed = consumed;
        }
        &mut buf[written..]
    }
}

/// Write the given frame (starting from skip index) into the beginning of the buffer.
fn fill_buf(buf: &mut [i16], frame: &Frame, skip: usize) -> usize {
    // make iterators over left and right channels
    let right = match frame.right {
        Some(right) => right,
        None => frame.left,
    };
    let mut left = frame.left.as_array_ref().iter();
    let mut right = right.as_array_ref().iter();

    // skip the given number of samples
    let mut even = true;
    for _ in 0..skip {
        if even {
            left.next()
        } else {
            right.next()
        };
        even = !even;
    }

    let mut written = 0;
    #[allow(clippy::cast_possible_truncation)]
    for tar in buf.iter_mut() {
        let chan = if even { &mut left } else { &mut right };
        let Some(s) = chan.next() else { break };
        even = !even;
        written += 1;
        let s = s.clamp(-1., 1.);
        *tar = (s * f32::from(i16::MAX)) as i16;
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_buf_stereo() {
        let frame = Frame::stereo(
            Sample::new([
                u2f(11),
                u2f(13),
                u2f(15),
                u2f(17),
                u2f(19),
                u2f(21),
                u2f(23),
                u2f(25),
            ]),
            Sample::new([
                u2f(12),
                u2f(14),
                u2f(16),
                u2f(18),
                u2f(20),
                u2f(22),
                u2f(24),
                u2f(26),
            ]),
        );
        let mut buf = [0i16; 20];
        fill_buf(&mut buf[..], &frame, 0);
        for (a, b) in buf[..15].iter().zip(&buf[1..]) {
            assert!(a < b, "{a} < {b}");
        }
        assert!(buf[0] != 0);
        assert_eq!(&buf[16..], &[0, 0, 0, 0]);
    }

    #[allow(clippy::cast_lossless)]
    fn u2f(u: u16) -> f32 {
        u as f32 / 100.
    }
}
