use pav_core::{
    InputFrame, Sim, SimState,
    games::{
        Game, Outcome,
        dented::{Action, Dented, Mode, Progress},
    },
    input::buttons as b,
};
fn game(s: &Sim) -> &Dented {
    s.play().unwrap().get::<Dented>().unwrap()
}
fn game_mut(s: &mut Sim) -> &mut Dented {
    s.state.play.as_mut().unwrap().get_mut::<Dented>().unwrap()
}
fn tap(s: &mut Sim, button: u32) {
    s.step(&InputFrame { pressed: button, ..Default::default() });
}
#[test]
fn title_enters_demo_at_ten_seconds_and_every_input_exits() {
    for input in [
        InputFrame { pressed: b::ACTIVITY, ..Default::default() },
        InputFrame { held: b::FOCUS, ..Default::default() },
        InputFrame { move_dir: glam::Vec2::X, ..Default::default() },
    ] {
        let mut s = Sim::new("dented", 3).unwrap();
        for _ in 0..599 {
            s.step(&InputFrame::default());
        }
        assert!(!game(&s).demo);
        s.step(&InputFrame::default());
        assert!(game(&s).demo);
        s.step(&input);
        assert_eq!(game(&s).mode, Mode::Title);
        assert!(!game(&s).demo);
        assert_eq!(game(&s).idle, 0.);
    }
}
#[test]
fn bot_completes_the_whole_pilgrimage() {
    for seed in 1..=3 {
        let mut s = Sim::new("dented/play", seed).unwrap();
        let (outcome, ticks) = s.run_bot(60 * 300);
        assert_eq!(outcome, Outcome::Won, "seed {seed}, tick {ticks}: {}", game(&s).status(&s));
        assert!(game(&s).progress.shortcut);
        assert!(game(&s).progress.relic);
        assert!(game(&s).metrics.phase_two);
    }
}
#[test]
fn save_validation_rejects_unknown_formats_and_bounds_values() {
    let p = Progress { version: 12, ..Default::default() };
    assert!(p.validate().is_none());
    let p = Progress { checkpoint: 90, vigor: 900, weapon: 9, ..Default::default() }.validate().unwrap();
    assert_eq!(p.checkpoint, 63);
    assert_eq!(p.vigor, 5);
    assert_eq!(p.weapon, 0);
}
#[test]
fn pause_freezes_combat_and_navigation_snaps() {
    let mut s = Sim::new("dented/play", 1).unwrap();
    tap(&mut s, b::MENU);
    let t = game(&s).time;
    for _ in 0..120 {
        s.step(&InputFrame::default());
    }
    assert_eq!(t, game(&s).time);
    tap(&mut s, b::DOWN);
    assert_eq!(game(&s).menu, 1);
    tap(&mut s, b::UP);
    assert_eq!(game(&s).menu, 0);
    tap(&mut s, b::ACCEPT);
    assert_eq!(game(&s).mode, Mode::Playing);
}
#[test]
fn stamina_prevents_attack_and_roll_spam() {
    let mut s = Sim::new("dented/play", 1).unwrap();
    game_mut(&mut s).stamina = 0.;
    game_mut(&mut s).regen_delay = 5.;
    tap(&mut s, b::PRIMARY | b::DODGE);
    assert_eq!(game(&s).action, Action::Idle);
    assert_eq!(game(&s).metrics.lights, 0);
    assert_eq!(game(&s).metrics.dodges, 0);
}
#[test]
fn snapshot_serialization_and_input_replay_are_exact() {
    let mut s = Sim::new("dented/play", 4).unwrap();
    for _ in 0..180 {
        let f = s.bot_input();
        s.step(&f);
    }
    let mut bytes = Vec::new();
    ciborium::into_writer(&s.snapshot(), &mut bytes).unwrap();
    let state: SimState = ciborium::from_reader(bytes.as_slice()).unwrap();
    let mut copy = Sim::new("dented/play", 4).unwrap();
    copy.restore(state);
    for _ in 0..180 {
        let f = s.bot_input();
        s.step(&f);
        copy.step(&f);
    }
    assert_eq!(s.state_hash(), copy.state_hash());
    let mut replay = Sim::new("dented/play", 4).unwrap();
    for f in s.recording.iter() {
        replay.step(f);
    }
    assert_eq!(s.state_hash(), replay.state_hash());
}

