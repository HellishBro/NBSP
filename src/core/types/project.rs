use serde::{Deserialize, Serialize};

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