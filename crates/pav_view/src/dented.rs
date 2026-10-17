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
            style: if glow > 0.0 { Style::Unlit } else { Style::Cel },
            flags: if glow > 0.0 {
                flags::NO_SHADOW | flags::NO_CUT
            } else if self.group > 1 {
                flags::OBJECT | flags::NO_CUT
            } else {
                0
            },
            group: self.group,
            material: if self.group > 1 || glow > 0.0 || matches!(key, MeshKey::Tuft) {
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
        let beacon = *p + Vec3::X * 0.68;
        a.cyl(beacon + Vec3::Y * 0.6, 0.15, 0.9, "#a89768");
        a.flame(beacon + Vec3::Y * 0.9, t);
        a.s.point_lights.push(PointLight {
            position: beacon + Vec3::Y * 1.4,
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
    sk: &pav_core::puppet::Skel,
    kind: Option<Kind>,
    flash: f32,
    great: bool,
    heal: bool,
) {
    let boss = kind == Some(Kind::Guardian);
    let caster = kind == Some(Kind::Cantor);
    let lancer = kind == Some(Kind::Lancer);
    let cloak = if boss {
        "#442d68"
    } else if kind.is_some() {
        "#236b69"
    } else {
        "#d64c37"
    };
    let steel = if flash > 0.06 {
        "#fff7cf"
    } else if boss {
        "#858bb4"
    } else if kind.is_some() {
        "#839e99"
    } else {
        "#eee4c8"
    };
    let gold = if boss { "#f3ba53" } else { "#dca34c" };
    let dark = "#233b4a";
    let body = Quat::from_rotation_y(yaw);
    let at = |p: Vec3| pos + body * (p * scale);
    let chest = body * sk.chest_rot;
    let pelvis = body * sk.pelvis_rot;
    let head = body * sk.head_rot;
    let shaped = MeshKey::rounded_box(Vec3::splat(0.5), 0.14);
    // Separate articulated upper/lower limbs: black joints read through ivory armour.
    for i in 0..2 {
        a.beam(at(sk.hip[i]), at(sk.knee[i]), 0.104 * scale, dark);
        a.beam(at(sk.knee[i]), at(sk.ankle[i]), 0.092 * scale, dark);
        let leg_q = body * Quat::from_rotation_arc(Vec3::Y, (sk.knee[i] - sk.ankle[i]).normalize_or(Vec3::Y));
        a.mesh(shaped, at(sk.ankle[i].lerp(sk.knee[i], 0.53)), Vec3::new(0.21, 0.33, 0.23) * scale, leg_q, steel, 0.);
        a.sphere(at(sk.knee[i]), Vec3::new(0.235, 0.21, 0.24) * scale, gold, 0.);
        let foot_q = body * Quat::from_rotation_arc(Vec3::Z, sk.toe[i]);
        a.mesh(shaped, at(sk.ankle[i] + sk.toe[i] * 0.10), Vec3::new(0.24, 0.18, 0.40) * scale, foot_q, dark, 0.);
        a.beam(at(sk.shoulder[i]), at(sk.elbow[i]), 0.082 * scale, dark);
        a.beam(at(sk.elbow[i]), at(sk.hand[i]), 0.074 * scale, dark);
        let forearm = body * Quat::from_rotation_arc(Vec3::Y, (sk.elbow[i] - sk.hand[i]).normalize_or(Vec3::Y));
        a.mesh(shaped, at(sk.hand[i].lerp(sk.elbow[i], 0.56)), Vec3::new(0.19, 0.25, 0.20) * scale, forearm, steel, 0.);
        a.mesh(shaped, at(sk.shoulder[i]), Vec3::new(0.36, 0.25, 0.39) * scale, chest, steel, 0.);
        a.sphere(at(sk.hand[i]), Vec3::splat(0.17 * scale), dark, 0.);
    }
    a.mesh(shaped, at(sk.pelvis.lerp(sk.chest, 0.55)), Vec3::new(0.59, 0.53, 0.38) * scale, chest, steel, 0.);
    a.mesh(shaped, at(sk.pelvis), Vec3::new(0.54, 0.21, 0.36) * scale, pelvis, dark, 0.);
    a.mesh(
        MeshKey::Cube,
        at(sk.pelvis) + pelvis * Vec3::new(0., 0.05, 0.02) * scale,
        Vec3::new(0.57, 0.08, 0.40) * scale,
        pelvis,
        gold,
        0.,
    );
    for side in [-1., 1.] {
        a.mesh(
            shaped,
            at(sk.hip[if side < 0. { 0 } else { 1 }]) + pelvis * Vec3::new(side * 0.04, -0.12, 0.) * scale,
            Vec3::new(0.25, 0.30, 0.38) * scale,
            pelvis * Quat::from_rotation_z(side * 0.12),
            steel,
            0.,
        );
    }
    let breast = at(sk.chest) + chest * Vec3::new(0., -0.16, 0.205) * scale;
    a.mesh(MeshKey::Cube, breast, Vec3::new(0.07, 0.30, 0.03) * scale, chest * Quat::from_rotation_z(-0.35), gold, 0.);
    a.mesh(MeshKey::Cube, breast, Vec3::new(0.25, 0.055, 0.035) * scale, chest * Quat::from_rotation_z(-0.35), gold, 0.);
    a.mesh(MeshKey::Sphere, at(sk.head), Vec3::new(0.45, 0.48, 0.43) * scale, head, steel, 0.);
    a.mesh(
        shaped,
        at(sk.head) + head * Vec3::new(0., 0.015, 0.205) * scale,
        Vec3::new(0.34, 0.092, 0.05) * scale,
        head,
        dark,
        0.,
    );
    a.mesh(
        MeshKey::Cube,
        at(sk.head) + head * Vec3::new(0., -0.065, 0.235) * scale,
        Vec3::new(0.045, 0.24, 0.045) * scale,
        head,
        gold,
        0.,
    );
    for x in [-0.09, 0.09] {
        a.mesh(
            MeshKey::Cube,
            at(sk.head) + head * Vec3::new(x, 0.025, 0.238) * scale,
            Vec3::new(0.06, 0.026, 0.016) * scale,
            head,
            if kind.is_some() { "#ffb749" } else { "#a5f7ec" },
            0.6,
        );
    }
    a.mesh(shaped, at(sk.neck), Vec3::new(0.48, 0.15, 0.44) * scale, chest, cloak, 0.);
    if boss {
        for x in [-0.23, 0.23] {
            a.beam(
                at(sk.head + sk.head_rot * Vec3::new(x, 0.12, 0.)),
                at(sk.head + sk.head_rot * Vec3::new(x * 1.65, 0.67, -0.12)),
                0.05 * scale,
                gold,
            );
        }
    } else {
        a.mesh(
            shaped,
            at(sk.head) + head * Vec3::new(0., 0.24, -0.025) * scale,
            Vec3::new(0.075, 0.14, 0.35) * scale,
            head,
            gold,
            0.,
        );
    }
    // A continuous, tapered cloth surface follows the chest with delayed secondary motion.
    let mut cloth = pav_render::mesh::MeshData::default();
    let corner = |u: f32, side: f32| {
        let wave = (t * 7.0 - u * 5.0).sin() * u * 0.045 + (t * 3.2 - u * 3.0).sin() * u * 0.035;
        let p = sk.chest
            + sk.chest_rot
                * Vec3::new(
                    side * (0.30 + u * 0.07) + wave,
                    -u * 0.95 + side.abs() * u.powi(8) * 0.12,
                    -0.24 - u * (0.13 + moving.min(1.0) * 0.28) + wave * 0.8 + (side * 5.0 + t * 4.0 - u * 4.0).sin() * u * 0.035,
                );
        at(p)
    };
    for i in 0..9 {
        let u = i as f32 / 9.;
        let v = (i + 1) as f32 / 9.;
        for column in 0..4 {
            let left = -1.0 + column as f32 * 0.5;
            let right = left + 0.5;
            let p = [corner(u, left), corner(v, left), corner(v, right), corner(u, right)];
            let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or(Vec3::NEG_Z);
            cloth.push_quad(p, n);
        }
    }
    a.s.dynamic.push(pav_render::scene::DynamicMesh {
        data: cloth,
        color: color(cloak),
        emissive: 0.,
        style: Style::Cel,
        flags: flags::OBJECT | flags::NO_CUT | flags::TWO_SIDED,
        group: a.group,
        material: 0,
    });
    // Attach blade and shield to the actual wrists, including wrist and torso rotation.
    let hand = at(sk.hand[1]);
    let weapon = body * sk.weapon;
    let sr = Quat::from_rotation_arc(Vec3::Y, weapon);
    let length = if great || boss {
        1.65
    } else if lancer || caster {
        2.2
    } else {
        1.20
    };
    a.mesh(shaped, hand - weapon * 0.06 * scale, Vec3::new(0.095, 0.29, 0.095) * scale, sr, dark, 0.);
    a.mesh(shaped, hand + weapon * 0.13 * scale, Vec3::new(0.43, 0.075, 0.12) * scale, sr, gold, 0.);
    a.mesh(
        MeshKey::Cube,
        hand + weapon * (length * 0.5 + 0.13) * scale,
        Vec3::new(if great || boss { 0.22 } else { 0.115 }, length, 0.055) * scale,
        sr,
        if caster { gold } else { "#d4f1ed" },
        0.,
    );
    a.mesh(
        MeshKey::Cube,
        hand + weapon * (length * 0.5 + 0.13) * scale + sr * Vec3::Z * 0.035 * scale,
        Vec3::new(0.026, length * 0.9, 0.008) * scale,
        sr,
        "#fff5d5",
        0.25,
    );
    a.mesh(MeshKey::Cone, hand + weapon * (length + 0.20) * scale, Vec3::new(0.13, 0.23, 0.055) * scale, sr, "#fff5d5", 0.);
    if caster {
        a.sphere(hand + weapon * (length + 0.2) * scale, Vec3::splat(0.24 * scale), "#78e8c7", 1.4);
    }
    if heal {
        let p = at(sk.hand[0]);
        a.sphere(p, Vec3::new(0.16, 0.22, 0.16) * scale, "#ffc469", 0.7);
        a.mesh(MeshKey::Cylinder, p + chest * Vec3::Y * 0.13 * scale, Vec3::new(0.08, 0.08, 0.08) * scale, chest, gold, 0.);
    } else if !caster && !lancer && !great && !boss {
        let q = chest * Quat::from_rotation_y(-0.2);
        let shield = at(sk.hand[0]) + q * Vec3::new(0., 0., 0.08) * scale;
        a.mesh(shaped, shield, Vec3::new(0.50, 0.66, 0.13) * scale, q, dark, 0.);
        a.mesh(shaped, shield + q * Vec3::Z * 0.065 * scale, Vec3::new(0.42, 0.57, 0.05) * scale, q, gold, 0.);
        a.mesh(shaped, shield + q * Vec3::Z * 0.095 * scale, Vec3::new(0.35, 0.49, 0.025) * scale, q, cloak, 0.);
        a.mesh(MeshKey::Cube, shield + q * Vec3::Z * 0.12 * scale, Vec3::new(0.045, 0.40, 0.025) * scale, q, "#f8df9b", 0.);
    }
}

fn sword_trail(a: &mut Art<'_>, g: &Dented, p: Vec3) {
    if !matches!(g.action, Action::Light | Action::Heavy) {
        return;
    }
    let elapsed = g.action_total - g.action_time;
    let spec = pav_core::games::dented_motion::attack(g.action, g.combo, g.progress.weapon == 1);
    if elapsed < spec.contact - 0.075 || elapsed > spec.contact + 0.16 {
        return;
    }
    let q = Quat::from_rotation_y(g.facing);
    let mut mesh = pav_render::mesh::MeshData::default();
    let length = if g.progress.weapon == 1 { 1.85 } else { 1.4 };
    for i in 0..9 {
        let t0 = (elapsed - i as f32 * 0.009).max(0.);
        let t1 = (t0 - 0.009).max(0.);
        let s0 = crate::dented_motion::attack_pose(g.action, g.combo, g.progress.weapon == 1, t0);
        let s1 = crate::dented_motion::attack_pose(g.action, g.combo, g.progress.weapon == 1, t1);
        let width = (1. - i as f32 / 9.) * 0.24;
        let pt = |s: &pav_core::puppet::Skel, d: f32| p + q * (s.hand[1] + s.weapon * d);
        let quad = [pt(&s0, length - width), pt(&s1, length - width * 0.85), pt(&s1, length), pt(&s0, length)];
        let normal = (quad[1] - quad[0]).cross(quad[2] - quad[0]).normalize_or(Vec3::Y);
        mesh.push_quad(quad, normal);
    }
    a.s.dynamic.push(pav_render::scene::DynamicMesh {
        data: mesh,
        color: color(if g.action == Action::Heavy { "#ffc45f" } else { "#a6f5e9" }),
        emissive: 0.8,
        style: Style::Unlit,
        flags: flags::NO_SHADOW | flags::NO_CUT | flags::TWO_SIDED,
        group: 0,
        material: 0,
    });
}

fn impacts(a: &mut Art<'_>, g: &Dented) {
    use pav_core::games::dented_motion::ImpactKind;
    a.group = 0;
    for impact in &g.impacts {
        let t = impact.age;
        let strength = impact.strength();
        let ink = if matches!(impact.kind, ImpactKind::Parry | ImpactKind::Block) { "#8dfbec" } else { "#ffcc63" };
        let camera = a.s.camera.forward;
        let right = camera.cross(Vec3::Y).normalize_or(Vec3::X);
        let up = right.cross(camera).normalize_or(Vec3::Y);
        if t < 0.12 {
            let r = (0.32 + t * 6.) * strength;
            for i in 0..6 {
                let n = i as f32 * std::f32::consts::TAU / 6.;
                let d = right * n.cos() + up * n.sin();
                let length = if i % 2 == 0 { 1.0 } else { 0.6 };
                a.mesh(
                    MeshKey::Cone,
                    impact.pos + d * r * length,
                    Vec3::new(0.10 * (1. - t / 0.12), r * length * 1.9, 0.04),
                    Quat::from_rotation_arc(Vec3::Y, d),
                    "#fff6cd",
                    1.0,
                );
            }
        }
        for i in 0..12 {
            let n = i as f32;
            let direction = (right * (n * 2.4).cos() + up * (n * 2.4).sin() + impact.direction * 0.4).normalize_or(Vec3::Y);
            let speed = (2.5 + hash(i) * 4.) * strength;
            let pos = impact.pos + direction * t * speed - Vec3::Y * t * t * 4.;
            let length = (1. - t / 0.45).max(0.) * (0.18 + strength * 0.2);
            a.mesh(MeshKey::Cone, pos, Vec3::new(0.035, length, 0.035), Quat::from_rotation_arc(Vec3::Y, direction), ink, 0.8);
        }
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
    scene.post.outlines = true;
    scene.post.outline_px = 1.5;
    scene.post.outline_part = pav_render::scene::Part::Objects;
    scene.post.outline_darken = 0.18;
    scene.post.outline_color = Some(color("#182e40"));
    scene.post.outline_normal = 0.25;
    for mesh in &mut scene.meshes {
        if mesh.style != Style::Unlit {
            mesh.style = Style::Cel;
        }
        if mesh.group == 1 {
            mesh.material = if mesh.transform.y_axis.truncate().length() > 0.8 { mats[1] } else { mats[0] };
        }
    }
    let mut a = Art { s: scene, group: 1, mats };
    world(&mut a, g, time);
    let old = prev.hud.dented.as_deref().unwrap_or(g);
    let p = old.player_pos.lerp(g.player_pos, alpha);
    a.group = 10000;
    let pose = crate::dented_motion::player(g, old, alpha);
    knight(
        &mut a,
        p,
        crate::dented_motion::yaw_lerp(old.facing, g.facing, alpha),
        1.,
        g.time,
        g.motion.speed / 4.6,
        &pose,
        None,
        g.flash,
        g.progress.weapon == 1,
        g.action == Action::Heal,
    );
    sword_trail(&mut a, g, p);
    if g.action == Action::Heal {
        a.ring(p + Vec3::Y * 0.15, 0.8, 0.04, "#eac579", 0.8, false);
    }
    for (i, e) in g.enemies.iter().enumerate() {
        if e.hp <= 0. && e.death_time > 2.5 {
            continue;
        }
        a.group = 10001 + i as u32;
        let olde = old.enemies.iter().find(|o| o.id == e.id).unwrap_or(e);
        let p = olde.pos.lerp(e.pos, alpha);
        let phase = 1. - e.timer / e.duration.max(0.01);
        let pose = crate::dented_motion::enemy(e, g.time + i as f32);
        let sink = if e.hp <= 0. { ((e.death_time - 1.6).max(0.) * 0.8).min(0.7) } else { 0. };
        knight(
            &mut a,
            p - Vec3::Y * sink,
            crate::dented_motion::yaw_lerp(olde.facing, e.facing, alpha),
            if e.boss() { 2.2 } else { 1.05 },
            g.time + i as f32,
            e.motion.speed / 3.,
            &pose,
            Some(e.kind),
            e.flash,
            e.boss(),
            false,
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
    impacts(&mut a, g);
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
