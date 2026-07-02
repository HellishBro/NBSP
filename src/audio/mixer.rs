use crate::audio::sample::Sample;
use rodio::mixer::Mixer as RodioMixer;

pub trait MixerT {
    fn add(&self, sample: &Sample);
}

pub struct Mixer<'a>(&'a RodioMixer);

impl<'a> Mixer<'a> {
    pub fn from_rodio(mixer: &'a RodioMixer) -> Self {
        Self(mixer)
    }
}

impl<'a> MixerT for Mixer<'a> {
    fn add(&self, sample: &Sample) {
        self.0.add(sample.to_samples_buffer())
    }
}