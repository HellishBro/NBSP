use crate::asset;
use crate::audio::sample::{Sample, SampleIterator, StereoFrame};
use crate::core::types::midi::{Note, Pattern, PatternInstance, Pitch, Tick};
use crate::core::types::project::{Project, Track};
use crate::from_root;
use cached::cached;
use rodio::{ChannelCount, Sample as RodioSample, SampleRate, Source};
use slotmap::{DefaultKey, SlotMap};
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::time::Duration;
use wavers::error::FormatError;
use wavers::{FormatCode, Samples, WavHeader, DATA};

const ROOT_DIR: &str = asset!("sounds");

pub struct AudioRenderer<'a> {
    sample_rate: u32,
    context: AudioRenderContext<'a>,
    samples_per_tick: usize
}

impl<'a> AudioRenderer<'a> {
    const SAMPLE_RATE: u32 = 44100;

    pub fn new(project: &'a Project) -> AudioRenderer<'a> {
        AudioRenderer {
            sample_rate: Self::SAMPLE_RATE,
            context: AudioRenderContext::new(project),
            samples_per_tick: (Self::SAMPLE_RATE as f32 / project.metadata.tps) as usize
        }
    }
}

impl<'a> Iterator for AudioRenderer<'a> {
    type Item = StereoFrame;

    fn next(&mut self) -> Option<StereoFrame> {
        if self.context.current_frame % self.samples_per_tick == 0 {
            self.context.next_tick();
        }
        let this = self.context.next();
        self.context.current_frame += 1;
        this
    }
}

pub struct SourceAudioRenderer<'a> {
    rend: AudioRenderer<'a>,
    this_frame: Option<StereoFrame>,
    channel: usize
}

impl<'a> SourceAudioRenderer<'a> {
    pub fn new(rend: AudioRenderer) -> SourceAudioRenderer {
        SourceAudioRenderer {
            rend,
            this_frame: Some(StereoFrame::default()),
            channel: 0
        }
    }

    pub fn wav_bytes(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        // adapted from wavers::write

        let mut writer = BufWriter::new(Vec::new());
        let me: Vec<f32> = self.collect();
        let samples = Samples::from(me);
        let samples_bytes = samples.as_bytes();
        let new_header = WavHeader::new_header::<f32>(self.rend.sample_rate as i32, 2, samples.len())?;

        match new_header.fmt_chunk.format {
            FormatCode::WAV_FORMAT_PCM | FormatCode::WAV_FORMAT_IEEE_FLOAT => {
                let header_bytes = new_header.as_base_bytes();
                writer.write_all(&header_bytes)?;
            }
            FormatCode::WAVE_FORMAT_EXTENSIBLE => {
                let header_bytes = new_header.as_extended_bytes();
                writer.write_all(&header_bytes)?;
            }
            _ => {
                return Err(FormatError::InvalidTypeId("Invalid type ID").into());
            }
        }

        writer.write_all(&DATA)?;
        let data_size_bytes = samples_bytes.len() as u32; // write up to the data size
        writer.write_all(&data_size_bytes.to_ne_bytes())?; // write the data size
        writer.write_all(&samples_bytes)?; // write the data
        Ok(writer.get_ref().clone())
    }
}

impl<'a> Iterator for SourceAudioRenderer<'a> {
    type Item = RodioSample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == 0 {
            self.this_frame = self.rend.next();
            self.channel = 1;
        } else if self.channel == 1 {
            self.channel = 0;
        }