fn duel() -> Sim {
    use glam::Vec3;
    use pav_core::games::dented::Intent;
    let mut s = Sim::new("dented/play", 9).unwrap();
    let pid = s.state.player.unwrap();
    s.set_position(pid, Vec3::new(0., 0., 17.));
    let ids: Vec<_> = game(&s).enemies.iter().map(|e| e.id).collect();
    for (i, id) in ids.iter().enumerate() {
        s.set_position(*id, if i == 0 { Vec3::new(0., 0., 18.5) } else { Vec3::new(18., 0., -24.) });
    }
    let g = game_mut(&mut s);
    g.invuln = 0.;
    g.facing = 0.;
    for (i, e) in g.enemies.iter_mut().enumerate() {
        if i == 0 {
            e.pos = Vec3::new(0., 0., 18.5);
            e.intent = Intent::Windup;
            e.timer = 0.01;
            e.duration = 0.8;
            e.facing = std::f32::consts::PI;
        } else {
            e.hp = 0.;
        }
    }
    s
}
#[test]
fn timed_parry_guard_and_dodge_have_distinct_real_effects() {
    let mut hit = duel();
    hit.step(&InputFrame::default());
    assert!(game(&hit).hp < 120.);
    let mut block = duel();
    block.step(&InputFrame { held: b::FOCUS, ..Default::default() });
    assert!(game(&block).hp > game(&hit).hp);
    assert!(game(&block).hp < 120.);
    assert_eq!(game(&block).metrics.blocks, 1);
    assert!(game(&block).stamina < 100.);
    let mut parry = duel();
    tap(&mut parry, b::SKILL4);
    assert_eq!(game(&parry).hp, 120.);
    assert_eq!(game(&parry).metrics.parries, 1);
    assert_eq!(game(&parry).enemies[0].intent, pav_core::games::dented::Intent::Stagger);
    let mut dodge = duel();
    tap(&mut dodge, b::DODGE);
    assert_eq!(game(&dodge).hp, 120.);
    assert_eq!(game(&dodge).metrics.dodges, 1);
}
#[test]
fn death_cache_is_recovered_and_survives_checkpoint_reset() {
    let mut s = duel();
    game_mut(&mut s).hp = 1.;
    game_mut(&mut s).progress.echoes = 125;
    s.step(&InputFrame::default());
    assert_eq!(game(&s).mode, Mode::Dead);
    assert_eq!(game(&s).progress.echoes, 0);
    let (at, n) = game(&s).blood.unwrap();
    assert_eq!(n, 125);
    assert_eq!(game(&s).progress.blood, Some((at, n)));
    tap(&mut s, b::ACCEPT);
    assert_eq!(game(&s).mode, Mode::Playing);
    assert_eq!(game(&s).blood, Some((at, n)));
    s.set_position(s.state.player.unwrap(), at);
    s.step(&InputFrame::default());
    assert!(game(&s).blood.is_none());
    assert!(game(&s).progress.blood.is_none());
    assert_eq!(game(&s).progress.echoes, 125);
}
#[test]
fn resting_refills_and_upgrades_spend_currency_once() {
    let mut s = Sim::new("dented/play", 2).unwrap();
    game_mut(&mut s).hp = 20.;
    game_mut(&mut s).flasks = 0;
    game_mut(&mut s).progress.echoes = 200;
    game_mut(&mut s).enemies[0].hp = 0.;
    tap(&mut s, b::INTERACT);
    assert_eq!(game(&s).mode, Mode::Rest);
    assert_eq!(game(&s).flasks, 3);
    assert_eq!(game(&s).hp, 120.);
    assert!(game(&s).enemies[0].hp > 0.);
    tap(&mut s, b::DOWN);
    tap(&mut s, b::ACCEPT);
    assert_eq!(game(&s).progress.vigor, 1);
    assert_eq!(game(&s).progress.echoes, 120);
    assert_eq!(game(&s).hp, 140.);
    tap(&mut s, b::ACCEPT);
    assert_eq!(game(&s).progress.vigor, 1);
    assert_eq!(game(&s).progress.echoes, 120);
}
#[test]
fn heavy_damage_occurs_once_and_heal_has_commitment() {
    use pav_core::games::dented::Intent;
    let mut s = duel();
    game_mut(&mut s).enemies[0].intent = Intent::Recover;
    game_mut(&mut s).enemies[0].timer = 99.;
    game_mut(&mut s).enemies[0].hp = 1000.;
    tap(&mut s, b::SECONDARY);
    for _ in 0..58 {
        s.step(&InputFrame::default());
    }
    assert!((game(&s).enemies[0].hp - 952.).abs() < 0.01);
    game_mut(&mut s).hp = 20.;
    tap(&mut s, b::POTION);
    assert_eq!(game(&s).hp, 20.);
    assert_eq!(game(&s).flasks, 2);
    for _ in 0..50 {
        s.step(&InputFrame::default());
    }
    assert_eq!(game(&s).hp, 90.);
}
#[test]
fn thin_gates_do_not_obstruct_their_interaction_approach() {
    let s = Sim::new("dented/play", 1).unwrap();
    let nav = pav_core::nav::NavGrid::for_room(&s).unwrap();
    assert!(nav.walkable(glam::Vec3::new(0., 0., 5.)));
    assert!(!nav.walkable(glam::Vec3::new(0., 0., 3.)));
}
#[test]
fn attract_repeats_actual_runs_without_changing_the_saved_pilgrimage() {
    let mut s = Sim::new("dented", 11).unwrap();
    let personal = Progress { started: true, echoes: 123, vigor: 2, shortcut: true, ..Default::default() };
    game_mut(&mut s).progress = personal.clone();
    game_mut(&mut s).time = 57.0;
    game_mut(&mut s).metrics.lights = 9;
    for _ in 0..60 * 600 {
        s.step(&InputFrame::default());
        if game(&s).metrics.cycles >= 2 {
            break;
        }
    }
    assert!(game(&s).demo);
    assert!(game(&s).metrics.cycles >= 2, "{}", game(&s).status(&s));
    tap(&mut s, b::ACTIVITY);
    assert_eq!(game(&s).mode, Mode::Title);
    assert_eq!(serde_json::to_value(&game(&s).progress).unwrap(), serde_json::to_value(personal).unwrap());
    assert_eq!(game(&s).time, 57.0);
    assert_eq!(game(&s).metrics.lights, 9);
}

