use std::fs::File;
pub mod file_formats;
pub mod core;
pub mod audio;
pub mod macros;

use crate::audio::renderer::{AudioRenderer, SourceAudioRenderer};
use crate::core::types::project::Project;
use std::io::Write;
use std::time::Instant;

macro_rules! time {
    ($x:expr) => {{
        let start = Instant::now();
        let res = $x;
        let dur = start.elapsed();
        (dur.as_secs_f64(), res)
    }};
}

fn main() {
    let (t, song) = time!(serde_json::from_str::<Project>(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/electroman.json"))).unwrap());
    println!("Loaded song in {t} seconds");
    let mut renderer = SourceAudioRenderer::new(AudioRenderer::new(&song));

    let (t, bytes) = time!(renderer.wav_bytes().unwrap());
    println!("Rendered song in {t} seconds");
    let mut output = File::create(concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/electroman.wav")).unwrap();
    let (t, _) = time!(output.write_all(bytes.as_slice()).unwrap());
    println!("Wrote audio in {t} seconds");

    output.flush().unwrap();
}