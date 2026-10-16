//! Ten-foot presentation. All actions also pass through the simulation's focus menu.
use crate::input::Device;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Stroke, Vec2};
use pav_core::games::dented::{Dented, Intent, Kind, Mode};
const GOLD: Color32 = Color32::from_rgb(205, 181, 125);
const IVORY: Color32 = Color32::from_rgb(233, 232, 213);
const MUTED: Color32 = Color32::from_rgb(153, 170, 158);

pub fn install_fonts(ctx: &egui::Context) {
    let mut f = egui::FontDefinitions::default();
    f.font_data
        .insert("abbey".into(), egui::FontData::from_static(include_bytes!("../../../assets/fonts/AbbeySerif.ttf")).into());
    f.families.insert(FontFamily::Name("abbey".into()), vec!["abbey".into()]);
    ctx.set_fonts(f);
}
struct Ink {
    p: egui::Painter,
    origin: Pos2,
    k: f32,
}
impl Ink {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.origin + Vec2::new(x, y) * self.k
    }
    fn text(&self, x: f32, y: f32, s: &str, size: f32, c: Color32, serif: bool) {
        self.p.text(
            self.at(x, y),
            Align2::LEFT_TOP,
            s,
            FontId::new(size * self.k, if serif { FontFamily::Name("abbey".into()) } else { FontFamily::Proportional }),
            c,
        );
    }
    fn centered(&self, x: f32, y: f32, s: &str, size: f32, c: Color32, serif: bool) {
        self.p.text(
            self.at(x, y),
            Align2::CENTER_TOP,
            s,
            FontId::new(size * self.k, if serif { FontFamily::Name("abbey".into()) } else { FontFamily::Proportional }),
            c,
        );
    }
    fn line(&self, x: f32, y: f32, w: f32, c: Color32) {
        self.p.line_segment([self.at(x, y), self.at(x + w, y)], Stroke::new(self.k, c));
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color32) {
        self.p.rect_filled(Rect::from_min_size(self.at(x, y), Vec2::new(w, h) * self.k), 0., c);
    }
    fn bar(&self, x: f32, y: f32, w: f32, h: f32, v: f32, c: Color32) {
        self.rect(x - 2., y - 2., w + 4., h + 4., Color32::from_black_alpha(180));
        self.rect(x, y, w, h, Color32::from_rgb(47, 47, 41));
        self.rect(x, y, w * v.clamp(0., 1.), h, c);
        self.line(x, y, w, Color32::from_white_alpha(50));
    }
    fn diamond(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.add(egui::Shape::convex_polygon(
            vec![self.at(x, y - r), self.at(x + r, y), self.at(x, y + r), self.at(x - r, y)],
            c,
            Stroke::NONE,
        ));
    }
    fn emblem(&self, x: f32, y: f32, r: f32) {
        let p = self.at(x, y);
        for m in [1., 0.78] {
            self.p.circle_stroke(p, r * m * self.k, Stroke::new(1. * self.k, GOLD));
        }
        self.p.line_segment([self.at(x, y - r * 1.4), self.at(x, y + r * 1.4)], Stroke::new(self.k, GOLD));
        self.line(x - r * 0.5, y, r, GOLD);
        self.diamond(x, y, 7., GOLD);
    }
}
fn veil(p: &egui::Painter, r: Rect, left: u8, right: u8) {
    let mut m = egui::Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    for (pos, a) in [(r.left_top(), left), (r.right_top(), right), (r.right_bottom(), right), (r.left_bottom(), left)] {
        m.vertices.push(egui::epaint::Vertex { pos, uv, color: Color32::from_black_alpha(a) });
    }
    m.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    p.add(egui::Shape::mesh(m));
}