#[test]
fn bot_does_not_oscillate_between_a_gate_and_an_unseen_enemy() {
    use glam::Vec3;
    let mut s = Sim::new("dented/play", 1).unwrap();
    let at = Vec3::new(3.07, 0.02, 6.98);
    s.set_position(s.state.player.unwrap(), at);
    let g = game_mut(&mut s);
    g.player_pos = at;
    g.progress.chime = true;
    g.progress.relic = true;
    for e in g.enemies.iter_mut().take(5) {
        e.hp = 0.0;
    }
    // This position straddles the northern lancer's previous 10 m perception threshold.
    // It must reach the gate rather than switch between two opposing steering directions.
    for _ in 0..1800 {
        let f = s.bot_input();
        s.step(&f);
        if game(&s).progress.gate {
            break;
        }
    }
    assert!(game(&s).progress.gate, "{}", game(&s).status(&s));
    let mut bytes = Vec::new();
    ciborium::into_writer(&s.snapshot(), &mut bytes).unwrap();
    let mut copy = Sim::new("dented/play", 1).unwrap();
    copy.restore(ciborium::from_reader(bytes.as_slice()).unwrap());
    assert_eq!(s.bot_input(), copy.bot_input(), "bot decisions must not depend on a warm nav cache");
}

#[test]
fn pause_holds_airborne_position_and_restart_discards_buffered_actions() {
    let mut s = Sim::new("dented/play", 7).unwrap();
    tap(&mut s, b::JUMP);
    for _ in 0..8 {
        s.step(&InputFrame::default());
    }
    tap(&mut s, b::MENU);
    let paused = s.feet(s.state.player.unwrap()).unwrap();
    for _ in 0..120 {
        s.step(&InputFrame::default());
    }
    assert!(s.feet(s.state.player.unwrap()).unwrap().distance(paused) < 0.001);
    let g = game_mut(&mut s);
    g.buffer = b::PRIMARY;
    g.buffer_time = 0.2;
    g.set_mode(Mode::Dead);
    tap(&mut s, b::ACCEPT);
    s.step(&InputFrame::default());
    assert_eq!(game(&s).action, Action::Idle);
    assert_eq!(game(&s).buffer, 0);
}
