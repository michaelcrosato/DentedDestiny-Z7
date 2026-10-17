//! Deterministic visual review of the actual Dented renderer and combat, without a server.
//! cargo run -p pav_tools --example dented_review -- /tmp/dented-review [inspect|actions|great|combat]
use anyhow::Result;
use glam::{Vec2, Vec3};
use pav_core::{
    InputFrame, clips,
    games::dented::{Action, Dented, Intent, Mode},
    games::dented_motion as motion,
    input::buttons as b,
};
use pav_tools::Session;
use std::path::PathBuf;
fn game(s: &Session) -> &Dented {
    s.sim.play().unwrap().get::<Dented>().unwrap()
}
fn game_mut(s: &mut Session) -> &mut Dented {
    s.sim.state.play.as_mut().unwrap().get_mut::<Dented>().unwrap()
}
fn main() -> Result<()> {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or("/tmp/dented-review".into()));
    let mode = std::env::args().nth(2).unwrap_or("combat".into());
    std::fs::create_dir_all(&out)?;
    if mode == "inspect" {
        for name in ["Sword_Regular_A", "Sword_Regular_B", "Sword_Regular_C", "Sword_Attack"] {
            let id = clips::find_cached(&format!("QUATERNIUS/{name}"));
            println!("{name}: maximum forward reach at {:?}", clips::strike_time(id));
        }
        return Ok(());
    }
    let mut s = Session::new("dented/play", 9)?;
    let pid = s.sim.state.player.unwrap();
    s.sim.set_position(pid, Vec3::new(0., 0., 17.));
    let ids: Vec<_> = game(&s).enemies.iter().map(|e| e.id).collect();
    for (i, id) in ids.iter().enumerate() {
        s.sim.set_position(*id, Vec3::new(if i == 0 { 0. } else { 18. }, 0., if i == 0 { 14.9 } else { -24. }));
    }
    {
        let g = game_mut(&mut s);
        g.player_pos = Vec3::new(0., 0., 17.);
        g.safe_pos = g.player_pos;
        g.facing = std::f32::consts::PI;
        g.invuln = 999.;
        g.hp = 45.;
        g.progress.weapon = u8::from(mode == "great");
        for (i, e) in g.enemies.iter_mut().enumerate() {
            e.hp = if i == 0 && mode == "combat" { 10000. } else { 0. };
            e.death_time = 10.;
            e.sight = 0.;
            e.intent = Intent::Idle;
            e.pos = Vec3::new(0., 0., 14.9);
            e.facing = 0.;
        }
    }
    s.sync_camera();
    s.camera.params.yaw = 65.;
    s.camera.params.tilt = 18.;
    s.camera.params.distance = 5.7;
    let (w, h) = (600, 460);
    if mode == "actions" || mode == "great" {
        let actions = [
            ("idle", Action::Idle),
            ("run", Action::Idle),
            ("light-a", Action::Light),
            ("light-b", Action::Light),
            ("finisher", Action::Light),
            ("heavy", Action::Heavy),
            ("roll", Action::Dodge),
            ("guard", Action::Idle),
            ("parry", Action::Parry),
            ("flask", Action::Heal),
            ("jump", Action::Idle),
            ("land", Action::Idle),
            ("interact", Action::Interact),
            ("equip", Action::Equip),
            ("hurt", Action::Stagger),
            ("death", Action::Idle),
        ];
        for (name, action) in actions {
            let mut frames = Vec::new();
            for n in 0..6 {
                let u = n as f32 / 6.;
                let g = game_mut(&mut s);
                g.mode = if name == "death" { Mode::Dead } else { Mode::Playing };
                g.mode_time = u * 2.5;
                g.time = u * 2.;
                g.motion = motion::Motion { grounded: true, ..Default::default() };
                g.guard = false;
                g.action = action;
                g.combo = match name {
                    "light-b" => 1,
                    "finisher" => 2,
                    _ => 0,
                };
                g.action_total = match action {
                    Action::Light | Action::Heavy => motion::attack(action, g.combo, g.progress.weapon == 1).duration,
                    Action::Dodge => 0.55,
                    Action::Heal => 0.95,
                    Action::Parry => 0.48,
                    Action::Stagger => 0.32,
                    Action::Interact => 0.55,
                    Action::Equip => 0.45,
                    _ => 1.,
                };
                g.action_time = g.action_total * (1. - u);
                g.motion.dodge_yaw = g.facing;
                if name == "run" {
                    g.motion.speed = 4.6;
                    g.motion.stride = u;
                    g.motion.velocity = Vec3::NEG_Z * 4.6;
                }
                if name == "guard" {
                    g.guard = true;
                    g.motion.guard = 1.;
                }
                if name == "jump" {
                    g.motion.grounded = false;
                    g.motion.air_time = u * 0.5 + 0.05;
                }
                if name == "land" {
                    g.motion.landing = (1. - u) * 0.24;
                }
                frames.push(s.render(w, h)?);
            }
            let (tw, th, px) = pav_render::capture::tile_frames(&frames, w, h, 3);
            pav_render::capture::save_png(&out.join(format!("{name}.png")), tw, th, &px)?;
            println!("{}", out.join(format!("{name}.png")).display());
        }
    } else {
        let mut frames = Vec::new();
        let mut contact = Vec::new();
        let mut audio_events = Vec::new();
        for tick in 0..180 {
            let g = game(&s);
            let enemy = g.enemies[0].pos;
            let elapsed = g.action_total - g.action_time;
            let press = if tick == 0
                || (matches!(g.action, Action::Light) && g.motion.combo_next < 3 && elapsed > 0.23 && g.metrics.lights < 3)
            {
                b::PRIMARY
            } else if tick == 115 {
                b::SECONDARY
            } else {
                0
            };
            s.sim.step(&InputFrame { pressed: press, aim: Some(enemy), move_dir: Vec2::ZERO, ..Default::default() });
            let events = s.sim.drain_events();
            s.gpu()?.builder.add_events(&events);
            audio_events.extend(events.into_iter().map(|e| (tick as f32 / 60., e)));
            if game(&s).impacts.last().is_some_and(|i| i.age == 0.) {
                contact.push((tick, game(&s).combo, game(&s).player_pos, game(&s).enemies[0].pos));
            }
            if tick % 3 == 0 {
                let px = s.render(w, h)?;
                pav_render::capture::save_png(&out.join(format!("frame-{:03}.png", tick / 3)), w, h, &px)?;
                if tick % 9 == 0 {
                    frames.push(px);
                }
            }
        }
        let (tw, th, px) = pav_render::capture::tile_frames(&frames, w, h, 4);
        pav_render::capture::save_png(&out.join("combat-sheet.png"), tw, th, &px)?;
        std::fs::write(out.join("contacts.json"), serde_json::to_string_pretty(&contact)?)?;
        let listener = pav_audio::Listener { pos: Vec3::new(0., 1., 16.), right: Vec3::X };
        let pcm = pav_audio::render_events(&audio_events, 3.0, 44100., &listener);
        pav_audio::write_wav(&out.join("combat.wav"), &pcm, 44100)?;
        let peak = pcm.iter().fold(0.0_f32, |peak, x| peak.max(x.abs()));
        std::fs::write(
            out.join("audio.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "seconds":3.0,"events":audio_events.len(),"peak":peak,
                "rms":(pcm.iter().map(|x|x*x).sum::<f32>()/pcm.len() as f32).sqrt()
            }))?,
        )?;
        println!("{}", out.join("combat-sheet.png").display());
    }
    Ok(())
}
