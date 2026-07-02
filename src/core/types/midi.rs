use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(transparent)]
pub struct InstrumentID(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(transparent)]
pub struct SampleID(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(transparent)]
pub struct PatternID(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[serde(transparent)]
pub struct TrackID(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[serde(transparent)]
pub struct Tick(pub usize);

impl Add for Tick {
    type Output = Tick;
    
    fn add(self, rhs: Tick) -> Self::Output {
        Tick(self.0 + rhs.0)
    }
}


#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub struct Pitch {
    pub pitch: i16,
    pub tune: i16
}

impl Pitch {
    pub fn from_cents(cents: f32) -> Pitch {
        Pitch {
            pitch: (cents / 100.).floor() as i16,
            tune: (cents % 100.) as i16
        }
    }

    pub fn new(pitch: i16, tune: i16) -> Pitch {
        Pitch { pitch, tune }.normalize()
    }

    pub fn normalize(&self) -> Pitch {
        let total_cents = self.cents();
        Pitch {
            pitch: (total_cents / 100.).floor() as i16,
            tune: (total_cents % 100.) as i16
        }
    }

    pub fn cents(&self) -> f32 {
        self.pitch as f32 * 100. + self.tune as f32
    }

    pub fn frequency(&self) -> f32 {
        2f32.powf(self.cents() / 1200.)
    }
}

impl Add for Pitch {
    type Output = Pitch;
    fn add(self, rhs: Self) -> Self::Output {
        Pitch {
            pitch: self.pitch + rhs.pitch,
            tune: self.tune + rhs.tune
        }.normalize()
    }
}

impl Sub for Pitch {
    type Output = Pitch;
    fn sub(self, rhs: Self) -> Self::Output {
        Pitch {
            pitch: self.pitch - rhs.pitch,
            tune: self.tune - rhs.tune
        }.normalize()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Note {
    pub instrument: InstrumentID,
    pub pitch: Pitch,
    pub velocity: f32, // 0. (no audio) ~ 1. (loudest)
    pub pan: f32, // -1. (left ear enjoyment) ~ 0. (center) ~ 1. (right ear enjoyment)
    pub time: Tick
}

/*
impl Note {
    fn t0(&self) -> Note {
        Note {
            instrument: self.instrument,
            pitch: self.pitch,
            velocity: self.velocity,
            pan: self.pan,
            time: Tick(0)
        }
    }
}
*/

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Sample {
    pub id: SampleID,
    pub source: String,
    pub base_pitch: Pitch,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Instrument {
    pub id: InstrumentID,
    pub name: String,
    pub sample: SampleID,
    pub transpose: Pitch
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pattern {
    pub id: PatternID,
    pub name: String,
    pub notes: Vec<Note>,
    pub duration: Tick
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct PatternInstance {
    pub pattern: PatternID,
    pub time: Tick,
    pub end_time: Option<Tick>,
    pub track: TrackID
}

impl PatternInstance {
    pub fn duration(&self, pattern: &Pattern) -> Tick {
        if let Some(end_time) = self.end_time {
            end_time
        } else {
            let last_note = pattern.notes.iter().max_by_key(|note| note.time.0);
            if let Some(last_note) = last_note {
                last_note.time
            } else {
                Tick(0)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8
}
