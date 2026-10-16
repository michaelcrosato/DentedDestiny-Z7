//! Dented Destiny. Rules are deterministic and independent of window, renderer and audio.
//! Add encounters and tune combat in dented.toml; all changing state is snapshot-safe.
use super::{Game, GameBox, Outcome};
use crate::{
    InputFrame, Sim,
    color::Color,
    entity::{BodyKind, EntityId, Spawn},
    frame::SimEvent,
    input::buttons as b,
    nav::NavGrid,
    puppet::{BodyPlan, PuppetDef},
    room::RoomDef,
    shape::{Shape, Visual},
};
use anyhow::{Result, anyhow};
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

pub const SHRINES: [Vec3; 2] = [Vec3::new(0., 0., 23.), Vec3::new(0., 0., -6.)];
pub const RELIQUARY: Vec3 = Vec3::new(-13., 0., 10.);
pub const CHIME: Vec3 = Vec3::new(13., 0., 0.);
pub const LEVER: Vec3 = Vec3::new(-12., 0., -6.);
pub const GATE: Vec3 = Vec3::new(0., 0., 3.);
pub const FOG: Vec3 = Vec3::new(0., 0., -12.);
pub const BELL: Vec3 = Vec3::new(0., 0., -24.);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Title,
    Playing,
    Pause,
    Rest,
    Settings,
    Guide,
    Journal,
    Dead,
    Victory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Idle,
    Light,
    Heavy,
    Dodge,
    Heal,
    Parry,
    Stagger,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Warden,
    Lancer,
    Cantor,
    Guardian,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Idle,
    Chase,
    Windup,
    Recover,
    Stagger,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub version: u32,
    pub started: bool,
    pub checkpoint: usize,
    pub echoes: u32,
    pub vigor: u32,
    pub edge: u32,
    pub chime: bool,
    pub gate: bool,
    pub shortcut: bool,
    pub relic: bool,
    pub weapon: u8,
    pub guardian: bool,
    pub lore: u8,
    pub deaths: u32,
    pub kills: u32,
    pub best_seconds: f32,
    pub blood: Option<(Vec3, u32)>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            version: 1,
            started: false,
            checkpoint: 0,
            echoes: 0,
            vigor: 0,
            edge: 0,
            chime: false,
            gate: false,
            shortcut: false,
            relic: false,
            weapon: 0,
            guardian: false,
            lore: 0,
            deaths: 0,
            kills: 0,
            best_seconds: 0.,
            blood: None,
        }
    }
}
impl Progress {
    pub fn validate(mut self) -> Option<Self> {
        if self.version != 1 || !self.best_seconds.is_finite() {
            return None;
        }
        self.checkpoint = self.checkpoint.min(63);
        self.vigor = self.vigor.min(5);
        self.edge = self.edge.min(5);
        self.weapon = if self.relic { self.weapon.min(1) } else { 0 };
        self.echoes = self.echoes.min(999_999);
        Some(self)
    }
    pub fn max_hp(&self) -> f32 {
        120. + self.vigor as f32 * 20.
    }
    pub fn max_stamina(&self) -> f32 {
        if self.relic { 120. } else { 100. }
    }
    pub fn weapon_name(&self) -> &'static str {
        if self.weapon == 0 { "Pilgrim's edge" } else { "Vesper greatblade" }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    pub assist: bool,
    pub shake: bool,
    pub sound: bool,
    pub contrast: bool,
    pub camera: u8,
}
impl Default for Options {
    fn default() -> Self {
        Self { assist: false, shake: true, sound: true, contrast: false, camera: 0 }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemySpec {
    pub kind: Kind,
    pub at: Vec3,
    pub hp: f32,
    pub reward: u32,
    pub sight: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    pub shrines: Vec<Vec3>,
    pub reliquary: Vec3,
    pub chime: Vec3,
    pub lever: Vec3,
    pub gate: Vec3,
    pub fog: Vec3,
    pub bell: Vec3,
    pub speed: f32,
    pub light_damage: f32,
    pub heavy_damage: f32,
    pub regen: f32,
    pub encounters: Vec<EnemySpec>,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            shrines: SHRINES.to_vec(),
            reliquary: RELIQUARY,
            chime: CHIME,
            lever: LEVER,
            gate: GATE,
            fog: FOG,
            bell: BELL,
            speed: 4.6,
            light_damage: 25.,
            heavy_damage: 48.,
            regen: 30.,
            encounters: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enemy {
    pub id: EntityId,
    pub kind: Kind,
    pub home: Vec3,
    pub pos: Vec3,
    pub hp: f32,
    pub max_hp: f32,
    pub reward: u32,
    pub sight: f32,
    pub facing: f32,
    pub intent: Intent,
    pub timer: f32,
    pub duration: f32,
    pub attack: u8,
    pub cycle: u32,
    pub phase: u8,
    pub poise: f32,
    pub flash: f32,
}
impl Enemy {
    pub fn name(&self) -> &'static str {
        match self.kind {
            Kind::Warden => "Hollow retainer",
            Kind::Lancer => "Thorn sentinel",
            Kind::Cantor => "Ash cantor",
            Kind::Guardian => "VESPER, THE UNRUNG",
        }
    }
    pub fn boss(&self) -> bool {
        self.kind == Kind::Guardian
    }
    pub fn reach(&self) -> f32 {
        match self.kind {
            Kind::Lancer => 3.5,
            Kind::Guardian => {
                if self.attack == 1 {
                    7.5
                } else {
                    4.5
                }
            }
            Kind::Cantor => 12.,
            _ => 2.4,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bolt {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub lights: u32,
    pub heavies: u32,
    pub dodges: u32,
    pub blocks: u32,
    pub parries: u32,
    pub heals: u32,
    pub recovered: u32,
    pub rests: u32,
    pub phase_two: bool,
    pub cycles: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoBackup {
    pub progress: Progress,
    pub time: f32,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dented {
    pub rules: Rules,
    pub progress: Progress,
    pub options: Options,
    pub mode: Mode,
    pub return_mode: Mode,
    pub menu: usize,
    pub idle: f32,
    pub time: f32,
    pub mode_time: f32,
    pub demo: bool,
    pub demo_backup: Option<DemoBackup>,
    pub lab: bool,
    pub hp: f32,
    pub stamina: f32,
    pub flasks: u32,
    pub player_pos: Vec3,
    pub facing: f32,
    pub action: Action,
    pub action_time: f32,
    pub action_total: f32,
    pub hit_done: bool,
    pub combo: u8,
    pub regen_delay: f32,
    pub invuln: f32,
    pub flash: f32,
    pub guard: bool,
    pub locked: Option<usize>,
    pub enemies: Vec<Enemy>,
    pub bolts: Vec<Bolt>,
    pub blood: Option<(Vec3, u32)>,
    pub boss_active: bool,
    pub scenery: Vec<EntityId>,
    pub gate_entity: Option<EntityId>,
    pub shortcut_entity: Option<EntityId>,
    pub fog_entity: Option<EntityId>,
    pub prompt: String,
    pub notice: String,
    pub notice_time: f32,
    pub area: String,
    pub area_time: f32,
    pub metrics: Metrics,
    pub save_serial: u64,
    pub buffer: u32,
    pub buffer_time: f32,
    #[serde(skip)]
    nav: Option<Arc<NavGrid>>,
}

pub fn build(sim: &mut Sim, rest: Option<&str>) -> Result<()> {
    let text = super::level_text("dented", include_str!("dented.toml"));
    let def = RoomDef::parse(&text).map_err(|e| anyhow!("dented.toml: {e}"))?;
    let rules: Rules = toml::Value::Table(def.game.clone()).try_into().map_err(|e| anyhow!("dented rules: {e}"))?;
    if rules.shrines.is_empty() || rules.shrines.iter().any(|p| !p.is_finite()) {
        return Err(anyhow!("dented [game].shrines needs finite checkpoint positions"));
    }
    let start = rules.shrines[0];
    crate::scenes::build_standalone_room(sim, "dented", def);
    sim.config.movement.speed = rules.speed;
    sim.config.movement.ledge_grab = false;
    let mut g = Dented {
        rules,
        progress: Progress::default(),
        options: Options::default(),
        mode: Mode::Title,
        return_mode: Mode::Title,
        menu: 0,
        idle: 0.,
        time: 0.,
        mode_time: 0.,
        demo: false,
        demo_backup: None,
        lab: rest == Some("lab"),
        hp: 120.,
        stamina: 100.,
        flasks: 3,
        player_pos: start,
        facing: std::f32::consts::PI,
        action: Action::Idle,
        action_time: 0.,
        action_total: 0.,
        hit_done: false,
        combo: 0,
        regen_delay: 0.,
        invuln: 0.,
        flash: 0.,
        guard: false,
        locked: None,
        enemies: Vec::new(),
        bolts: Vec::new(),
        blood: None,
        boss_active: false,
        scenery: Vec::new(),
        gate_entity: None,
        shortcut_entity: None,
        fog_entity: None,
        prompt: String::new(),
        notice: String::new(),
        notice_time: 0.,
        area: "PILGRIM'S APPROACH".into(),
        area_time: 5.,
        metrics: Metrics::default(),
        save_serial: 0,
        buffer: 0,
        buffer_time: 0.,
        nav: None,
    };
    for z in [-23., -15., -6., 4., 14., 24.] {
        for x in [-7., 7.] {
            g.scenery.push(
                sim.spawn(
                    Spawn::new("abbey pillar collider", Vec3::new(x, 1.5, z))
                        .body(BodyKind::Fixed)
                        .visual(Visual::new(Shape::Cylinder { radius: 0.53, half_height: 1.5 }, Color::hex("#83877b"))),
                ),
            );
        }
    }
    g.reset_encounters(sim);
    g.sync_doors(sim);
    if rest == Some("play") || rest == Some("lab") {
        g.begin(sim, false);
    }
    sim.state.play = Some(GameBox::new(g));
    Ok(())
}
fn flat(v: Vec3) -> f32 {
    Vec2::new(v.x, v.z).length()
}
fn dir(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.z).normalize_or_zero()
}
fn angle(v: Vec3) -> f32 {
    v.x.atan2(v.z)
}
fn forward(yaw: f32) -> Vec3 {
    Vec3::new(yaw.sin(), 0., yaw.cos())
}
fn event_hit(events: &mut Vec<SimEvent>, pos: Vec3, power: f32) {
    events.push(SimEvent::Strike { pos: pos + Vec3::Y, power, element: 0, crit: power > 40. });
}

impl Dented {
    pub fn set_mode(&mut self, m: Mode) {
        self.mode = m;
        self.menu = 0;
        self.mode_time = 0.;
        self.idle = 0.;
    }
    pub fn announce(&mut self, s: impl Into<String>) {
        self.notice = s.into();
        self.notice_time = 4.;
    }
    fn save(&mut self) {
        if !self.demo {
            self.save_serial += 1;
        }
    }
    fn door(sim: &mut Sim, name: &str, p: Vec3, half: Vec3) -> EntityId {
        sim.spawn(
            Spawn::new(name, p + Vec3::Y * half.y)
                .body(BodyKind::Fixed)
                .visual(Visual::new(Shape::Box { half }, Color::hex("#39382e"))),
        )
    }
    pub fn sync_doors(&mut self, sim: &mut Sim) {
        for id in [self.gate_entity.take(), self.shortcut_entity.take(), self.fog_entity.take()].into_iter().flatten() {
            sim.despawn(id);
        }
        if !self.progress.gate {
            self.gate_entity = Some(Self::door(sim, "oath gate", self.rules.gate, Vec3::new(2.4, 2.5, 0.35)));
        }
        if !self.progress.shortcut {
            self.shortcut_entity = Some(Self::door(sim, "shortcut gate", Vec3::new(-12., 0., 3.), Vec3::new(2.4, 2., 0.35)));
        }
        if self.boss_active {
            self.fog_entity = Some(Self::door(sim, "sealed mist", self.rules.fog, Vec3::new(3.5, 2., 0.2)));
        }
        self.nav = None;
    }
    pub fn reset_encounters(&mut self, sim: &mut Sim) {
        for e in &self.enemies {
            sim.despawn(e.id);
        }
        self.enemies.clear();
        self.bolts.clear();
        self.boss_active = false;
        self.locked = None;
        for spec in &self.rules.encounters {
            if spec.kind == Kind::Guardian && self.progress.guardian {
                continue;
            }
            let id = sim.spawn_npc("abbey foe", spec.at, 0., PuppetDef::preset(BodyPlan::Biped), None, None);
            self.enemies.push(Enemy {
                id,
                kind: spec.kind,
                home: spec.at,
                pos: spec.at,
                hp: spec.hp,
                max_hp: spec.hp,
                reward: spec.reward,
                sight: spec.sight,
                facing: 0.,
                intent: Intent::Idle,
                timer: sim.state.rng.range(0.1, 0.8),
                duration: 0.,
                attack: 0,
                cycle: 0,
                phase: 1,
                poise: 0.,
                flash: 0.,
            });
        }
    }
    pub fn begin(&mut self, sim: &mut Sim, new: bool) {
        if new {
            self.progress = Progress::default();
            self.blood = None;
            self.time = 0.;
            self.metrics = Metrics::default();
        }
        self.progress.started = true;
        self.blood = self.progress.blood;
        if self.lab {
            self.progress.chime = true;
            self.progress.gate = true;
            self.progress.shortcut = true;
            self.progress.relic = true;
            self.progress.echoes = 600;
        }
        self.hp = self.progress.max_hp();
        self.stamina = self.progress.max_stamina();
        self.flasks = 3;
        self.action = Action::Idle;
        self.action_time = 0.;
        self.invuln = 1.;
        self.guard = false;
        self.reset_encounters(sim);
        self.sync_doors(sim);
        self.progress.checkpoint = self.progress.checkpoint.min(self.rules.shrines.len() - 1);
        self.player_pos = self.rules.shrines[self.progress.checkpoint];
        if let Some(id) = sim.state.player {
            sim.set_position(id, self.player_pos);
        }
        self.set_mode(Mode::Playing);
        self.area_time = 5.;
        self.save();
    }
    pub fn restore_progress(&mut self, p: Progress, sim: &mut Sim) {
        self.progress = p;
        self.begin(sim, false);
        self.set_mode(Mode::Title);
    }
    fn start_demo(&mut self, sim: &mut Sim) {
        self.demo_backup = Some(DemoBackup { progress: self.progress.clone(), time: self.time, metrics: self.metrics.clone() });
        self.demo = true;
        let cycles = self.metrics.cycles;
        self.begin(sim, true);
        self.metrics.cycles = cycles;
        self.announce("A pilgrim walks where bells no longer ring.");
    }
    fn end_demo(&mut self, sim: &mut Sim) {
        let backup = self.demo_backup.take().unwrap_or(DemoBackup {
            progress: Progress::default(),
            time: 0.0,
            metrics: Metrics::default(),
        });
        let started = backup.progress.started;
        self.progress = backup.progress;
        self.time = backup.time;
        self.metrics = backup.metrics;
        self.begin(sim, false);
        self.progress.started = started;
        self.demo = false;
        self.set_mode(Mode::Title);
        self.notice_time = 0.;
    }
    pub fn options_list(&self) -> Vec<String> {
        vec![
            format!("Pilgrim aid                 {}", if self.options.assist { "On" } else { "Off" }),
            format!("Camera motion           {}", if self.options.shake { "On" } else { "Off" }),
            format!("Sound                        {}", if self.options.sound { "On" } else { "Off" }),
            format!("High contrast             {}", if self.options.contrast { "On" } else { "Off" }),
            format!("Camera                        {}", if self.options.camera == 0 { "Adventure" } else { "Tactical" }),
            "Return".into(),
        ]
    }
    pub fn menu_items(&self) -> Vec<String> {
        match self.mode {
            Mode::Title => vec![
                if self.progress.started { "Continue pilgrimage" } else { "Begin pilgrimage" }.into(),
                "A new oath".into(),
                "Settings".into(),
                "How to play".into(),
            ],
            Mode::Pause => vec![
                "Continue".into(),
                "Equipment & memories".into(),
                "Settings".into(),
                "How to play".into(),
                "Return to title".into(),
            ],
            Mode::Rest => vec![
                "Leave the sanctuary".into(),
                format!("Temper vitality       {} echoes", self.upgrade_cost(false)),
                format!("Hone the edge         {} echoes", self.upgrade_cost(true)),
                format!("Weapon: {}", self.progress.weapon_name()),
                "Memories of the abbey".into(),
            ],
            Mode::Settings => self.options_list(),
            Mode::Guide | Mode::Journal => vec!["Return".into()],
            Mode::Dead => vec!["Rise at the sanctuary".into(), "Return to title".into()],
            Mode::Victory => vec!["Walk the quiet abbey".into(), "Begin another pilgrimage".into(), "Return to title".into()],
            _ => Vec::new(),
        }
    }
    pub fn upgrade_cost(&self, edge: bool) -> u32 {
        80 + if edge { self.progress.edge } else { self.progress.vigor } * 60
    }
    fn menu_input(&mut self, sim: &mut Sim, input: &InputFrame, events: &mut Vec<SimEvent>) {
        let mut input = *input;
        input.pressed |= self.buffer & b::ACCEPT;
        self.buffer &= !b::ACCEPT;
        let n = self.menu_items().len();
        if n == 0 {
            return;
        }
        if input.just(b::DOWN | b::NEXT) {
            self.menu = (self.menu + 1) % n;
            events.push(SimEvent::Coin { pos: self.player_pos });
        }
        if input.just(b::UP | b::PREV) {
            self.menu = (self.menu + n - 1) % n;
            events.push(SimEvent::Coin { pos: self.player_pos });
        }
        if input.just(b::BACK | b::MENU) {
            match self.mode {
                Mode::Pause | Mode::Rest => self.set_mode(Mode::Playing),
                Mode::Settings | Mode::Guide | Mode::Journal => self.set_mode(self.return_mode),
                _ => {}
            }
            return;
        }
        if !input.just(b::ACCEPT | b::INTERACT) {
            return;
        }
        match self.mode {
            Mode::Title => match self.menu {
                0 => self.begin(sim, false),
                1 => self.begin(sim, true),
                2 => {
                    self.return_mode = Mode::Title;
                    self.set_mode(Mode::Settings)
                }
                _ => {
                    self.return_mode = Mode::Title;
                    self.set_mode(Mode::Guide)
                }
            },
            Mode::Pause => match self.menu {
                0 => self.set_mode(Mode::Playing),
                1 => {
                    self.return_mode = Mode::Pause;
                    self.set_mode(Mode::Journal)
                }
                2 => {
                    self.return_mode = Mode::Pause;
                    self.set_mode(Mode::Settings)
                }
                3 => {
                    self.return_mode = Mode::Pause;
                    self.set_mode(Mode::Guide)
                }
                _ => {
                    self.save();
                    self.set_mode(Mode::Title)
                }
            },
            Mode::Rest => match self.menu {
                0 => self.set_mode(Mode::Playing),
                1 | 2 => {
                    let edge = self.menu == 2;
                    let cost = self.upgrade_cost(edge);
                    let level = if edge { self.progress.edge } else { self.progress.vigor };
                    if level >= 5 {
                        self.announce("This blessing is complete.");
                    } else if self.progress.echoes >= cost {
                        self.progress.echoes -= cost;
                        if edge {
                            self.progress.edge += 1;
                        } else {
                            self.progress.vigor += 1;
                            self.hp = self.progress.max_hp();
                        }
                        self.save();
                        events.push(SimEvent::LevelUp { pos: self.player_pos });
                        self.announce("An oath made stronger.");
                    } else {
                        self.announce(format!("You need {cost} echoes. Fallen retainers carry them."));
                    }
                }
                3 => self.switch_weapon(),
                _ => {
                    self.return_mode = Mode::Rest;
                    self.set_mode(Mode::Journal)
                }
            },
            Mode::Settings => {
                match self.menu {
                    0 => self.options.assist = !self.options.assist,
                    1 => self.options.shake = !self.options.shake,
                    2 => self.options.sound = !self.options.sound,
                    3 => self.options.contrast = !self.options.contrast,
                    4 => self.options.camera = 1 - self.options.camera,
                    _ => self.set_mode(self.return_mode),
                }
                self.save();
            }
            Mode::Guide | Mode::Journal => self.set_mode(self.return_mode),
            Mode::Dead => {
                if self.menu == 0 {
                    self.begin(sim, false);
                } else {
                    self.set_mode(Mode::Title);
                }
            }
            Mode::Victory => match self.menu {
                0 => self.set_mode(Mode::Playing),
                1 => self.begin(sim, true),
                _ => self.set_mode(Mode::Title),
            },
            _ => {}
        }
    }
    fn switch_weapon(&mut self) {
        if self.progress.relic {
            self.progress.weapon = 1 - self.progress.weapon;
            self.save();
            self.announce(self.progress.weapon_name());
        } else {
            self.announce("A second blade sleeps in the western reliquary.");
        }
    }
    fn interact(&mut self, sim: &mut Sim, events: &mut Vec<SimEvent>) {
        let p = self.player_pos;
        for (i, s) in self.rules.shrines.iter().enumerate() {
            if flat(p - *s) < 2.3 && !self.boss_active {
                self.progress.checkpoint = i;
                self.hp = self.progress.max_hp();
                self.stamina = self.progress.max_stamina();
                self.flasks = 3;
                self.reset_encounters(sim);
                self.sync_doors(sim);
                self.metrics.rests += 1;
                self.save();
                events.push(SimEvent::Checkpoint { pos: p });
                self.set_mode(Mode::Rest);
                self.announce("The flame remembers you. The fallen rise again.");
                return;
            }
        }
        if flat(p - self.rules.reliquary) < 2.3 && !self.progress.relic {
            self.progress.relic = true;
            self.progress.lore |= 1;
            self.progress.echoes += 50;
            self.stamina = self.progress.max_stamina();
            self.save();
            events.push(SimEvent::Loot { pos: p, rarity: 3 });
            self.announce("VESPER GREATBLADE · Stamina +20. Triangle / Q to equip.");
            return;
        }
        if flat(p - self.rules.chime) < 2.3 && !self.progress.chime {
            if self.enemies.iter().any(|e| e.hp > 0. && flat(e.pos - self.rules.chime) < 6.) {
                self.announce("Silence the cantor before claiming the chime.");
                return;
            }
            self.progress.chime = true;
            self.progress.lore |= 2;
            self.save();
            events.push(SimEvent::Pickup { pos: p, rarity: 3 });
            self.announce("THE UNBROKEN CHIME · The central oath gate will answer.");
            return;
        }
        if flat(p - self.rules.gate) < 3. && !self.progress.gate {
            if self.progress.chime {
                self.progress.gate = true;
                self.sync_doors(sim);
                self.save();
                events.push(SimEvent::Gate { pos: p, ok: true });
                self.announce("The oath gate opens.");
            } else {
                self.announce("A chime is missing. Search the eastern chapel.");
            }
            return;
        }
        if flat(p - self.rules.lever) < 2.5 && !self.progress.shortcut {
            self.progress.shortcut = true;
            self.sync_doors(sim);
            self.save();
            events.push(SimEvent::Gate { pos: p, ok: true });
            self.announce("SHORTCUT OPENED · The west passage returns to the courtyard.");
            return;
        }
        if flat(p - self.rules.fog) < 3. && !self.progress.guardian && !self.boss_active && self.progress.gate {
            self.boss_active = true;
            self.player_pos = Vec3::new(0., 0., -15.);
            if let Some(id) = sim.state.player {
                sim.set_position(id, self.player_pos);
            }
            self.sync_doors(sim);
            self.announce("VESPER, THE UNRUNG");
            events.push(SimEvent::Slam { pos: self.rules.fog, radius: 4. });
            return;
        }
        if flat(p - self.rules.bell) < 3. && self.progress.guardian {
            self.progress.lore |= 4;
            self.notice_time = 0.0;
            self.set_mode(Mode::Victory);
            self.save();
            events.push(SimEvent::LevelUp { pos: self.rules.bell });
        }
    }
    fn set_action(&mut self, a: Action, duration: f32, cost: f32) {
        self.action = a;
        self.action_time = duration;
        self.action_total = duration;
        self.hit_done = false;
        self.stamina = (self.stamina - cost).max(0.);
        self.regen_delay = 0.55;
    }
    fn damage_player(&mut self, damage: f32, from: Vec3, events: &mut Vec<SimEvent>) {
        if self.invuln > 0. || self.mode != Mode::Playing {
            return;
        }
        let facing = forward(self.facing).dot((from - self.player_pos).normalize_or_zero()) > 0.1;
        if self.action == Action::Parry && self.action_total - self.action_time < 0.26 && facing {
            return;
        }
        let mut damage = damage;
        if self.options.assist {
            damage *= 0.55;
        }
        if self.guard && facing && self.stamina > 0. {
            self.metrics.blocks += 1;
            self.stamina = (self.stamina - damage * 1.5).max(0.);
            self.regen_delay = 1.;
            events.push(SimEvent::Block { pos: self.player_pos });
            damage *= 0.12;
            if self.stamina == 0. {
                self.set_action(Action::Stagger, 1., 0.);
                self.announce("GUARD BROKEN");
            }
        }
        self.hp = (self.hp - damage).max(0.);
        self.invuln = 0.55;
        self.flash = 0.35;
        event_hit(events, self.player_pos, damage);
        if self.hp <= 0. {
            self.progress.deaths += 1;
            self.blood = Some((self.player_pos, self.progress.echoes));
            self.progress.blood = self.blood;
            self.progress.echoes = 0;
            self.set_mode(Mode::Dead);
            self.save();
        }
    }
    fn kill(&mut self, i: usize, sim: &mut Sim, events: &mut Vec<SimEvent>) {
        let e = &mut self.enemies[i];
        e.hp = 0.;
        e.intent = Intent::Dead;
        sim.set_position(e.id, e.home - Vec3::Y * 40.);
        self.progress.echoes += e.reward;
        self.progress.kills += 1;
        events.push(SimEvent::Slain { pos: e.pos, size: if e.boss() { 3. } else { 1. } });
        if e.boss() {
            self.progress.guardian = true;
            self.boss_active = false;
            self.bolts.clear();
            self.sync_doors(sim);
            self.save();
            self.announce("OATH FULFILLED · Ring the bell beyond the throne.");
        }
        if self.locked == Some(i) {
            self.locked = None;
        }
    }
    fn strike(&mut self, sim: &mut Sim, events: &mut Vec<SimEvent>) {
        let heavy = self.action == Action::Heavy;
        let great = self.progress.weapon == 1;
        let reach = if great { 3.5 } else { 2.9 };
        let base = if heavy { self.rules.heavy_damage } else { self.rules.light_damage };
        let damage = base * (1. + self.progress.edge as f32 * 0.15) * if great { 1.3 } else { 1. };
        let mut hit = false;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if e.hp <= 0. {
                continue;
            }
            let v = e.pos - self.player_pos;
            if flat(v) > reach + if e.boss() { 0.6 } else { 0. }
                || forward(self.facing).dot(v.normalize_or_zero()) < if heavy { -0.15 } else { 0.0 }
            {
                continue;
            }
            if !self.line_clear(self.player_pos, e.pos) {
                continue;
            }
            let critical = e.intent == Intent::Stagger;
            let e = &mut self.enemies[i];
            let dealt = damage * if critical { 1.7 } else { 1. };
            e.hp -= dealt;
            e.flash = 0.22;
            e.poise += if heavy { 55. } else { 20. };
            hit = true;
            event_hit(events, e.pos, dealt);
            if e.poise >= if e.boss() { 110. } else { 60. } {
                e.intent = Intent::Stagger;
                e.timer = 1.5;
                e.duration = 1.5;
                e.poise = 0.;
            }
            if e.hp <= 0. {
                self.kill(i, sim, events);
            }
        }
        if hit {
            self.flash = self.flash.max(0.06);
        }
    }
    fn line_clear(&self, a: Vec3, z: Vec3) -> bool {
        let Some(nav) = &self.nav else {
            return true;
        };
        let steps = (flat(z - a) * 3.).ceil() as usize;
        (1..steps).all(|i| nav.walkable(a.lerp(z, i as f32 / steps as f32)))
    }
    fn steer(&self, p: Vec3, to: Vec3) -> Vec2 {
        if let Some(nav) = &self.nav {
            if let Some(path) = nav.path(p, to) {
                if let Some(q) = path.iter().find(|q| flat(**q - p) > 0.18) {
                    return dir(*q - p);
                }
            }
        }
        dir(to - p)
    }
    fn update_prompt(&mut self) {
        let p = self.player_pos;
        self.prompt = if self.rules.shrines.iter().any(|s| flat(p - *s) < 2.3) && !self.boss_active {
            "Rest at the sanctuary"
        } else if !self.progress.relic && flat(p - self.rules.reliquary) < 2.3 {
            "Open the reliquary"
        } else if !self.progress.chime && flat(p - self.rules.chime) < 2.3 {
            "Claim the unbroken chime"
        } else if !self.progress.gate && flat(p - self.rules.gate) < 3. {
            "Open the oath gate"
        } else if !self.progress.shortcut && flat(p - self.rules.lever) < 2.5 {
            "Release the west passage"
        } else if !self.progress.guardian && !self.boss_active && flat(p - self.rules.fog) < 3. {
            "Enter the hush"
        } else if self.progress.guardian && flat(p - self.rules.bell) < 3. {
            "Ring the last bell"
        } else {
            ""
        }
        .into();
        let area = if p.z < -12. {
            "THE SILENT BELLFRY"
        } else if p.z < 3. {
            "THE OATH CLOISTER"
        } else if p.x < -8. {
            "WESTERN RELIQUARY"
        } else if p.x > 8. {
            "CHAPEL OF ASH"
        } else if p.z < 19. {
            "THE FORSAKEN COURT"
        } else {
            "PILGRIM'S APPROACH"
        };
        if area != self.area {
            self.area = area.into();
            self.area_time = 4.;
        }
    }
    pub fn objective(&self) -> &'static str {
        if self.progress.lore & 4 != 0 {
            "The oath is fulfilled"
        } else if self.progress.guardian {
            "Ring the last bell"
        } else if self.boss_active {
            "Silence Vesper, the Unrung"
        } else if !self.progress.chime {
            "Find the chime in the eastern chapel"
        } else if !self.progress.gate {
            "Return to the central oath gate"
        } else {
            "Cross the cloister. Enter the hush."
        }
    }
}

impl Game for Dented {
    const KEY: &'static str = "dented";
    fn pre(&mut self, sim: &mut Sim, raw: &InputFrame, dt: f32, events: &mut Vec<SimEvent>) -> BTreeMap<EntityId, InputFrame> {
        let mut out = BTreeMap::new();
        let Some(pid) = sim.state.player else {
            return out;
        };
        self.player_pos = sim.feet(pid).unwrap_or(self.player_pos);
        self.mode_time += dt;
        self.notice_time = (self.notice_time - dt).max(0.);
        self.area_time = (self.area_time - dt).max(0.);
        let activity = raw.held != 0 || raw.pressed != 0 || raw.move_dir.length_squared() > 0.03;
        if self.demo && activity {
            self.end_demo(sim);
            out.insert(pid, InputFrame::default());
            return out;
        }
        if !self.demo
            && (self.mode == Mode::Title
                || (matches!(self.mode, Mode::Settings | Mode::Guide | Mode::Journal) && self.return_mode == Mode::Title))
        {
            self.idle = if activity { 0. } else { self.idle + dt };
            if self.idle + 0.0001 >= 10. {
                self.start_demo(sim);
            }
        }
        if self.demo && matches!(self.mode, Mode::Dead | Mode::Victory) && self.mode_time > 4. {
            self.metrics.cycles += 1;
            let cycles = self.metrics.cycles;
            self.begin(sim, true);
            self.metrics.cycles = cycles;
        }
        if self.mode != Mode::Playing {
            let menu_frame = if self.demo && self.mode == Mode::Rest && self.mode_time < 2.5 {
                InputFrame::default()
            } else if self.demo {
                self.bot(sim)
            } else {
                *raw
            };
            self.menu_input(sim, &menu_frame, events);
            out.insert(pid, InputFrame::default());
            for e in &self.enemies {
                out.insert(e.id, InputFrame::default());
            }
            if let Some(ch) = sim.state.entities.get_mut(pid).and_then(|e| e.character.as_mut()) {
                ch.dash_time = 0.;
                ch.slow = 1.;
            }
            return out;
        }
        if self.nav.is_none() {
            self.nav = NavGrid::for_room(sim).map(Arc::new);
        }
        let input = if self.demo { self.bot(sim) } else { *raw };
        if input.just(b::MENU) {
            self.set_mode(Mode::Pause);
            out.insert(pid, InputFrame::default());
            return out;
        }
        if input.just(b::JOURNAL) {
            self.return_mode = Mode::Pause;
            self.set_mode(Mode::Journal);
            out.insert(pid, InputFrame::default());
            return out;
        }
        self.time += dt;
        self.regen_delay = (self.regen_delay - dt).max(0.);
        self.invuln = (self.invuln - dt).max(0.);
        self.flash = (self.flash - dt).max(0.);
        self.action_time = (self.action_time - dt).max(0.);
        if self.action_time <= 0. {
            self.action = Action::Idle;
        }
        self.guard = input.down(b::FOCUS) && self.action == Action::Idle;
        if self.regen_delay <= 0. && self.action == Action::Idle {
            self.stamina =
                (self.stamina + self.rules.regen * dt * if self.guard { 0.35 } else { 1. }).min(self.progress.max_stamina());
        }
        if input.just(b::SKILL3) {
            self.locked = if self.locked.is_some() {
                None
            } else {
                self.enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.hp > 0. && flat(e.pos - self.player_pos) < 16.)
                    .min_by(|(_, a), (_, z)| flat(a.pos - self.player_pos).total_cmp(&flat(z.pos - self.player_pos)))
                    .map(|(i, _)| i)
            };
        }
        if let Some(i) = self.locked {
            if self.enemies.get(i).is_none_or(|e| e.hp <= 0. || flat(e.pos - self.player_pos) > 22.) {
                self.locked = None;
            }
        }
        let aim = self.locked.map(|i| self.enemies[i].pos).or(input.aim);
        if self.action == Action::Idle || self.guard {
            if let Some(at) = aim {
                if flat(at - self.player_pos) > 0.3 {
                    self.facing = angle(at - self.player_pos);
                }
            } else if input.move_dir.length() > 0.1 {
                self.facing = input.move_dir.x.atan2(input.move_dir.y);
            }
        }
        self.buffer_time = (self.buffer_time - dt).max(0.);
        if self.buffer_time <= 0. {
            self.buffer = 0;
        }
        if input.pressed & (b::PRIMARY | b::SECONDARY) != 0 {
            self.buffer = input.pressed & (b::PRIMARY | b::SECONDARY);
            self.buffer_time = 0.2;
        }
        if input.just(b::DODGE) && self.stamina >= 24. && self.action != Action::Stagger {
            self.set_action(Action::Dodge, 0.55, 24.);
            self.invuln = 0.34;
            self.metrics.dodges += 1;
            self.guard = false;
            let d = if input.move_dir.length() > 0.1 {
                Vec3::new(input.move_dir.x, 0., input.move_dir.y).normalize()
            } else {
                -forward(self.facing)
            };
            if let Some(ch) = sim.state.entities.get_mut(pid).and_then(|e| e.character.as_mut()) {
                ch.dash_vel = d * 9.;
                ch.dash_time = 0.38;
                ch.dash_roll = true;
            }
            events.push(SimEvent::Roll { pos: self.player_pos });
        } else if self.action == Action::Idle {
            if input.just(b::POTION) && self.flasks > 0 && self.hp < self.progress.max_hp() {
                self.flasks -= 1;
                self.metrics.heals += 1;
                self.set_action(Action::Heal, 0.95, 0.);
                events.push(SimEvent::Potion { pos: self.player_pos });
            } else if input.just(b::SKILL4) && self.stamina >= 15. {
                self.set_action(Action::Parry, 0.6, 15.);
                events.push(SimEvent::Swing { pos: self.player_pos, heavy: false });
            } else if self.buffer & b::SECONDARY != 0 && self.stamina >= 34. {
                self.set_action(Action::Heavy, if self.progress.weapon == 1 { 1.15 } else { 0.95 }, 34.);
                self.metrics.heavies += 1;
                self.buffer = 0;
                events.push(SimEvent::Swing { pos: self.player_pos, heavy: true });
            } else if self.buffer & b::PRIMARY != 0 && self.stamina >= 18. {
                self.set_action(Action::Light, if self.progress.weapon == 1 { 0.75 } else { 0.52 }, 18.);
                self.combo = (self.combo + 1) % 3;
                self.metrics.lights += 1;
                self.buffer = 0;
                events.push(SimEvent::Swing { pos: self.player_pos, heavy: false });
            } else if input.just(b::INTERACT | b::ACCEPT) {
                self.interact(sim, events);
            }
            if input.just(b::KIT) {
                self.switch_weapon();
            }
        }
        let elapsed = self.action_total - self.action_time;
        if !self.hit_done {
            if matches!(self.action, Action::Light | Action::Heavy) && elapsed >= self.action_total * 0.46 {
                self.strike(sim, events);
                self.hit_done = true;
            }
            if self.action == Action::Heal && elapsed >= 0.7 {
                self.hp = (self.hp + 70.).min(self.progress.max_hp());
                self.hit_done = true;
                events.push(SimEvent::Potion { pos: self.player_pos });
            }
        }
        if let Some(ch) = sim.state.entities.get_mut(pid).and_then(|e| e.character.as_mut()) {
            ch.slow = if self.action == Action::Idle { if self.guard { 0.6 } else { 0. } } else { 0.83 };
            ch.face = Some(self.facing);
            ch.invuln = self.invuln;
        }
        let mut player = InputFrame { move_dir: input.move_dir, ..Default::default() };
        if self.action == Action::Idle {
            player.held = input.held & b::JUMP;
            player.pressed = input.pressed & b::JUMP;
        }
        out.insert(pid, player);
        // Enemies share the same telegraph -> active -> recovery state machine.
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if e.hp <= 0. {
                out.insert(e.id, InputFrame::default());
                continue;
            }
            let pos = sim.feet(e.id).unwrap_or(e.pos);
            let delta = self.player_pos - pos;
            let distance = flat(delta);
            let can_see = self.line_clear(pos, self.player_pos);
            let mut movement = Vec2::ZERO;
            let steer = self.steer(pos, self.player_pos);
            let e = &mut self.enemies[i];
            e.pos = pos;
            e.flash = (e.flash - dt).max(0.);
            e.timer = (e.timer - dt).max(0.);
            e.poise = (e.poise - dt * 4.).max(0.);
            if e.boss() && e.hp < e.max_hp * 0.5 && e.phase == 1 {
                e.phase = 2;
                e.intent = Intent::Recover;
                e.timer = 1.8;
                self.metrics.phase_two = true;
                self.notice = "THE SECOND TOLL · His silence becomes a storm.".into();
                self.notice_time = 4.;
                events.push(SimEvent::Slam { pos: e.pos, radius: 5. });
            }
            let engaged = (!e.boss() && distance < e.sight) || (e.boss() && self.boss_active);
            if !engaged && matches!(e.intent, Intent::Idle | Intent::Chase) {
                e.intent = Intent::Idle;
            } else if self.mode == Mode::Playing {
                match e.intent {
                    Intent::Idle | Intent::Chase => {
                        e.intent = Intent::Chase;
                        e.facing = angle(delta);
                        let reach = if e.kind == Kind::Cantor {
                            10.
                        } else if e.boss() {
                            3.6
                        } else if e.kind == Kind::Lancer {
                            2.8
                        } else {
                            1.8
                        };
                        if distance < reach && e.timer <= 0. && can_see {
                            e.intent = Intent::Windup;
                            e.attack = if e.boss() { (e.cycle % if e.phase == 2 { 3 } else { 2 }) as u8 } else { 0 };
                            e.cycle += 1;
                            e.duration = if e.boss() {
                                if e.attack == 1 { 1.25 } else { 0.95 }
                            } else if e.kind == Kind::Lancer {
                                0.95
                            } else {
                                0.8
                            };
                            e.timer = e.duration;
                        } else {
                            movement = steer
                                * if e.boss() {
                                    0.58
                                } else if e.kind == Kind::Lancer {
                                    0.48
                                } else {
                                    0.42
                                };
                        }
                    }
                    Intent::Windup => {
                        if e.timer > e.duration * 0.45 {
                            e.facing = angle(delta);
                        }
                        if e.timer <= 0. {
                            let ep = e.pos;
                            let yaw = e.facing;
                            let ranged = e.kind == Kind::Cantor || e.boss() && e.attack == 2;
                            let attack = e.attack;
                            let reach = e.reach();
                            let boss = e.boss();
                            e.intent = Intent::Recover;
                            e.timer = if boss { if e.phase == 2 { 0.95 } else { 1.25 } } else { 1.05 };
                            e.duration = e.timer;
                            if ranged {
                                let n = if boss { 3 } else { 1 };
                                for k in 0..n {
                                    let a = yaw + (k as f32 - (n - 1) as f32 * 0.5) * 0.2;
                                    self.bolts.push(Bolt { pos: ep + Vec3::Y * 0.8, vel: forward(a) * 7.5, life: 3. });
                                }
                                events.push(SimEvent::Spell { pos: ep, element: 1 });
                            } else {
                                events.push(if attack == 1 && boss {
                                    SimEvent::Slam { pos: ep, radius: reach }
                                } else {
                                    SimEvent::Swing { pos: ep, heavy: boss }
                                });
                                let in_arc = attack == 1 && boss || forward(yaw).dot(delta.normalize_or_zero()) > 0.05;
                                if distance < reach && in_arc && can_see {
                                    let parry = self.action == Action::Parry
                                        && self.action_total - self.action_time < 0.26
                                        && attack != 1
                                        && forward(self.facing).dot((-delta).normalize_or_zero()) > 0.;
                                    if parry {
                                        self.metrics.parries += 1;
                                        let e = &mut self.enemies[i];
                                        e.intent = Intent::Stagger;
                                        e.timer = 2.4;
                                        e.duration = 2.4;
                                        self.stamina = (self.stamina + 25.).min(self.progress.max_stamina());
                                        events.push(SimEvent::Block { pos: self.player_pos });
                                        self.announce("OATHBREAK · Strike now");
                                    } else {
                                        self.damage_player(
                                            if boss {
                                                34.
                                            } else if self.enemies[i].kind == Kind::Lancer {
                                                28.
                                            } else {
                                                22.
                                            },
                                            ep,
                                            events,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Intent::Recover | Intent::Stagger => {
                        if e.timer <= 0. {
                            e.intent = Intent::Chase;
                        }
                    }
                    Intent::Dead => {}
                }
            }
            let e = &self.enemies[i];
            let movement = if self.mode == Mode::Playing { movement } else { Vec2::ZERO };
            if let Some(ch) = sim.state.entities.get_mut(e.id).and_then(|e| e.character.as_mut()) {
                ch.face = Some(e.facing);
                ch.slow = 0.;
            }
            out.insert(e.id, InputFrame { move_dir: movement, ..Default::default() });
        }
        self.update_prompt();
        out
    }
    fn post(&mut self, sim: &mut Sim, dt: f32, events: &mut Vec<SimEvent>) {
        if self.mode != Mode::Playing {
            return;
        }
        self.player_pos = sim.state.player.and_then(|id| sim.feet(id)).unwrap_or(self.player_pos);
        // Mist is crossed only through the interaction, so it cannot be bypassed on foot.
        if !self.boss_active && !self.progress.guardian && self.player_pos.z < -12. {
            self.player_pos.z = -11.7;
            if let Some(id) = sim.state.player {
                sim.set_position(id, self.player_pos);
            }
        }
        if self.player_pos.y < -3. {
            self.damage_player(999., self.player_pos, events);
        }
        for e in &mut self.enemies {
            if e.hp > 0. {
                e.pos = sim.feet(e.id).unwrap_or(e.pos);
            }
        }
        let mut hits = Vec::new();
        for bolt in &mut self.bolts {
            bolt.life -= dt;
            bolt.pos += bolt.vel * dt;
            if flat(bolt.pos - self.player_pos) < 0.65 {
                hits.push(bolt.pos);
                bolt.life = 0.;
            }
            if self.nav.as_ref().is_some_and(|n| !n.walkable(bolt.pos)) {
                bolt.life = 0.;
            }
        }
        self.bolts.retain(|x| x.life > 0.);
        for p in hits {
            self.damage_player(24., p, events);
        }
        if let Some((at, n)) = self.blood {
            if flat(at - self.player_pos) < 1.3 {
                self.blood = None;
                self.progress.blood = None;
                self.progress.echoes += n;
                self.metrics.recovered += 1;
                self.announce("ECHOES RECLAIMED");
                events.push(SimEvent::Pickup { pos: at, rarity: 2 });
                self.save();
            }
        }
        if self.progress.guardian && self.progress.best_seconds == 0. {
            self.progress.best_seconds = self.time;
            self.save();
        }
    }
    fn hud(&self, _: &Sim) -> Vec<String> {
        vec![
            format!("{} — {:?}", self.area, self.mode),
            format!(
                "HP {:.0}/{:.0} · stamina {:.0} · flasks {} · echoes {}",
                self.hp,
                self.progress.max_hp(),
                self.stamina,
                self.flasks,
                self.progress.echoes
            ),
            self.objective().into(),
        ]
    }
    fn status(&self, _: &Sim) -> serde_json::Value {
        json!({"mode":self.mode,"demo":self.demo,"time":self.time,"player":self.player_pos,"hp":self.hp,"stamina":self.stamina,"action":self.action,"progress":self.progress,"metrics":self.metrics,"boss_active":self.boss_active,"enemies":self.enemies.iter().map(|e|json!({"kind":e.kind,"hp":e.hp,"pos":e.pos,"intent":e.intent,"timer":e.timer,"attack":e.attack})).collect::<Vec<_>>()})
    }
    fn outcome(&self) -> Outcome {
        if self.mode == Mode::Victory {
            Outcome::Won
        } else if self.mode == Mode::Dead {
            Outcome::Lost
        } else {
            Outcome::Playing
        }
    }
    fn bot(&self, _: &Sim) -> InputFrame {
        let mut f = InputFrame::default();
        if self.mode == Mode::Title {
            f.pressed = b::ACCEPT;
            return f;
        }
        if self.mode == Mode::Rest {
            let target = if self.progress.vigor == 0 && self.progress.echoes >= self.upgrade_cost(false) {
                1
            } else if self.progress.edge == 0 && self.progress.echoes >= self.upgrade_cost(true) {
                2
            } else if self.progress.relic && self.progress.weapon == 0 {
                3
            } else {
                0
            };
            f.pressed = if self.menu == target { b::ACCEPT } else { b::DOWN };
            return f;
        }
        if self.mode != Mode::Playing {
            return f;
        }
        let p = self.player_pos;
        let nearest = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0. && (!e.boss() || self.boss_active) && flat(e.pos - p) < 10.)
            .min_by(|a, z| flat(a.pos - p).total_cmp(&flat(z.pos - p)));
        if let Some(e) = nearest {
            let v = e.pos - p;
            let d = flat(v);
            f.aim = Some(e.pos);
            if self.hp < self.progress.max_hp() * 0.55 && self.flasks > 0 && e.intent != Intent::Windup {
                f.pressed = b::POTION;
                f.move_dir = -dir(v);
                return f;
            }
            if e.intent == Intent::Windup && d < e.reach() + 0.4 {
                if e.timer < 0.22 && self.action == Action::Idle && self.stamina >= 24. {
                    f.pressed = b::DODGE;
                    let toward = dir(v);
                    f.move_dir = (toward * 0.35 + Vec2::new(toward.y, -toward.x) * 0.94).normalize_or_zero();
                } else if e.timer > 0.32 && d > 2.0 {
                    f.move_dir = self.steer(p, e.pos);
                }
                return f;
            }
            if d > 2.3 {
                f.move_dir = self.steer(p, e.pos);
            } else if self.action == Action::Idle {
                if self.stamina > 42. {
                    f.pressed = if e.intent == Intent::Stagger || (self.metrics.lights + self.metrics.heavies) % 4 == 3 {
                        b::SECONDARY
                    } else {
                        b::PRIMARY
                    };
                } else {
                    f.move_dir = -dir(v) * 0.55;
                }
            }
            return f;
        }
        let target = if !self.progress.relic {
            self.rules.reliquary
        } else if !self.progress.chime {
            self.rules.chime
        } else if !self.progress.gate {
            self.rules.gate
        } else if !self.progress.shortcut {
            self.rules.lever
        } else if self.progress.checkpoint == 0 && self.rules.shrines.len() > 1 {
            self.rules.shrines[1]
        } else if !self.progress.guardian && !self.boss_active {
            self.rules.fog
        } else {
            self.rules.bell
        };
        if flat(target - p) < 1.8 {
            f.pressed = b::INTERACT;
        } else {
            f.move_dir = self.steer(p, target);
        }
        f
    }
}
