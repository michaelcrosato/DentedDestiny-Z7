# Dented Destiny — Bellwether Abbey

Original single-level action RPG inspired by Elden Ring (2022). No source-game assets, lore, names or audio are reused.

## Required deliverables
- Public GitHub repository `michaelcrosato/DentedDestiny-Z7`; 50 title candidates and random draw recorded in naming.json.
- Shardfall/Pavilion engine based on dd5584bc0dbd0d3fb27de7059f97430132256b31, with original deterministic game module.
- One complete authored abbey: sanctuary, courtyard, reliquary, chapel, shortcut, approach, boss arena, ending.
- Deliberate light/heavy melee, stamina, dodge invulnerability, guard, timed parry, lock-on, healing, weapon switch, enemy telegraphs, ranged threat, two-phase boss.
- Death/recovery, checkpoint rest and enemy respawn, permanent unlocks, currency upgrades, equipment, narrative discoveries, victory and replay.
- Controller-first title/pause/rest/settings/help/journal, large readable HUD, accessible settings, keyboard/mouse equivalents.
- Title/menu inactivity after exactly 10 seconds launches real AI play; continuous cycles; any input cancels to title; no demo writes to personal progress.
- Mobile: autonomous demo, no gameplay touch buttons.
- Procedural original geometry, lighting, animation, synthesized sound. Frame timer visible; verify performance on available RTX 3060 Ti path where possible.
- Save/load, deterministic snapshots/replay, bot, data-driven encounters, development proving-ground scene, engine feedback and handoff documentation.
- Logic checks first; then headless/offscreen and browser checks, no server if single HTML suffices; uncommon port if required.
- Source information in README, HTML, final chat, separate from engine evaluation.

## Implementation
Use the existing Game hook for rules, Rapier characters and static collision, NavGrid paths, seeded state, snapshots/replay, event-driven audio/effects and wgpu rendering. Add a typed game presentation snapshot to HudFrame. Presentation emits custom armored actors and architecture through Pavilion meshes; egui supplies controller-focused menus. Preserve the engine workshop scenes for expansion. Authored TOML contains the level and encounter balance. New content uses the same rules and assets.

## Verification record
Pending implementation. Completion is requirement-by-requirement, with command outputs and visual evidence recorded in VERIFICATION.md.
