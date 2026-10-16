//! Original procedural art for Dented Destiny, using Pavilion's instanced WebGPU meshes.
use glam::{Mat4, Quat, Vec3};
use pav_core::{
    RenderFrame,
    color::Color,
    games::dented::{Action, Dented, Intent, Kind, Mode},
};
use pav_render::{
    MeshInstance, MeshKey, Scene, Style,
    scene::{PointLight, flags},
};

fn color(s: &str) -> Vec3 {
    Vec3::from_array(Color::hex(s).0)
}
struct Art<'a> {
    s: &'a mut Scene,
    group: u32,
    mats: [u32; 5],
}
impl Art<'_> {
    fn mesh(&mut self, key: MeshKey, p: Vec3, size: Vec3, r: Quat, c: &str, glow: f32) {
        let stone = [
            "#77796c", "#979589", "#83877b", "#a5a391", "#a09b88", "#8e8b7a", "#555e59", "#686c60", "#485451", "#747866",
            "#3c4947", "#626a5f", "#3b4542", "#777564", "#90907b", "#7f8478", "#929482",
        ]
        .contains(&c);
        self.s.meshes.push(MeshInstance {
            mesh: key,
            transform: Mat4::from_scale_rotation_translation(size, r, p),
            color: color(c),
            emissive: glow,
            style: Style::Lit,
            flags: if self.group > 1 { flags::OBJECT | flags::NO_CUT } else { 0 },
            group: self.group,
            material: if glow > 0.0 || matches!(key, MeshKey::Tuft) {
                0
            } else if ["#9f8856", "#b49a59", "#c5c9b5", "#a3a9a1", "#8d8c77", "#737c70"].contains(&c) {
                self.mats[3]
            } else if ["#472e2d", "#613d37", "#272d31", "#52594a"].contains(&c) {
                self.mats[4]
            } else if self.group == 1 && size.y < 0.1 && size.x > 1.0 {
                self.mats[0]
            } else if self.group == 1 && stone && matches!(key, MeshKey::Cylinder) {
                self.mats[2]
            } else if self.group == 1 && stone && size.y > 0.2 && glow == 0.0 {
                self.mats[1]
            } else {
                0
            },
            ext: Vec3::ZERO,
        });
    }
    fn box_at(&mut self, p: Vec3, size: Vec3, c: &str) {
        self.mesh(MeshKey::Cube, p, size, Quat::IDENTITY, c, 0.);
    }
    fn sphere(&mut self, p: Vec3, size: Vec3, c: &str, glow: f32) {
        self.mesh(MeshKey::Sphere, p, size, Quat::IDENTITY, c, glow);
    }
    fn cyl(&mut self, p: Vec3, r: f32, h: f32, c: &str) {
        self.mesh(MeshKey::Cylinder, p, Vec3::new(r * 2., h, r * 2.), Quat::IDENTITY, c, 0.);
    }
    fn beam(&mut self, a: Vec3, b: Vec3, r: f32, c: &str) {
        let d = b - a;
        self.mesh(
            MeshKey::Cylinder,
            (a + b) * 0.5,
            Vec3::new(r * 2., d.length(), r * 2.),
            Quat::from_rotation_arc(Vec3::Y, d.normalize_or_zero()),
            c,
            0.,
        );
    }
    fn ring(&mut self, p: Vec3, r: f32, width: f32, c: &str, glow: f32, vertical: bool) {
        for i in 0..56 {
            let a = i as f32 * std::f32::consts::TAU / 56.;
            let off = if vertical { Vec3::new(a.sin() * r, a.cos() * r, 0.) } else { Vec3::new(a.sin() * r, 0., a.cos() * r) };
            let q = if vertical { Quat::from_rotation_z(-a) } else { Quat::from_rotation_y(a) };
            self.mesh(MeshKey::Cube, p + off, Vec3::new(r * 0.12, width, width), q, c, glow);
        }
    }
    fn pillar(&mut self, p: Vec3, h: f32, r: f32) {
        self.box_at(p + Vec3::Y * 0.2, Vec3::new(r * 2.8, 0.4, r * 2.8), "#77796c");
        self.cyl(p + Vec3::Y * 0.55, r * 1.15, 0.3, "#979589");
        self.cyl(p + Vec3::Y * (h * 0.5), r, h - 1., "#83877b");
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::TAU / 8.;
            self.beam(
                p + Vec3::new(a.sin() * r, 0.8, a.cos() * r),
                p + Vec3::new(a.sin() * r, h - 0.8, a.cos() * r),
                0.055,
                "#a5a391",
            );
        }
        self.cyl(p + Vec3::Y * (h - 0.6), r * 1.2, 0.25, "#a09b88");
        self.box_at(p + Vec3::Y * (h - 0.3), Vec3::new(r * 2.8, 0.4, r * 2.8), "#77796c");
    }
    fn arch(&mut self, p: Vec3, r: f32, h: f32) {
        for side in [-1., 1.] {
            self.pillar(p + Vec3::X * r * side, h, 0.5);
        }
        for i in 0..17 {
            let a = i as f32 * std::f32::consts::PI / 16.;
            let at = p + Vec3::new(a.cos() * r, h + a.sin() * r, 0.);
            self.mesh(MeshKey::Cube, at, Vec3::new(0.62, r * 0.21, 0.85), Quat::from_rotation_z(a), "#8e8b7a", 0.);
        }
    }
    fn flame(&mut self, p: Vec3, t: f32) {
        self.cyl(p, 0.3, 0.22, "#453e2d");
        for i in 0..4 {
            let u = i as f32;
            self.sphere(
                p + Vec3::new((t * 3. + u).sin() * 0.06, 0.2 + u * 0.13, 0.),
                Vec3::new(0.25 - u * 0.045, 0.5, 0.25 - u * 0.045),
                if i < 2 { "#f0aa4f" } else { "#ffe7a1" },
                1.8,
            );
        }
    }
}
fn hash(n: u32) -> f32 {
    let mut a = n.wrapping_mul(747796405).wrapping_add(2891336453);
    a = ((a >> ((a >> 28) + 4)) ^ a).wrapping_mul(277803737);
    ((a >> 22) ^ a) as f32 / u32::MAX as f32
}

