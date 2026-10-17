//! Dented's animation/combat contract. Contact times drive both damage and clip retiming.
//! Quaternius clips are already embedded, CC0, and use the engine's full biped skeleton.
use super::dented::Action;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub clip: &'static str,
    pub recovery: Option<&'static str>,
    pub source_hit: f32,
    pub source_end: f32,
    pub duration: f32,
    pub contact: f32,
    pub chain: f32,
    pub travel: f32,
    pub damage: f32,
}

pub fn attack(action: Action, combo: u8, great: bool) -> Attack {
    let mut a = if action == Action::Heavy {
        Attack {
            clip: "QUATERNIUS/Sword_Attack",
            recovery: None,
            source_hit: 0.433,
            source_end: 1.30,
            duration: 0.94,
            contact: 0.43,
            chain: 0.78,
            travel: 0.92,
            damage: 1.0,
        }
    } else {
        match combo % 3 {
            0 => Attack {
                clip: "QUATERNIUS/Sword_Regular_A",
                recovery: Some("QUATERNIUS/Sword_Regular_A_Rec"),
                source_hit: 0.25,
                source_end: 0.433,
                duration: 0.58,
                contact: 0.20,
                chain: 0.34,
                travel: 0.62,
                damage: 1.0,
            },
            1 => Attack {
                clip: "QUATERNIUS/Sword_Regular_B",
                recovery: Some("QUATERNIUS/Sword_Regular_B_Rec"),
                source_hit: 0.25,
                source_end: 0.533,
                duration: 0.62,
                contact: 0.23,
                chain: 0.39,
                travel: 0.56,
                damage: 1.05,
            },
            _ => Attack {
                clip: "QUATERNIUS/Sword_Regular_C",
                recovery: None,
                source_hit: 0.683,
                source_end: 1.67,
                duration: 0.83,
                contact: 0.38,
                chain: 0.70,
                travel: 0.84,
                damage: 1.35,
            },
        }
    };
    if great {
        a.duration *= 1.20;
        a.contact *= 1.22;
        a.chain *= 1.20;
        a.travel *= 0.82;
    }
    a
}
impl Attack {
    /// Preserve readable anticipation, accelerate through contact, then settle into recovery.
    pub fn clip_time(self, elapsed: f32) -> f32 {
        let before = (self.contact - 0.085).max(0.01);
        let after = self.contact + 0.075;
        if elapsed < before {
            elapsed / before * (self.source_hit - 0.09).max(0.0)
        } else if elapsed < self.contact {
            self.source_hit - 0.09 + (elapsed - before) / (self.contact - before) * 0.09
        } else if elapsed < after {
            self.source_hit + (elapsed - self.contact) / 0.075 * 0.10
        } else {
            let end = if self.recovery.is_some() { self.chain } else { self.duration };
            self.source_hit
                + 0.10
                + ((elapsed - after) / (end - after).max(0.01)).clamp(0.0, 1.0) * (self.source_end - self.source_hit - 0.10)
        }
    }
    /// Collision-resolved displacement, concentrated in the driving step before contact.
    pub fn root_speed(self, elapsed: f32) -> f32 {
        let start = (self.contact - 0.14).max(0.0);
        let end = self.contact + 0.08;
        if elapsed < start || elapsed > end {
            return 0.0;
        }
        let u = (elapsed - start) / (end - start);
        self.travel * 6.0 * u * (1.0 - u) / (end - start)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Motion {
    pub stride: f32,
    pub speed: f32,
    pub velocity: Vec3,
    pub grounded: bool,
    pub air_time: f32,
    pub landing: f32,
    pub guard: f32,
    pub turn: f32,
    pub dodge_yaw: f32,
    pub combo_next: u8,
    pub combo_left: f32,
    pub reaction: f32,
    pub reaction_dir: Vec3,
    pub swing_sound: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactKind {
    Hit,
    Heavy,
    Break,
    Block,
    Parry,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Impact {
    pub pos: Vec3,
    pub direction: Vec3,
    pub age: f32,
    pub kind: ImpactKind,
}
impl Impact {
    pub fn strength(&self) -> f32 {
        match self.kind {
            ImpactKind::Hit => 0.55,
            ImpactKind::Heavy => 0.85,
            ImpactKind::Break | ImpactKind::Parry => 1.0,
            ImpactKind::Block => 0.4,
        }
    }
}
