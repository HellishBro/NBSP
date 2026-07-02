use crate::core::types::midi::Pitch;
use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, Float, Sample as RodioSample, Source};
use std::error::Error as ErrorT;
use std::f32::consts::PI;
use std::io::{Cursor, Error, ErrorKind, Read, Seek};
use std::num::NonZero;

type MonoFrame = RodioSample;
pub type StereoFrame = (RodioSample, RodioSample);

#[derive(Clone, Debug)]
pub struct Sample {
    sample_rate: u32,
    samples: Vec<StereoFrame>
}

fn lerp(a: StereoFrame, b: StereoFrame, t: f32) -> StereoFrame {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

impl Sample {
    pub fn from_mono(sample_rate: u32, mono: Vec<MonoFrame>) -> Sample {
        let sqrt_half = (0.5 as Float).sqrt();
        let mut samples = Vec::<StereoFrame>::with_capacity(mono.len());
        for sample in mono {
            samples.push((sample * sqrt_half, sample * sqrt_half));
        }
        Sample { sample_rate, samples }
    }

    pub fn from_stereo(sample_rate: u32, stereo: Vec<StereoFrame>) -> Sample {
        Sample { sample_rate, samples: stereo }
    }

    pub fn to_samples_buffer(&self) -> SamplesBuffer {
        SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(self.sample_rate).unwrap(),
            self.to_interleaved()
        )
    }

    pub fn from_file(data: &'static [u8]) -> Result<Sample, Box<dyn ErrorT>> {
        let cursor = Cursor::new(data);
        let decoder = Decoder::new(cursor)?;
        match decoder.channels().get() {
            1 => Ok(Sample::from_mono_decoder(decoder)),
            2 => Ok(Sample::from_stereo_decoder(decoder)),
            channels => Err(Box::from(Error::new(ErrorKind::InvalidInput, format!("Unsupported {channels}-channel audio"))))
        }
    }
}

impl Sample {
    pub fn shift_pitch(&self, cents: f32) -> Sample {
        let factor = Pitch::from_cents(cents).frequency();
        let original_length = self.samples.len();
        if original_length == 1 { return self.clone(); }

        let new_length = (original_length as f32 / factor).round() as usize; // inverse, as >factor when >cents, <factor when <cents
        if new_length <= 1 {
            return Sample { sample_rate: self.sample_rate, samples: vec![self.samples[0]] };
        }

        let mut res = Vec::<StereoFrame>::with_capacity(new_length);
        for i in 0..new_length {
            let pos = (i as f32 / (new_length - 1) as f32) * (original_length - 1) as f32;
            let floor = pos.floor() as usize;
            let fractional = pos.fract();
            let ceil = (floor + 1).min(original_length - 1);
            res.push(lerp(self.samples[floor], self.samples[ceil], fractional));
        }

        Sample::from_stereo(self.sample_rate, res)
    }

    pub fn pan(&self, pan: f32) -> Sample {
        let angle = (pan + 1.) * PI / 4.;
        let left_gain = angle.cos();
        let right_gain = angle.sin();
        let mut res = Vec::<StereoFrame>::with_capacity(self.samples.len());
        for pair in &self.samples {
            res.push((pair.0 * left_gain, pair.1 * right_gain));
        }
        Sample::from_stereo(self.sample_rate, res)
    }
}

impl Sample {
    fn to_interleaved(&self) -> Vec<RodioSample> {
        let mut v = Vec::<RodioSample>::with_capacity(self.samples.len() * 2);
        for pair in &self.samples {
            v.push(pair.0);
            v.push(pair.1);
        }
        v
    }

    fn from_mono_decoder<R: Read + Seek>(decoder: Decoder<R>) -> Sample {
        let sample_rate = decoder.sample_rate().get();
        let mut vec = Vec::<MonoFrame>::with_capacity(decoder.size_hint().0);
        for sample in decoder.into_iter() {
            vec.push(sample);
        }

        Sample::from_mono(sample_rate, vec)
    }

    fn from_stereo_decoder<R: Read + Seek>(decoder: Decoder<R>) -> Sample {
        let sample_rate = decoder.sample_rate().get();
        let mut vec = Vec::<StereoFrame>::with_capacity(decoder.size_hint().0);
        let mut iter = decoder.into_iter();
        while let (Some(a), Some(b)) = (iter.next(), iter.next()) { 
            vec.push((a, b));
        }

        Sample::from_stereo(sample_rate, vec)
    }
}