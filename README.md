# Dented Destiny · The Bellwether Abbey

**The crown is broken. The oath remains.**

An original Souls-inspired action RPG vertical slice, built on the **Shardfall / Pavilion** engine in Rust and WebGPU. One authored ruined abbey connects deliberate melee combat, exploration, a hidden weapon, two sanctuaries, recoverable death currency, permanent upgrades, a looping shortcut, and Vesper, a two-phase bell guardian.

The default scene is the game. The engine's original workshop and tools remain available for expansion. This repository preserves its upstream history; original engine documentation is in [the upstream README](docs/game/UPSTREAM_README.md).

## Play and build

Rust 1.98.1 is pinned. The repository includes Cargo.lock. No API keys or hosted services are needed.

```bash
# Native Linux dependencies, using local package extraction when no admin access is available:
scripts/setup-dented-local.sh
scripts/build-dented.sh
./target/debug/shardfall                 # title screen
./target/debug/shardfall --scene dented/play
./target/debug/shardfall --scene dented/lab
```

The native binary retains the upstream `shardfall` executable name for tool compatibility; its default game, title, settings and save are Dented Destiny. `dented/lab` opens practice with the gates, relic and upgrade currency available. `--scene world` opens the original engine pavilion.

```bash
# Browser output; install the wasm-bindgen-cli version pinned in Cargo.lock first.
rustup target add wasm32-unknown-unknown
scripts/build-web.sh --single-file --scene dented \
  --title 'Dented Destiny — The Bellwether Abbey' --out out/DentedDestiny-Z7.html
```

Open the self-contained HTML in a WebGPU-capable browser with hardware acceleration. It requires no web server. `scripts/build-web.sh` without flags produces ordinary static files in target/web. `scripts/build-windows.sh` is the upstream Windows cross-build route and requires the MinGW toolchain.

## Controls

A standard-mapped PS5 DualSense is the primary layout. Keyboard/mouse actions follow the same combat and focus-navigation model.

| Action | PS5 controller | Keyboard / mouse |
| --- | --- | --- |
| Move | Left stick | WASD |
| Camera | Right stick | Middle-button drag |
| Light / heavy strike | R1 / R2 | J / K or left / right click |
| Evade | Circle | Space |
| Guard / timed parry | L1 / L2 | Shift / F |
| Lock on | R3 | R |
| Amber flask | Square | X |
| Interact / confirm | Cross | E / Enter |
| Switch weapon | Triangle | Q |
| Jump | L3 | V |
| Pause | Options | Escape |
| Equipment and memories | Create | Tab |
| Menu navigation | D-pad / left stick | Arrows |
| Return | Circle | Backspace |

Light and heavy attacks consume stamina and commit to an animation. Evades have a short invulnerable window. Guarding absorbs most frontal damage at a stamina cost; a well-timed parry staggers a melee enemy. A heavy blow breaks poise. Read windups and punish recovery. The second phase of Vesper adds a spread of spectral bolts.

Rest refills health and three flasks and revives ordinary enemies. Echoes buy vitality and weapon improvements. Death leaves your carried echoes at the fall; another death replaces that cache. Opened gates, the relic, blessings, and the defeated guardian persist. Ring the final bell to complete the pilgrimage, then explore the quiet abbey or begin again.

## Attract mode and accessibility

After ten seconds without input on the title or its submenus, the same AI used by the headless tools plays the actual level. It repeats after victory or death. Any physical input immediately returns to the title. Demo progress never replaces the player's saved pilgrimage. Phones and tablets present the autonomous scene without touch combat controls.

Settings offer reduced incoming damage, impact shake, sound, contrast, and an elevated tactical camera. Menus use large text and snapping focus. FPS and frame time remain visible on desktop.

Native checkpoint saves are versioned JSON beside the executable (`dented-destiny-save.json`). Browser saves use localStorage (`dented-destiny-z7-v1`). New oath resets progression. Unknown save versions are rejected, and bounded values are validated.

## Develop and verify

```bash
scripts/test-dented.sh
./target/debug/pav play scene=dented/play bot=true seeds=10 ticks=18000
./target/debug/pav capture scene=dented/play ticks=60 width=1280 height=720 out=out/abbey.png
./target/debug/pav room_check path=crates/pav_core/src/games/dented.toml
./target/debug/pav bench scene=dented/play ticks=1200
```

Start with [the game plan](docs/game/PLAN.md), [engine feedback](docs/game/ENGINE_FEEDBACK.md), and the engine's [MAKE_A_GAME](docs/MAKE_A_GAME.md) guide. The original brief is preserved in [BRIEF.txt](docs/game/BRIEF.txt). The 50 generated names, random selection method and suffix are recorded in [naming.json](docs/game/naming.json).

- `crates/pav_core/src/games/dented.rs`: simulation, combat, AI, progression, menus and attract lifecycle.
- `crates/pav_core/src/games/dented.toml`: level geometry, encounter data, balance and view configuration.
- `crates/pav_view/src/dented.rs`: original procedural architecture, armor, weapons, animation and effects.
- `crates/pav_app/src/dented_ui.rs`: controller-first presentation.
- `crates/pav_app/src/dented_save.rs`: native/browser persistence.
- `crates/pav_core/tests/dented.rs`: deterministic and gameplay verification.

## Source information

This provenance is separate from the engine evaluation linked above.

| Field | Reference |
| --- | --- |
| Initial date | 2026-10-16 |
| SOURCE | Elden Ring (2022), inspiration for deliberate combat, discovery and death/recovery. No original game's assets, story or audio are used. |
| GAME ENGINE | [Shardfall / Pavilion](https://github.com/michaelcrosato/shardfall), workspace 0.1.0, upstream commit `dd5584bc0dbd0d3fb27de7059f97430132256b31`; Rust 1.98.1, wgpu 30, Rapier 0.36, egui 0.36. |
| Environment | Codex tools using Bash in WSL2, Ubuntu 26.04 Linux, Windows host; RTX 3060 Ti detected (8 GiB). Browser target is WebAssembly + WebGPU. |
| Model and effort | OpenAI Codex, GPT-6 family as identified by session configuration. Exact model variant and reasoning effort are not exposed and are not asserted. |
| Repository | [michaelcrosato/DentedDestiny-Z7](https://github.com/michaelcrosato/DentedDestiny-Z7) |

All new scenery, characters and sound use procedural code. The included DejaVu Serif font retains its [license](assets/fonts/LICENSE-DejaVu.txt). Upstream source/data and their credits are preserved.
