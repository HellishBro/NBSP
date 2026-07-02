use crate::audio::mixer::Mixer;
use rodio::MixerDeviceSink;
use std::error::Error;

pub fn get_mixer(device_sink: &Result<MixerDeviceSink, impl Error>) -> Option<Mixer<'_>> {
    if let Ok(device_sink) = device_sink {
        Some(Mixer::from_rodio(device_sink.mixer()))
    } else {
        None
    }
}