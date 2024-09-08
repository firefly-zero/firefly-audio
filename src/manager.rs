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
    pub fn new() -> Self {
        Self {
            root: Node::new_root(),
            paths: vec![Box::new([])],
            prev: None,
            consumed: 0,
        }
    }

    pub fn get_node(&mut self, id: u32) -> Result<&mut Node, NodeError> {
        let Some(path) = self.paths.get(id as usize) else {
            return Err(NodeError::UnknownID(id));
        };
        Ok(self.root.get_node(path))
    }

    pub fn add_node(&mut self, parent_id: u32, b: Box<dyn Processor>) -> Result<u32, NodeError> {
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

    /// Remove all child nodes.
    pub fn clear(&mut self, id: u32) -> Result<(), NodeError> {
        let Some(path) = self.paths.get(id as usize) else {
            return Err(NodeError::UnknownID(id));
        };
        let node = self.root.get_node(path);
        node.children.clear();
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

    pub fn write(&mut self, buf: &mut [i16]) {
        // If there is partially emitted frame left from the previous write iteration,
        // write it to the buffer.
        let mut buf = self.write_prev(buf);

        while buf.len() >= 16 {
            let Some(mut frame) = self.root.next_frame() else {
                break;
            };
            let written = fill_buf(buf, &mut frame, 0);
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
                debug_assert!(buf.is_empty())
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

fn fill_buf(buf: &mut [i16], frame: &mut Frame, skip: usize) -> usize {
    // make iterators over left and right channels
    let right = match frame.right {
        Some(right) => right,
        None => frame.left,
    };
    let mut left = frame.left.as_array_ref().iter();
    let mut right = right.as_array_ref().iter();

    // skip the given number
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
    for tar in buf.iter_mut() {
        let chan = if even { &mut left } else { &mut right };
        let Some(s) = chan.next() else { break };
        even = !even;
        written += 1;
        *tar = (s * i16::MAX as f32) as i16;
    }
    written
}