pub fn draw(ui: &mut egui::Ui, g: &Dented, fps: f32, ms: f32, device: Device, mobile: bool, vp: glam::Mat4) -> Option<usize> {
    let ctx = ui.ctx().clone();
    let screen = ctx.content_rect();
    let k = (screen.width() / 1600.).min(screen.height() / 900.);
    let origin = screen.center() - Vec2::new(800., 450.) * k;
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("dented")));
    let ink = Ink { p: p.clone(), origin, k };
    let pad = device == Device::Gamepad;
    let mut selected = None;
    if mobile {
        return None;
    }
    let project = |v: glam::Vec3| -> Option<Pos2> {
        let clip = vp * v.extend(1.);
        if clip.w <= 0. {
            return None;
        }
        let n = clip.truncate() / clip.w;
        Some(screen.min + Vec2::new((n.x + 1.) * 0.5 * screen.width(), (1. - n.y) * 0.5 * screen.height()))
    };
    // FPS remains visible in every desktop menu and play mode.
    let fps_color = if fps >= 25. || fps == 0. { MUTED } else { Color32::from_rgb(238, 152, 117) };
    ink.text(1410., 28., &format!("{fps:.0} FPS  /  {ms:.1} MS"), 17., fps_color, false);
    let gameplay = matches!(g.mode, Mode::Playing | Mode::Dead | Mode::Victory);
    if gameplay {
        veil(&p, Rect::from_min_max(screen.min, Pos2::new(screen.max.x, screen.min.y + 180. * k)), 100, 10);
        ink.emblem(69., 79., 25.);
        ink.bar(111., 53., 310., 14., g.hp / g.progress.max_hp(), Color32::from_rgb(158, 67, 59));
        ink.bar(111., 78., 270., 8., g.stamina / g.progress.max_stamina(), Color32::from_rgb(117, 151, 109));
        ink.text(111., 99., &format!("PILGRIM  /  VITALITY {}", g.progress.vigor + 1), 15., MUTED, false);
        ink.centered(800., 34., "W            N            E", 16., MUTED, false);
        ink.line(670., 62., 260., Color32::from_white_alpha(35));
        ink.diamond(800., 62., 4., GOLD);
        ink.text(1285., 67., &format!("✧  {:05} ECHOES", g.progress.echoes), 21., GOLD, false);
        if g.area_time > 0. {
            let alpha = (g.area_time.min(1.) * 255.) as u8;
            ink.centered(800., 164., &g.area, 30., IVORY.gamma_multiply(alpha as f32 / 255.), true);
            ink.line(650., 211., 300., GOLD.gamma_multiply(alpha as f32 / 255. * 0.5));
        }
        if g.notice_time > 0. {
            ink.rect(340., 248., 920., 44., Color32::from_black_alpha(125));
            ink.centered(800., 258., &g.notice, 21., IVORY, false);
        }
        ink.rect(35., 746., 345., 105., Color32::from_black_alpha(125));
        ink.emblem(76., 787., 20.);
        ink.text(116., 764., &format!("{}  ·  {}", if pad { "□" } else { "X" }, "AMBER FLASK"), 21., GOLD, false);
        ink.text(116., 794., &format!("{} remaining", g.flasks), 20., IVORY, false);
        ink.text(49., 861., g.progress.weapon_name(), 19., IVORY, true);
        ink.text(
            1168.,
            837.,
            if pad {
                "R1  Strike     R2  Heavy\n○  Evade     L1  Guard"
            } else {
                "J / LMB  Strike     K / RMB  Heavy\nSpace  Evade     Shift  Guard"
            },
            19.,
            IVORY,
            false,
        );
        if !g.prompt.is_empty() && g.mode == Mode::Playing {
            ink.rect(520., 723., 560., 52., Color32::from_black_alpha(165));
            ink.centered(800., 736., &format!("{}   {}", if pad { "×" } else { "E" }, g.prompt), 23., IVORY, false);
        }
        let boss = g.enemies.iter().find(|e| e.kind == Kind::Guardian && e.hp > 0. && g.boss_active);
        if let Some(e) = boss {
            ink.text(390., 799., e.name(), 23., IVORY, true);
            ink.text(1175., 802., if e.phase == 1 { "I" } else { "II" }, 21., GOLD, true);
            ink.bar(390., 837., 820., 12., e.hp / e.max_hp, Color32::from_rgb(151, 72, 60));
            if e.intent == Intent::Windup && e.attack == 1 {
                ink.centered(800., 687., "BELLWAVE  ·  Evade the toll", 22., GOLD, false);
            }
        } else {
            ink.centered(800., 835., g.objective(), 21., MUTED, false);
        }
        for (i, e) in g.enemies.iter().enumerate() {
            if e.hp <= 0. || e.boss() {
                continue;
            }
            if e.hp < e.max_hp || g.locked == Some(i) {
                if let Some(at) = project(e.pos + glam::Vec3::Y * 2.35) {
                    let r = Rect::from_center_size(at, Vec2::new(70., 5.) * k);
                    p.rect_filled(r, 0., Color32::from_black_alpha(180));
                    p.rect_filled(
                        Rect::from_min_size(r.min, Vec2::new(r.width() * e.hp / e.max_hp, r.height())),
                        0.,
                        Color32::from_rgb(173, 92, 72),
                    );
                }
            }
        }
        if let Some(e) = g.locked.and_then(|i| g.enemies.get(i)) {
            if let Some(at) = project(e.pos + glam::Vec3::Y * 1.4) {
                p.circle_stroke(at, 8. * k, Stroke::new(1.5 * k, IVORY));
            }
        }
        if g.flash > 0.15 {
            let c = Color32::from_rgba_unmultiplied(130, 38, 28, (g.flash * 180.) as u8);
            p.rect_stroke(screen.shrink(3.), 0., Stroke::new(10. * k, c), egui::StrokeKind::Inside);
        }
    }
    if g.demo {
        ink.rect(510., 93., 580., 38., Color32::from_black_alpha(150));
        ink.centered(800., 101., "THE ABBEY REMEMBERS   /   DEMONSTRATION", 18., GOLD, false);
        ink.centered(800., 878., "Any input returns to the title", 16., MUTED, false);
    }
    if g.mode == Mode::Playing {
        return None;
    }
    if g.mode == Mode::Title {
        veil(&p, screen, 220, 18);
        ink.text(96., 103., "A PILGRIM'S TALE", 19., GOLD, false);
        ink.line(96., 142., 55., GOLD);
        ink.text(89., 180., "DENTED", 79., IVORY, true);
        ink.text(89., 266., "DESTINY", 79., IVORY, true);
        ink.text(98., 377., "THE BELLWETHER ABBEY", 20., GOLD, false);
        ink.text(98., 421., "The crown is broken.\nThe oath remains.", 24., MUTED, true);
        ink.text(98., 851., "AN ORIGINAL SOULS-INSPIRED PILGRIMAGE", 14., MUTED, false);
        ink.text(1172., 853., &format!("THE ABBEY WAKES IN {:02}", (10. - g.idle).ceil().max(0.) as i32), 15., GOLD, false);
    } else {
        veil(&p, screen, 228, 170);
        let title = match g.mode {
            Mode::Pause => "A moment of stillness",
            Mode::Rest => "SANCTUARY",
            Mode::Settings => "Shape your pilgrimage",
            Mode::Guide => "The pilgrim's way",
            Mode::Journal => "What the silence holds",
            Mode::Dead => "YOUR OATH ENDURES",
            Mode::Victory => "LET THE SILENCE END",
            _ => "",
        };
        ink.text(
            100.,
            89.,
            if matches!(g.mode, Mode::Dead | Mode::Victory) { "DENTED DESTINY" } else { "THE BELLWETHER ABBEY" },
            17.,
            GOLD,
            false,
        );
        ink.text(96., 135., title, 49., if g.mode == Mode::Dead { Color32::from_rgb(180, 106, 85) } else { IVORY }, true);
        ink.line(100., 213., 1250., Color32::from_white_alpha(40));
        match g.mode {
            Mode::Rest => {
                ink.text(102., 249., "Your flask is filled. Your wounds are healed.", 25., IVORY, true);
                ink.text(102., 291., "Rest renews the fallen. Spend their echoes to strengthen your oath.", 21., MUTED, false);
                ink.text(
                    1050.,
                    264.,
                    &format!("{} ECHOES\nVitality {}  /  Edge {}", g.progress.echoes, g.progress.vigor + 1, g.progress.edge + 1),
                    25.,
                    GOLD,
                    false,
                );
            }
            Mode::Guide => {
                let left = if pad {
                    [
                        ("LEFT STICK", "Move"),
                        ("RIGHT STICK", "Orbit camera"),
                        ("R1 / R2", "Light strike / heavy strike"),
                        ("○", "Evade · briefly invulnerable"),
                        ("L1 / L2", "Guard / timed parry"),
                        ("□", "Drink amber flask"),
                    ]
                } else {
                    [
                        ("W A S D", "Move"),
                        ("MIDDLE DRAG", "Orbit camera"),
                        ("J / K · MOUSE", "Light strike / heavy strike"),
                        ("SPACE", "Evade · briefly invulnerable"),
                        ("SHIFT / F", "Guard / timed parry"),
                        ("X", "Drink amber flask"),
                    ]
                };
                for (i, (key, desc)) in left.iter().enumerate() {
                    let y = 263. + i as f32 * 58.;
                    ink.text(104., y, key, 20., GOLD, false);
                    ink.text(340., y, desc, 24., IVORY, false);
                }
                let right = if pad {
                    [
                        ("×", "Interact / confirm"),
                        ("R3", "Lock to nearest enemy"),
                        ("△", "Switch weapon"),
                        ("OPTIONS", "Pause"),
                        ("L3", "Jump"),
                        ("D-PAD", "Snap menu focus"),
                    ]
                } else {
                    [
                        ("E / ENTER", "Interact / confirm"),
                        ("R", "Lock to nearest enemy"),
                        ("Q", "Switch weapon"),
                        ("ESC", "Pause"),
                        ("V", "Jump"),
                        ("ARROWS", "Snap menu focus"),
                    ]
                };
                for (i, (key, desc)) in right.iter().enumerate() {
                    let y = 263. + i as f32 * 58.;
                    ink.text(850., y, key, 20., GOLD, false);
                    ink.text(1070., y, desc, 24., IVORY, false);
                }
                ink.text(104., 650., "Watch the windup. Spend stamina with intent. Strike during recovery.", 25., IVORY, true);
                ink.text(
                    104.,
                    695.,
                    "Death leaves your echoes behind. Reach them before falling again. Rest to refill your flask.",
                    21.,
                    MUTED,
                    false,
                );
            }
            Mode::Journal => {
                ink.text(103., 256., "YOUR OATH", 18., GOLD, false);
                ink.text(103., 298., g.objective(), 26., IVORY, true);
                ink.text(103., 361., "ARMS & BLESSINGS", 18., GOLD, false);
                ink.text(103., 400., g.progress.weapon_name(), 28., IVORY, true);
                ink.text(
                    103.,
                    448.,
                    &format!(
                        "Vitality {}   ·   Edge {}   ·   {} echoes",
                        g.progress.vigor + 1,
                        g.progress.edge + 1,
                        g.progress.echoes
                    ),
                    23.,
                    MUTED,
                    false,
                );
                ink.text(
                    103.,
                    512.,
                    if g.progress.relic {
                        "Vesper's seal: +20 stamina. Q / △ changes your blade."
                    } else {
                        "A sealed reliquary waits in the western court."
                    },
                    22.,
                    IVORY,
                    false,
                );
                ink.text(850., 256., "MEMORIES", 18., GOLD, false);
                let lines = [
                    if g.progress.lore & 1 != 0 {
                        "The guardian laid down his blade.\nThe abbey mistook his mercy for an oath."
                    } else {
                        "A memory sleeps beneath the western seal."
                    },
                    if g.progress.lore & 2 != 0 {
                        "They broke every bell but one,\nso no pilgrim would wake the guardian."
                    } else {
                        "A memory waits where the ash cantor sings."
                    },
                    if g.progress.guardian {
                        "At last the silence belongs to no one."
                    } else {
                        "Beyond the mist, a silence keeps watch."
                    },
                ];
                for (i, line) in lines.iter().enumerate() {
                    ink.text(850., 306. + i as f32 * 111., line, 24., IVORY, true);
                }
                ink.text(
                    103.,
                    622.,
                    &format!(
                        "{} fallen retainers  ·  {} returns from death\nWest passage: {}",
                        g.progress.kills,
                        g.progress.deaths,
                        if g.progress.shortcut { "opened" } else { "sealed" }
                    ),
                    22.,
                    MUTED,
                    false,
                );
            }
            Mode::Dead => {
                ink.text(102., 271., "The flame remembers what the world forgot.", 29., IVORY, true);
                ink.text(
                    102.,
                    330.,
                    &format!(
                        "{} echoes wait where you fell.\nYour blessings and opened passages endure.",
                        g.blood.map(|(_, v)| v).unwrap_or(0)
                    ),
                    24.,
                    MUTED,
                    false,
                );
            }
            Mode::Victory => {
                ink.text(102., 265., "One broken crown. One promise kept.", 30., GOLD, true);
                ink.text(102.,324.,"The abbey hears a bell for the first time in a hundred years.\nYou leave its silence lighter than you found it.",27.,IVORY,true);
                ink.text(
                    102.,
                    425.,
                    &format!(
                        "Pilgrimage  {:02}:{:02}    ·    {} fallen    ·    {} returns",
                        (g.time / 60.) as u32,
                        g.time as u32 % 60,
                        g.progress.kills,
                        g.progress.deaths
                    ),
                    23.,
                    MUTED,
                    false,
                );
            }
            Mode::Settings => {
                ink.text(860., 300., "PILGRIM AID", 18., GOLD, false);
                ink.text(860., 346., "Less incoming damage.\nThe same pilgrimage.", 27., IVORY, true);
                ink.text(860.,459.,"Camera motion controls impact shake.\nHigh contrast lifts dark surfaces.\nTactical camera gives a wider view.",22.,MUTED,false);
            }
            Mode::Pause => {
                ink.text(900., 300., "THE OATH", 18., GOLD, false);
                ink.text(900., 345., g.objective(), 24., IVORY, true);
                ink.text(900., 411., "Take a breath. The world will wait.", 24., MUTED, true);
            }
            _ => {}
        }
    }
    let y0 = match g.mode {
        Mode::Title => 551.,
        Mode::Guide | Mode::Journal => 759.,
        Mode::Dead => 493.,
        Mode::Victory => 542.,
        Mode::Rest => 366.,
        _ => 274.,
    };
    for (i, item) in g.menu_items().iter().enumerate() {
        let y = y0 + i as f32 * 61.;
        let width = if g.mode == Mode::Rest { 680. } else { 600. };
        let r = Rect::from_min_size(ink.at(96., y), Vec2::new(width, 52.) * k);
        let response = ui.interact(r, egui::Id::new(("dented_item", i)), egui::Sense::click());
        if response.clicked() {
            selected = Some(i);
        }
        let focus = g.menu == i || response.hovered();
        if focus {
            ink.rect(96., y, width, 52., Color32::from_rgba_unmultiplied(126, 109, 65, 38));
            ink.line(96., y, width, Color32::from_rgba_unmultiplied(185, 158, 96, 120));
            ink.line(96., y + 52., width, Color32::from_rgba_unmultiplied(185, 158, 96, 70));
            ink.diamond(111., y + 26., 4., GOLD);
        }
        ink.text(137., y + 11., item, 27., if focus { IVORY } else { MUTED }, true);
    }
    if g.mode != Mode::Title {
        ink.text(
            103.,
            850.,
            if pad {
                "D-PAD  Choose      ×  Confirm      ○  Return"
            } else {
                "↑ ↓  Choose      Enter  Confirm      Backspace  Return"
            },
            19.,
            MUTED,
            false,
        );
        if g.notice_time > 0. {
            ink.text(850., 770., &g.notice, 20., GOLD, false);
        }
    }
    selected
}