        if let Some(f) = self.this_frame {
            if self.channel == 0 {
                Some(f.0)
            } else if self.channel == 1 {
                Some(f.1)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> Source for SourceAudioRenderer<'a> {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(2).unwrap()
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(self.rend.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

struct PatternInstanceWithDuration<'a> {
    instance: &'a PatternInstance,
    track: &'a Track,
    duration: Tick,
    pat: &'a Pattern
}

impl<'a> PatternInstanceWithDuration<'a> {
    #[inline]
    fn end_tick(&self) -> Tick {
        self.instance.time + self.duration
    }
}

#[derive(Clone, PartialEq)]
struct PlayedNote {
    sample_source: String,
    pitch: Pitch,
    velocity: f32,
    pan: f32
}

impl Hash for PlayedNote {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sample_source.hash(state);
        self.pitch.hash(state);
        self.velocity.to_bits().hash(state);
        self.pan.to_bits().hash(state);
    }
}

impl Eq for PlayedNote {}

#[cached]
fn get_sample(sample_source: String) -> Option<Sample> {
    let mut source = sample_source;
    if source.starts_with("~") {
        source = source.replace("~", ROOT_DIR);
    }
    if let Ok(file) = File::open(&source) && let Ok(sample) = Sample::from_file_descriptor(file) {
        Some(sample)
    } else {
        None
    }
}

#[cached]
fn render_note(note: PlayedNote) -> Option<Sample> {
    if let Some(sample) = get_sample(note.sample_source) {
        Some(sample.shift_pitch(note.pitch.cents()).shift_volume(note.velocity).pan(note.pan))
    } else {
        None
    }
}

struct AudioRenderContext<'a> {
    project: &'a Project,
    current_frame: usize,
    tick_start_frame: usize,
    tick: Tick,
    current_patterns: SlotMap<DefaultKey, PatternInstanceWithDuration<'a>>,
    current_notes: SlotMap<DefaultKey, SampleIterator>,
    has_more: bool
}

impl<'a> AudioRenderContext<'a> {
    fn new(project: &'a Project) -> AudioRenderContext<'a> {
        AudioRenderContext {
            project,
            current_frame: 0,
            tick_start_frame: 0,
            tick: Tick(0),
            current_patterns: SlotMap::with_capacity(32),
            current_notes: SlotMap::with_capacity(512),
            has_more: true
        }
    }

    fn next_tick(&mut self) {
        self.tick_start_frame = self.current_frame;
        self.poll_patterns();
        self.poll_notes();
        self.tick.0 += 1;
    }

    fn poll_patterns(&mut self) {
        // remove outdated pattern instances first
        self.current_patterns.retain(|_, inst| {
            inst.end_tick() > self.tick
        });

        self.has_more = false;

        // find new ones
        for inst in &self.project.patterns {
            if inst.time == self.tick {
                let pat = self.project.registry.get_pattern(inst.pattern);
                let track = self.project.registry.get_track(inst.track);
                if let Some(canon) = pat && let Some(track) = track {
                    self.current_patterns.insert(PatternInstanceWithDuration {
                        instance: inst,
                        track,
                        duration: inst.duration(canon),
                        pat: canon
                    });
                }
            }
            if inst.time >= self.tick {
                self.has_more = true;
            }
        }
    }

    fn render_note(&self, note: &Note, track: &Track) -> Option<SampleIterator> {
        if
            let Some(instrument) = &self.project.registry.get_instrument(note.instrument) &&
            let Some(sample) = &self.project.registry.get_sample(instrument.sample)
        {
            let absolute_pitch = note.pitch + instrument.transpose - sample.base_pitch;
            let absolute_velocity = note.velocity * track.volume;
            let absolute_pan = (note.pan + track.pan).clamp(-1., 1.);
            render_note(PlayedNote {
                sample_source: sample.source.clone(),
                pitch: absolute_pitch,
                velocity: absolute_velocity,
                pan: absolute_pan,
            }).map(|s| s.into_iter())
        } else {
            None
        }
    }

    fn poll_notes(&mut self) {
        // remove outdated notes first
        self.current_notes.retain(|_, samp_iter| {
            !samp_iter.is_done()
        });

        // find new ones
        for pat_inst in self.current_patterns.values() {
            for note in &pat_inst.pat.notes {
                if note.time + pat_inst.instance.time == self.tick &&
                    let Some(rendered) = self.render_note(note, pat_inst.track) {
                        self.current_notes.insert(rendered);
                }
            }
        }
    }
}

impl<'a> Iterator for AudioRenderContext<'a> {
    type Item = StereoFrame;

    fn next(&mut self) -> Option<StereoFrame> {
        let mut sum = StereoFrame::default();
        let mut has_notes = false;

        for note in self.current_notes.values_mut() {
            if let Some(frame) = note.next() {
                sum.0 += frame.0;
                sum.1 += frame.1;
                has_notes = true;
            }
        }

        if self.has_more || has_notes {
            Some(sum)
        } else {
            None
        }
    }
}