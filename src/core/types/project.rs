use serde::{Deserialize, Serialize};

use crate::core::types::midi::{InstrumentID, PatternID, SampleID};
use crate::core::types::{midi::{Instrument, Pattern, PatternInstance, Sample, TimeSignature, TrackID}, utils::Timestamp};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub metadata: Metadata,
    pub registry: Registry,
    pub patterns: Vec<PatternInstance>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub creation_time: Timestamp,
    pub updated_time: Option<Timestamp>,
    pub tps: f32,
    pub time_signature: TimeSignature
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Track {
    pub id: TrackID,
    pub name: String,
    pub volume: f32,
    pub pan: f32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Registry {
    pub patterns: Vec<Pattern>,
    pub samples: Vec<Sample>,
    pub instruments: Vec<Instrument>,
    pub tracks: Vec<Track>
}

impl Registry {
    pub fn get_pattern(&self, id: PatternID) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    pub fn get_sample(&self, id: SampleID) -> Option<&Sample> {
        self.samples.iter().find(|p| p.id == id)
    }

    pub fn get_instrument(&self, id: InstrumentID) -> Option<&Instrument> {
        self.instruments.iter().find(|p| p.id == id)
    }

    pub fn get_track(&self, id: TrackID) -> Option<&Track> {
        self.tracks.iter().find(|p| p.id == id)
    }
}