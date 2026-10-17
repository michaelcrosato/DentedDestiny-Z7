//! Library poses retimed to gameplay, layered locomotion, and whole-body reactions.
use glam::{Quat, Vec3};
use pav_core::{
    clips,
    games::{
        dented::{Action, Dented, Enemy, Intent, Kind, Mode},
        dented_motion::{self as combat, Motion},
    },
    puppet::{BodyPlan, PuppetDef, Skel},
};
use std::sync::OnceLock;

pub fn rig() -> &'static PuppetDef {
    static RIG: OnceLock<PuppetDef> = OnceLock::new();
    RIG.get_or_init(|| PuppetDef {
        scale: 1.0,
        leg_length: 0.90,
        arm_length: 0.71,
        torso_length: 0.49,
        shoulder_width: 0.29,
        hip_width: 0.16,
        head_radius: 0.19,
        torso_radius: 0.23,
        limb_radius: 0.075,
        ..PuppetDef::preset(BodyPlan::Biped)
    })
}
pub fn sample(name: &str, t: f32) -> Skel {
    clips::pose_of(rig(), clips::find_cached(name), t, 0).unwrap_or_else(|| {
        clips::pose_of(rig(), clips::find_cached("QUATERNIUS/Sword_Idle"), 0., 0).expect("embedded sword idle")
    })
}
fn blend(a: &Skel, b: &Skel, w: f32) -> Skel {
    clips::blend(rig(), a, b, w.clamp(0., 1.), false)
}
pub fn yaw_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + ((b - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI) * t
}
fn locomotion(m: &Motion, facing: f32, time: f32) -> Skel {
    let idle = sample("QUATERNIUS/Sword_Idle", time);
    let stride = m.stride.rem_euclid(1.);
    let mut gait = sample(
        if m.speed < 2.4 { "QUATERNIUS/Walk_Loop" } else { "MESH2MOTION/Jog" },
        stride * if m.speed < 2.4 { 1.333 } else { 1.167 },
    );
    let local_v = Quat::from_rotation_y(-facing) * m.velocity;
    let direction = local_v.x.atan2(local_v.z);
    // The legs follow travel while the chest and eyes keep watching the locked opponent.
    if m.speed > 0.1 {
        if local_v.z < -m.speed * 0.5 {
            gait = sample("MESH2MOTION/Walk_Backwards", stride);
        } else {
            gait.transform(Quat::from_rotation_y(direction), Vec3::ZERO);
        }
    }
    gait = clips::blend(rig(), &gait, &idle, 0.78, true);
    let mut sk = blend(&idle, &gait, (m.speed / 1.2).clamp(0., 1.));
    if m.guard > 0.01 {
        let mut guard = sample("QUATERNIUS/Idle_Shield_Loop", time);
        guard.weapon = idle.weapon;
        sk = clips::blend(rig(), &sk, &guard, m.guard, m.speed > 0.2);
    }
    sk.head_rot = Quat::from_rotation_y(m.turn * 0.45) * sk.head_rot;
    sk
}
pub fn attack_pose(action: Action, combo: u8, great: bool, elapsed: f32) -> Skel {
    let a = combat::attack(action, combo, great);
    let mut pose = if elapsed > a.chain {
        if let Some(rec) = a.recovery {
            let t = (elapsed - a.chain) / (a.duration - a.chain) * if combo == 0 { 0.967 } else { 1.033 };
            sample(rec, t)
        } else {
            sample(a.clip, a.clip_time(elapsed))
        }
    } else {
        sample(a.clip, a.clip_time(elapsed))
    };
    if great {
        support_blade(&mut pose);
    }
    pose
}

fn support_blade(sk: &mut Skel) {
    // Project the grip into both arms' reach. Solving only the left elbow leaves its hand
    // floating beside one-handed source poses whose right wrist is out past the shoulder.
    let reach = rig().arm_length * 0.96;
    let left_center = sk.shoulder[0] + sk.weapon * 0.16;
    let mut grip = sk.hand[1];
    for _ in 0..6 {
        for center in [left_center, sk.shoulder[1]] {
            grip = center + (grip - center).clamp_length_max(reach);
        }
    }
    for i in 0..2 {
        let target = if i == 0 { grip - sk.weapon * 0.16 } else { grip };
        let hint = sk.elbow[i] - (sk.shoulder[i] + sk.hand[i]) * 0.5;
        let (elbow, hand) = pav_core::puppet::ik(sk.shoulder[i], target, rig().arm_length * 0.5, rig().arm_length * 0.5, hint);
        sk.elbow[i] = elbow;
        sk.hand[i] = hand;
    }
}

fn recoil(sk: &Skel, motion: &Motion, facing: f32) -> Skel {
    let local = Quat::from_rotation_y(-facing) * motion.reaction_dir;
    let lean = Quat::from_rotation_arc(Vec3::Y, (Vec3::Y + local * 0.42).normalize_or(Vec3::Y));
    let mut bent = *sk;
    bent.transform(lean, sk.pelvis - lean * sk.pelvis);
    clips::blend(rig(), sk, &bent, (motion.reaction / 0.32).clamp(0., 1.), true)
}

