# firefly-audio

[ [📄 docs](https://docs.rs/firefly-audio/latest/firefly_audio/) ] [ [🐙 github](https://github.com/firefly-zero/firefly-audio) ] [ [📦 crates.io](https://crates.io/crates/firefly-audio) ]

Rust crate for generating and processing digital audio. Powers the audio in [Firefly Zero](https://fireflyzero.com/). If you're looking into using audio in a Firefly Zero app, check out [Firefly Zero documentation](https://docs.fireflyzero.com/dev/audio/).

## Installation

```bash
cargo add firefly-audio
```

## Usage

```rust
let mut manager = firefly_audio::Manager::new();
let node = Box::new(firefly_audio::Sine::new(440., 0.));
manager.add_node(0, node);
manager.write(some_audio_buffer);
```