fn world(a: &mut Art<'_>, g: &Dented, t: f32) {
    a.group = 1;
    // Worn masonry: visible paving joints, scattered grit and fractured slabs.
    for z in -13_i32..14 {
        for x in -9_i32..10 {
            let seed = ((x + 20) * 971 + (z + 20) * 1237) as u32;
            let h = hash(seed);
            let p = Vec3::new(x as f32 * 2., 0.025, z as f32 * 2.);
            a.mesh(
                MeshKey::Cube,
                p,
                Vec3::new(1.94, 0.05, 1.94),
                Quat::from_rotation_y((h - 0.5) * 0.016),
                if x.abs() <= 1 {
                    if h < 0.5 { "#878675" } else { "#7a7c70" }
                } else if h < 0.3 {
                    "#535e52"
                } else {
                    "#6d7167"
                },
                0.,
            );
            if h > 0.55 && x.abs() > 2 {
                for j in 0..3 {
                    let q = p + Vec3::new(hash(seed + j) * 1.7 - 0.85, 0.15, hash(seed + j + 89) * 1.7 - 0.85);
                    a.mesh(
                        MeshKey::Tuft,
                        q,
                        Vec3::new(0.3, 0.35 + hash(seed + j + 4) * 0.4, 0.3),
                        Quat::from_rotation_y(h * 6.),
                        "#657354",
                        0.,
                    );
                }
            }
            if h < 0.11 {
                a.mesh(
                    MeshKey::Cube,
                    p + Vec3::new(0.5, 0.07, 0.2),
                    Vec3::new(0.5, 0.08, 0.18),
                    Quat::from_rotation_y(h * 58.),
                    "#929482",
                    0.,
                );
            }
        }
    }
    // The repeated nave establishes scale and leads the eye to the bell.
    for z in [-23., -15., -6., 4., 14., 24.] {
        for x in [-7., 7.] {
            let h = if z == 14. && x < 0. { 3.4 } else { 7.2 };
            a.pillar(Vec3::new(x, 0., z), h, 0.48);
            if h < 5. {
                a.mesh(
                    MeshKey::Cylinder,
                    Vec3::new(x - 1.6, 0.65, z + 1.),
                    Vec3::new(1.0, 3.2, 1.0),
                    Quat::from_rotation_z(1.4),
                    "#7f8478",
                    0.,
                );
            }
            if z != -23. {
                a.box_at(Vec3::new(x, 5.7, z + 0.45), Vec3::new(1.5, 0.3, 0.2), "#92734a");
                for k in 0..9 {
                    let v = k as f32;
                    let q = Vec3::new(x + (t * 1.8 + v * 0.4 + z).sin() * v * 0.014, 5.5 - v * 0.28, z + 0.55);
                    a.box_at(q, Vec3::new(1.1 - v * 0.035, 0.3, 0.055), "#472e2d");
                    a.box_at(q + Vec3::Z * 0.04, Vec3::new(0.08, 0.29, 0.02), "#ad9052");
                }
            }
        }
    }
    a.arch(Vec3::new(0., 0., 3.), 3., 4.2);
    a.arch(Vec3::new(0., 0., -12.), 4., 5.2);
    for (x, open) in [(0., g.progress.gate), (-12., g.progress.shortcut)] {
        if !open {
            for n in -6..=6 {
                let px = x + n as f32 * 0.35;
                a.beam(Vec3::new(px, 0., 3.), Vec3::new(px, 3.3, 3.), 0.045, "#8e794c");
                a.mesh(MeshKey::Cone, Vec3::new(px, 3.5, 3.), Vec3::new(0.16, 0.5, 0.16), Quat::IDENTITY, "#b29a61", 0.);
            }
            a.box_at(Vec3::new(x, 0.8, 3.), Vec3::new(4.6, 0.12, 0.1), "#6d664d");
            a.box_at(Vec3::new(x, 2.4, 3.), Vec3::new(4.6, 0.12, 0.1), "#6d664d");
            a.ring(Vec3::new(x, 1.65, 3.1), 0.48, 0.07, "#c4ab67", 0., true);
        }
    }
    // Broken perimeter and far-away buttressed cathedral: silhouettes remain visible from the court.
    for x in [-21., 21.] {
        for z in (-26..28).step_by(6) {
            let p = Vec3::new(x, 0., z as f32);
            a.box_at(p + Vec3::Y * 3., Vec3::new(1.8, 6., 2.), "#555e59");
            a.mesh(MeshKey::Cone, p + Vec3::Y * 6.6, Vec3::new(2.3, 2.1, 2.3), Quat::IDENTITY, "#686c60", 0.);
        }
    }
    for x in [-13., 13.] {
        let p = Vec3::new(x, 0., -30.);
        a.box_at(p + Vec3::Y * 7., Vec3::new(7., 14., 5.), "#485451");
        for y in [5., 9., 13.] {
            a.box_at(p + Vec3::new(0., y, 2.6), Vec3::new(7.6, 0.25, 0.5), "#747866");
            for dx in [-2., 0., 2.] {
                a.box_at(p + Vec3::new(dx, y - 1.7, 2.53), Vec3::new(0.7, 2.3, 0.05), "#202c2b");
            }
        }
        a.mesh(MeshKey::Cone, p + Vec3::Y * 17., Vec3::new(8., 8., 7.), Quat::IDENTITY, "#3c4947", 0.);
        a.beam(p + Vec3::Y * 18., p + Vec3::Y * 23., 0.12, "#9e8855");
    }
    a.arch(Vec3::new(0., 0., -27.), 4., 10.);
    // Suspended bronze bell, clapper and weathered dais.
    a.cyl(g.rules.bell + Vec3::Y * 0.16, 2.8, 0.3, "#777564");
    a.cyl(g.rules.bell + Vec3::Y * 0.38, 2.3, 0.2, "#90907b");
    a.mesh(MeshKey::Cone, g.rules.bell + Vec3::Y * 8., Vec3::new(3.3, 3.4, 3.3), Quat::IDENTITY, "#897449", 0.);
    a.cyl(g.rules.bell + Vec3::Y * 6.45, 1.8, 0.22, "#b0995c");
    a.cyl(g.rules.bell + Vec3::Y * 8.9, 0.7, 0.3, "#b0995c");
    a.beam(g.rules.bell + Vec3::Y * 6., g.rules.bell + Vec3::Y * 9., 0.16, "#3a372b");
    a.sphere(g.rules.bell + Vec3::Y * 6.0, Vec3::splat(0.5), "#4c4633", 0.);
    // A broken celestial ring, and distant mountains, drawn in geometry rather than a sky image.
    a.ring(Vec3::new(-17., 27., -75.), 10., 0.12, "#bdac72", 0.55, true);
    for i in 0..12 {
        let n = i as f32;
        a.mesh(
            MeshKey::Cone,
            Vec3::new(-85. + n * 15., -6., -92. - (n * 1.2).sin() * 8.),
            Vec3::new(33., 30. + hash(i) * 25., 25.),
            Quat::from_rotation_y(n),
            "#334947",
            0.,
        );
    }
    // Dead trees: tapered trunk segments and angular branches, original procedural silhouettes.
    for (x, z, s) in [(-17., 18., 0.8), (17., -8., 1.2), (-26., -17., 1.8), (24., 20., 1.3)] {
        let p = Vec3::new(x, 0., z);
        a.beam(p, p + Vec3::new(0.4, 5. * s, 0.), 0.32 * s, "#454b3e");
        for k in 0..7 {
            let q = k as f32;
            let root = p + Vec3::new(0.3, 2.4 * s + q * 0.35, 0.);
            let tip = root + Vec3::new((q * 2.4).cos() * 2.5 * s, 1.6 * s, (q * 2.4).sin() * 2. * s);
            a.beam(root, tip, 0.12 * s, "#535443");
            a.beam(tip, tip + Vec3::new(0.5 * q.sin(), 1.4 * s, 0.2), 0.05 * s, "#666650");
        }
    }
    for (i, p) in g.rules.shrines.iter().enumerate() {
        a.cyl(*p + Vec3::Y * 0.12, 1., 0.24, "#3b4542");
        a.ring(*p + Vec3::Y * 0.25, 0.9, 0.045, "#d5bb74", 0.5, false);
        a.cyl(*p + Vec3::Y * 0.6, 0.15, 0.9, "#a89768");
        a.flame(*p + Vec3::Y * 0.9, t);
        a.s.point_lights.push(PointLight {
            position: *p + Vec3::Y * 1.4,
            color: color("#ffd18b") * 3.,
            radius: 7.,
            shadows: i == 0,
        });
        for j in 0..16 {
            let n = j as f32;
            let at = *p
                + Vec3::new(
                    (n * 2.4 + t * 0.3).sin() * 0.65,
                    0.5 + (t * 0.3 + n * 0.17).fract() * 1.7,
                    (n * 2.4 + t * 0.3).cos() * 0.65,
                );
            a.sphere(at, Vec3::splat(0.035), "#ffe0a0", 1.8);
        }
    }
    for p in [Vec3::new(-4., 0., 3.), Vec3::new(4., 0., 3.), Vec3::new(-5., 0., -12.), Vec3::new(5., 0., -12.)] {
        a.cyl(p + Vec3::Y * 1., 0.15, 2., "#655d48");
        a.flame(p + Vec3::Y * 2., t);
    }
    // Interactable objects retain clear visual identities.
    let r = g.rules.reliquary;
    a.box_at(r + Vec3::Y * 0.45, Vec3::new(1.5, 0.9, 0.9), "#433b2e");
    a.mesh(
        MeshKey::Cube,
        r + Vec3::new(0., if g.progress.relic { 1.25 } else { 0.96 }, if g.progress.relic { -0.35 } else { 0. }),
        Vec3::new(1.58, 0.18, 0.96),
        Quat::from_rotation_x(if g.progress.relic { -1.1 } else { 0. }),
        "#918054",
        0.,
    );
    for x in [-0.55, 0.55] {
        a.box_at(r + Vec3::new(x, 0.48, 0.47), Vec3::new(0.1, 0.95, 0.04), "#b69c60");
    }
    let c = g.rules.chime;
    a.cyl(c + Vec3::Y * 0.45, 0.8, 0.9, "#626a5f");
    if !g.progress.chime {
        a.ring(c + Vec3::Y * 1.6, 0.4, 0.08, "#e1c277", 1., true);
        a.sphere(c + Vec3::Y * 1.6, Vec3::splat(0.2), "#ffe5a5", 1.5);
    }
    a.cyl(g.rules.lever + Vec3::Y * 0.45, 0.5, 0.9, "#5e665b");
    a.beam(
        g.rules.lever + Vec3::Y * 0.8,
        g.rules.lever + Vec3::new(if g.progress.shortcut { 0.6 } else { -0.6 }, 1.6, 0.),
        0.1,
        "#c0a46c",
    );
    if !g.progress.guardian {
        for i in 0..18 {
            let n = i as f32;
            let p = g.rules.fog + Vec3::new(-3. + n * 0.35, 1.3 + (t * 0.5 + n).sin() * 0.2, 0.);
            a.mesh(MeshKey::Cube, p, Vec3::new(0.035, 2.5, 0.02), Quat::IDENTITY, "#a9beac", 0.4);
        }
    }
    if let Some((p, n)) = g.blood {
        if n > 0 {
            a.ring(p + Vec3::Y * 0.08, 0.55, 0.05, "#69d7ba", 1., false);
            a.sphere(p + Vec3::Y * (0.9 + t.sin() * 0.1), Vec3::new(0.15, 0.4, 0.15), "#9effd0", 1.5);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn knight(
    a: &mut Art<'_>,
    pos: Vec3,
    yaw: f32,
    scale: f32,
    t: f32,
    moving: f32,
    action: Action,
    phase: f32,
    guard: bool,
    kind: Option<Kind>,
    flash: f32,
    great: bool,
) {
    let boss = kind == Some(Kind::Guardian);
    let caster = kind == Some(Kind::Cantor);
    let lancer = kind == Some(Kind::Lancer);
    let cloak = if boss {
        "#272d31"
    } else if kind.is_some() {
        "#52594a"
    } else {
        "#613d37"
    };
    let steel = if flash > 0.15 {
        "#fff2c3"
    } else if boss {
        "#8d8c77"
    } else if kind.is_some() {
        "#737c70"
    } else {
        "#a3a9a1"
    };
    let gold = if boss { "#b49a59" } else { "#9f8856" };
    let roll = if action == Action::Dodge { Quat::from_rotation_x(phase * std::f32::consts::TAU) } else { Quat::IDENTITY };
    let body = Quat::from_rotation_y(yaw) * roll;
    let pivot = pos + Vec3::Y * if action == Action::Dodge { 0.6 * scale } else { 0. };
    let at = |v: Vec3| pivot + body * (v * scale);
    let leg = (t * 9.).sin() * 0.45 * moving.min(1.);
    let breath = (t * 2.).sin() * 0.025;
    for (x, sgn) in [(-0.19, -1.), (0.19, 1.)] {
        let hip = Vec3::new(x, 0.85, 0.);
        let knee = Vec3::new(x, 0.5, leg * sgn);
        let ankle = Vec3::new(x, 0.13, -leg * sgn * 0.65);
        a.beam(at(hip), at(knee), 0.115 * scale, steel);
        a.beam(at(knee), at(ankle), 0.095 * scale, steel);
        a.mesh(MeshKey::Cube, at(ankle + Vec3::Z * 0.08), Vec3::new(0.23, 0.19, 0.4) * scale, body, "#343b37", 0.);
        a.sphere(at(knee), Vec3::new(0.22, 0.24, 0.2) * scale, gold, 0.);
    }
    a.mesh(MeshKey::Sphere, at(Vec3::new(0., 1.18 + breath, 0.)), Vec3::new(0.64, 0.77, 0.4) * scale, body, steel, 0.);
    a.mesh(MeshKey::Cube, at(Vec3::new(0., 0.87, 0.)), Vec3::new(0.61, 0.12, 0.45) * scale, body, gold, 0.);
    a.mesh(MeshKey::Cone, at(Vec3::new(0., 0.76, 0.)), Vec3::new(0.65, 0.55, 0.47) * scale, body, cloak, 0.);
    a.mesh(MeshKey::Cube, at(Vec3::new(0., 1.27, 0.2)), Vec3::new(0.08, 0.5, 0.04) * scale, body, gold, 0.);
    a.mesh(MeshKey::Cube, at(Vec3::new(0., 1.38, 0.21)), Vec3::new(0.36, 0.065, 0.04) * scale, body, gold, 0.);
    // Segmented cape follows motion with a delayed wave.
    for i in 0..8 {
        let n = i as f32;
        let p = Vec3::new((t * 3. - n * 0.45).sin() * n * 0.009, 1.45 - n * 0.13, -0.23 - n * 0.035 - moving * 0.1);
        a.mesh(MeshKey::Cube, at(p), Vec3::new(0.57 + n * 0.015, 0.15, 0.045) * scale, body, cloak, 0.);
    }
    a.sphere(at(Vec3::new(0., 1.75 + breath, 0.)), Vec3::new(0.42, 0.5, 0.4) * scale, steel, 0.);
    a.mesh(MeshKey::Cube, at(Vec3::new(0., 1.79, 0.195)), Vec3::new(0.31, 0.045, 0.035) * scale, body, "#1f2827", 0.);
    a.mesh(MeshKey::Cube, at(Vec3::new(0., 1.67, 0.207)), Vec3::new(0.035, 0.24, 0.035) * scale, body, gold, 0.);
    for x in [-0.09, 0.09] {
        a.sphere(
            at(Vec3::new(x, 1.79, 0.218)),
            Vec3::splat(0.034 * scale),
            if kind.is_some() { "#ffc070" } else { "#c8ddcc" },
            1.3,
        );
    }
    if boss {
        for x in [-0.23, 0.23] {
            a.beam(at(Vec3::new(x, 1.88, 0.)), at(Vec3::new(x * 1.7, 2.4, -0.1)), 0.065 * scale, gold);
        }
        a.ring(at(Vec3::new(0., 2.15, -0.2)), 0.43 * scale, 0.025 * scale, gold, 0.3, true);
    } else {
        a.mesh(MeshKey::Cube, at(Vec3::new(0., 1.99, -0.04)), Vec3::new(0.07, 0.16, 0.3) * scale, body, gold, 0.);
    }
    let swing = match action {
        Action::Light => ((phase * std::f32::consts::PI * 1.6).sin() * 1.6) - 0.3,
        Action::Heavy => {
            if phase < 0.45 {
                -1.7 * phase / 0.45
            } else {
                -1.7 + (phase - 0.45) * 6.
            }
        }
        _ => 0.,
    };
    let wrist = Vec3::new(
        0.48 + 0.15 * swing.sin(),
        1.0 + if matches!(action, Action::Heavy) { swing.abs() * 0.32 } else { 0. },
        0.2 + swing.cos() * 0.16,
    );
    for side in [-1., 1.] {
        let shoulder = Vec3::new(side * 0.39, 1.45, 0.);
        let hand =
            if side > 0. { wrist } else { Vec3::new(-0.47, if guard { 1.3 } else { 1.0 }, if guard { 0.48 } else { 0.17 }) };
        a.sphere(at(shoulder), Vec3::new(0.35, 0.29, 0.38) * scale, steel, 0.);
        a.beam(at(shoulder), at(hand), 0.095 * scale, steel);
        a.sphere(at(hand), Vec3::splat(0.18 * scale), "#44483c", 0.);
    }
    let sr = body
        * Quat::from_rotation_x(if lancer || caster { -0.25 } else { -0.45 + swing })
        * Quat::from_rotation_z(if matches!(action, Action::Light) { swing } else { -0.15 });
    let hand = at(wrist);
    let length = if great || boss {
        1.75
    } else if lancer || caster {
        2.3
    } else {
        1.25
    };
    a.mesh(
        MeshKey::Cube,
        hand + sr * Vec3::Y * (length * 0.44 * scale),
        Vec3::new(if great || boss { 0.19 } else { 0.09 }, length, 0.045) * scale,
        sr,
        if caster { "#786443" } else { "#c5c9b5" },
        0.,
    );
    a.mesh(MeshKey::Cube, hand + sr * Vec3::Y * 0.17 * scale, Vec3::new(0.43, 0.07, 0.1) * scale, sr, gold, 0.);
    if caster {
        a.sphere(hand + sr * Vec3::Y * (length * scale), Vec3::splat(0.25 * scale), "#98ccb2", 1.5);
    } else {
        a.mesh(
            MeshKey::Cone,
            hand + sr * Vec3::Y * (length * 0.97 * scale),
            Vec3::new(0.15, 0.3, 0.06) * scale,
            sr,
            "#d8d8c3",
            0.,
        );
    }
    if !caster && !lancer {
        let sh = at(Vec3::new(-0.55, if guard { 1.2 } else { 1.0 }, if guard { 0.5 } else { 0.1 }));
        let q = body * Quat::from_rotation_y(if guard { 0.2 } else { -0.65 });
        a.mesh(MeshKey::Sphere, sh, Vec3::new(0.55, 0.83, 0.14) * scale, q, gold, 0.);
        a.mesh(MeshKey::Sphere, sh + q * Vec3::Z * 0.05 * scale, Vec3::new(0.47, 0.73, 0.11) * scale, q, cloak, 0.);
        a.mesh(MeshKey::Cube, sh + q * Vec3::Z * 0.12 * scale, Vec3::new(0.06, 0.65, 0.025) * scale, q, gold, 0.);
    }
}

pub fn emit(
    scene: &mut Scene,
    prev: &RenderFrame,
    curr: &RenderFrame,
    alpha: f32,
    time: f32,
    materials: &mut crate::materials::MaterialTable,
) {
    let Some(g) = curr.hud.dented.as_deref() else {
        return;
    };
    let lib = pav_core::material::Library::current();
    let names = ["MAT/abbey_flagstones", "MAT/dungeon_wall", "MAT/pillar_stone", "MAT/pb/iron", "MAT/abbey_cloth"];
    let mats = names.map(|name| {
        materials
            .resolve(
                &lib,
                name,
                Some(pav_core::props::doc::MapMode::World),
                &[("$floor", "#6e6c51"), ("$wall", "#646e67"), ("$pillar", "#929482")],
            )
            .map(|s| s.param(0))
            .unwrap_or(0)
    });
    for mesh in &mut scene.meshes {
        if mesh.group == 1 {
            mesh.material = if mesh.transform.y_axis.truncate().length() > 0.8 { mats[1] } else { mats[0] };
        }
    }
    let mut a = Art { s: scene, group: 1, mats };
    world(&mut a, g, time);
    let old = prev.hud.dented.as_deref().unwrap_or(g);
    let p = old.player_pos.lerp(g.player_pos, alpha);
    let speed = (g.player_pos - old.player_pos).length() / curr.dt.max(0.01) / 4.6;
    a.group = 10000;
    knight(
        &mut a,
        p,
        g.facing,
        1.,
        time,
        speed,
        g.action,
        1. - g.action_time / g.action_total.max(0.01),
        g.guard,
        None,
        g.flash,
        g.progress.weapon == 1,
    );
    if g.action == Action::Heal {
        a.ring(p + Vec3::Y * 0.15, 0.8, 0.04, "#eac579", 0.8, false);
    }
    for (i, e) in g.enemies.iter().enumerate() {
        if e.hp <= 0. {
            continue;
        }
        a.group = 10001 + i as u32;
        let olde = old.enemies.iter().find(|o| o.id == e.id).unwrap_or(e);
        let p = olde.pos.lerp(e.pos, alpha);
        let move_speed = (e.pos - olde.pos).length() / curr.dt.max(0.01) / 2.;
        let phase = 1. - e.timer / e.duration.max(0.01);
        let (action, ph) = if e.intent == Intent::Windup {
            (Action::Heavy, phase * 0.43)
        } else if e.intent == Intent::Recover {
            (Action::Heavy, 0.43 + phase * 0.57)
        } else {
            (Action::Idle, 0.)
        };
        knight(
            &mut a,
            p,
            e.facing,
            if e.boss() { 2.35 } else { 1.05 },
            time + i as f32,
            move_speed,
            action,
            ph,
            false,
            Some(e.kind),
            e.flash,
            e.boss(),
        );
        if e.intent == Intent::Windup {
            let radius = if e.boss() && e.attack == 1 { e.reach() * phase.max(0.08) } else { 0.85 };
            a.ring(p + Vec3::Y * 0.09, radius, 0.04, if e.boss() && e.phase == 2 { "#f0ab6e" } else { "#ddbd73" }, 0.8, false);
        }
        if e.intent == Intent::Stagger {
            a.ring(p + Vec3::Y * if e.boss() { 4.8 } else { 2.3 }, 0.2, 0.035, "#d6e9cb", 0.5, true);
        }
        if g.locked == Some(i) {
            a.ring(p + Vec3::Y * 0.07, 0.65, 0.04, "#c8e1d8", 0.6, false);
        }
    }
    for bolt in &g.bolts {
        a.sphere(bolt.pos, Vec3::splat(0.25), "#8edab6", 1.8);
        a.beam(bolt.pos, bolt.pos - bolt.vel * 0.09, 0.045, "#c5e8bb");
    }
    // Quiet drifting embers make title and pauses feel alive without mutating gameplay.
    for i in 0..35 {
        let n = i as f32;
        let p = g.player_pos
            + Vec3::new(
                (n * 5.17 + time * 0.1).sin() * 12.,
                0.5 + (n * 0.23 + time * 0.2).rem_euclid(7.),
                (n * 1.71).cos() * 12.,
            );
        a.sphere(p, Vec3::splat(0.022), "#c4bb90", 0.6);
    }
    if g.mode == Mode::Dead {
        a.s.filter.saturation = 0.3;
    }
    if g.options.contrast {
        a.s.post.exposure *= 1.15;
        a.s.filter.contrast = 1.12;
    }
}