pub fn player(g: &Dented, old: &Dented, alpha: f32) -> Skel {
    let time = old.time + (g.time - old.time) * alpha;
    let mut m = g.motion.clone();
    m.stride = old.motion.stride + (g.motion.stride - old.motion.stride) * alpha;
    m.speed = old.motion.speed + (g.motion.speed - old.motion.speed) * alpha;
    m.guard = old.motion.guard + (g.motion.guard - old.motion.guard) * alpha;
    let base = locomotion(&m, g.facing, time);
    if g.mode == Mode::Dead {
        return blend(&base, &sample("QUATERNIUS/Death01", g.mode_time.min(2.399)), (g.mode_time / 0.12).min(1.));
    }
    let mut sk = base;
    let elapsed = if old.action == g.action && old.action_total == g.action_total && old.action_time >= g.action_time {
        g.action_total - (old.action_time + (g.action_time - old.action_time) * alpha)
    } else {
        g.action_total - g.action_time
    };
    if matches!(g.action, Action::Light | Action::Heavy) {
        let a = combat::attack(g.action, g.combo, g.progress.weapon == 1);
        let pose = attack_pose(g.action, g.combo, g.progress.weapon == 1, elapsed);
        let entrance = if g.action == Action::Light && g.combo > 0 { 1. } else { elapsed / 0.065 };
        let exit = (a.duration - elapsed) / 0.14;
        sk = blend(&base, &pose, entrance.min(exit).min(1.));
    } else {
        let (name, clock, duration, upper) = match g.action {
            Action::Dodge => ("QUATERNIUS/Roll", elapsed / 0.55 * 1.33, 0.55, false),
            Action::Parry => ("QUATERNIUS/Shield_OneShot", elapsed / 0.48 * 0.833, 0.48, false),
            Action::Heal => ("QUATERNIUS/Consume", elapsed / 0.95 * 1.333, 0.95, true),
            Action::Stagger => ("QUATERNIUS/Hit_Chest", elapsed.min(0.332), g.action_total, false),
            Action::Interact => ("QUATERNIUS/Interact", elapsed / 0.55 * 1.7, 0.55, true),
            Action::Equip => ("QUATERNIUS/Sword_Block", elapsed / 0.45 * 1.233, 0.45, true),
            _ => ("", 0., 0., false),
        };
        if !name.is_empty() {
            let mut p = sample(name, clock);
            if g.action == Action::Dodge {
                p.transform(Quat::from_rotation_y(m.dodge_yaw - g.facing), Vec3::ZERO);
            }
            let weight = (elapsed / 0.055).min((duration - elapsed) / 0.09).clamp(0., 1.);
            sk = clips::blend(rig(), &base, &p, weight, upper);
        } else if !m.grounded && m.air_time > 0.04 {
            let air = sample("QUATERNIUS/Jump_Loop", m.air_time);
            let launch = sample("QUATERNIUS/Jump_Start", 0.067 + m.air_time * 2.0);
            let p = blend(&launch, &air, ((m.air_time - 0.12) / 0.12).clamp(0., 1.));
            sk = blend(&base, &p, (m.air_time / 0.13).min(1.));
        } else if m.landing > 0.0 {
            let p = sample("QUATERNIUS/Jump_Land", (0.24 - m.landing) / 0.24 * 0.8);
            sk = blend(&base, &p, m.landing / 0.24);
        }
    }
    if g.progress.weapon == 1 && !matches!(g.action, Action::Heal | Action::Interact | Action::Dodge) {
        support_blade(&mut sk);
    }
    if m.reaction > 0.0 && g.action != Action::Dodge {
        let p = sample("QUATERNIUS/Hit_Chest", (0.39 - m.reaction).clamp(0.07, 0.332));
        sk = blend(&sk, &p, (m.reaction / 0.32) * if g.guard { 0.3 } else { 0.65 });
        sk = recoil(&sk, &m, g.facing);
    }
    sk
}

