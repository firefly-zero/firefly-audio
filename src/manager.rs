use crate::basic_types::*;

// Sinks:
// 0. Discard
// 1. Headphones
// 2. Speakers
// 3. Preferred (headphones if inserted, fallback to speakers)
//
// Sources: 4-12 (8).
// Filters: 13-31 (18).

pub struct Context<'a> {
    values: &'a Values,
    sources: &'a [Frame],
}

impl<'a> Context<'a> {
    fn get_value(&self, i: usize) -> Option<f32> {
        self.values.get(i).copied()
    }

    fn get_source(&self, i: usize) -> Option<Frame> {
        self.sources.get(i).copied()
    }
}

pub struct AudioManager {}