pub fn enemy(e: &Enemy, time: f32) -> Skel {
    let base = locomotion(&e.motion, e.facing, time);
    if e.hp <= 0. {
        return blend(&base, &sample("QUATERNIUS/Death01", (e.death_time * 1.35).min(2.399)), (e.death_time / 0.08).min(1.));
    }
    let mut pose = base;
    if matches!(e.intent, Intent::Windup | Intent::Recover) && e.motion.reaction <= 0.0 {
        // Both sides of the state boundary sample the SAME contact pose. No delayed visual hit.
        let spec = combat::attack(Action::Heavy, 0, false);
        let t = if e.intent == Intent::Windup {
            let u = 1.0 - e.timer / e.duration.max(0.01);
            if e.timer > 0.13 { (spec.source_hit - 0.10) * u } else { spec.source_hit - e.timer / 0.13 * 0.10 }
        } else {
            spec.source_hit + (1.0 - e.timer / e.duration.max(0.01)) * (spec.source_end - spec.source_hit)
        };
        let name =
            if e.kind == Kind::Cantor || (e.boss() && e.attack == 2) { "QUATERNIUS/Spell_Simple_Shoot" } else { spec.clip };
        pose = sample(name, t);
        if e.kind == Kind::Lancer {
            // Hold the long spear along the forearm instead of using a sword's blade channel.
            pose.weapon = (pose.hand[1] - pose.elbow[1]).normalize_or(Vec3::Z);
        }
        let weight = if e.intent == Intent::Windup { (e.duration - e.timer) / 0.12 } else { e.timer / 0.18 };
        pose = blend(&base, &pose, weight.min(1.));
    }
    if e.intent == Intent::Stagger {
        let p = sample("QUATERNIUS/Idle_Shield_Break", (e.duration - e.timer).min(1.066));
        pose = blend(&base, &p, (e.timer / 0.18).min(1.));
    }
    if e.motion.reaction > 0. {
        let p = sample("QUATERNIUS/Hit_Chest", (0.39 - e.motion.reaction).clamp(0.07, 0.332));
        pose = blend(&pose, &p, e.motion.reaction / 0.32);
        pose = recoil(&pose, &e.motion, e.facing);
    }
    pose
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipped_motion_palette_exists_and_attacks_move_the_whole_body() {
        for name in [
            "QUATERNIUS/Sword_Idle",
            "QUATERNIUS/Walk_Loop",
            "MESH2MOTION/Jog",
            "MESH2MOTION/Walk_Backwards",
            "QUATERNIUS/Idle_Shield_Loop",
            "QUATERNIUS/Death01",
            "QUATERNIUS/Roll",
            "QUATERNIUS/Shield_OneShot",
            "QUATERNIUS/Consume",
            "QUATERNIUS/Hit_Chest",
            "QUATERNIUS/Interact",
            "QUATERNIUS/Sword_Block",
            "QUATERNIUS/Jump_Loop",
            "QUATERNIUS/Jump_Start",
            "QUATERNIUS/Jump_Land",
            "QUATERNIUS/Idle_Shield_Break",
            "QUATERNIUS/Spell_Simple_Shoot",
        ] {
            assert_ne!(clips::find_cached(name), 0, "missing embedded clip {name}");
        }
        for (action, combo) in [(Action::Light, 0), (Action::Light, 1), (Action::Light, 2), (Action::Heavy, 0)] {
            let a = combat::attack(action, combo, false);
            assert_ne!(clips::find_cached(a.clip), 0);
            if let Some(rec) = a.recovery {
                assert_ne!(clips::find_cached(rec), 0);
            }
            let ready = attack_pose(action, combo, false, 0.);
            let contact = attack_pose(action, combo, false, a.contact);
            assert!(ready.chest_rot.angle_between(contact.chest_rot).abs() > 0.3, "attack must rotate the torso");
            assert!(ready.hand[1].distance(contact.hand[1]) > 0.10, "weapon cannot swing independently of the arm");
            for n in 0..=60 {
                let p = attack_pose(action, combo, false, a.duration * n as f32 / 60.);
                assert!(p.head.is_finite() && p.weapon.is_finite());
                for i in 0..2 {
                    assert!((p.shoulder[i].distance(p.elbow[i]) - rig().arm_length * 0.5).abs() < 0.01);
                    assert!((p.elbow[i].distance(p.hand[i]) - rig().arm_length * 0.5).abs() < 0.01);
                }
            }
        }
    }
    #[test]
    fn attacks_settle_into_idle_without_a_last_frame_pose_pop() {
        let sim = pav_core::Sim::new("dented/play", 1).unwrap();
        let original = sim.play().unwrap().get::<Dented>().unwrap();
        for (action, combo) in [(Action::Light, 0), (Action::Light, 1), (Action::Light, 2), (Action::Heavy, 0)] {
            let mut g = original.clone();
            g.motion.grounded = true;
            g.time = 1.0;
            let ready = player(&g, &g, 1.0);
            g.action = action;
            g.combo = combo;
            g.action_total = combat::attack(action, combo, false).duration;
            g.action_time = 1.0 / 240.0;
            let settled = player(&g, &g, 1.0);
            assert!(settled.chest.distance(ready.chest) < 0.02);
            assert!(settled.hand[1].distance(ready.hand[1]) < 0.03);
        }
    }
    #[test]
    fn greatblade_stays_in_both_hands_through_the_whole_chain() {
        for (action, combo) in [(Action::Light, 0), (Action::Light, 1), (Action::Light, 2), (Action::Heavy, 0)] {
            let a = combat::attack(action, combo, true);
            for frame in 0..=90 {
                let sk = attack_pose(action, combo, true, a.duration * frame as f32 / 90.);
                let grip = sk.hand[1] - sk.weapon * 0.16;
                assert!(sk.hand[0].distance(grip) < 0.015, "support hand left the blade: {action:?}/{combo}/{frame}");
                for i in 0..2 {
                    assert!((sk.shoulder[i].distance(sk.elbow[i]) - rig().arm_length * 0.5).abs() < 0.01);
                    assert!((sk.elbow[i].distance(sk.hand[i]) - rig().arm_length * 0.5).abs() < 0.01);
                }
            }
        }
    }
}
