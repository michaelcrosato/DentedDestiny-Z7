# Verification — Dented Destiny Z7

The v0.2.0 combat/animation update is verified in [COMBAT-REVISION.md](COMBAT-REVISION.md), with [machine-readable evidence](evidence/combat-revision.json). The record below describes the original v0.1.0 delivery.


Validation uses actual simulation, rendered output and repository state. The source requirement list is preserved in PLAN.md and the original brief in BRIEF.txt.

## Verified implementation

| Requirement | Evidence |
| --- | --- |
| Fifty titles, uniform random selection, two-character ID | `naming.json`: 50 distinct candidates; `DentedDestiny-Z7` selected with Python secrets.choice. |
| Public repository and source engine foundation | `michaelcrosato/DentedDestiny-Z7`, public; full Shardfall history retained. Foundation commit `dd5584bc0dbd0d3fb27de7059f97430132256b31`. |
| Complete single level | Real bot and browser runs traverse relic, chime, oath gate, shortcut, sanctuary, two boss phases and final bell; `Outcome::Won` / rendered Victory reached. |
| Combat and progression | `tests/dented.rs` exercises stamina denial, distinct guard/parry/evade effects, one activation per heavy, delayed healing, death cache, rest/respawn and upgrade spending. |
| Autonomous attract loop | At tick 599 it has not started; tick 600 starts it. Multiple actual runs repeat. Physical-input representations cancel it to Title; personal progression, run time and combat counters are restored. |
| Controller-first UX | Standard PS5 mapping exercised through the browser Gamepad API: snapped focus, Cross, Circle, R1/R2, L2, Options, journal/settings; keyboard Escape and Backspace regression tested. |
| Save/load and practice isolation | Practice starts with its own resources and never writes the campaign save; browser resting/upgrading verified this. Browser fixture restored checkpoint, echoes, vitality, edge, weapon, unlocks, death cache and contrast after a page reload. Version validation tested. Demo does not change localStorage. |
| Deterministic foundation | Snapshot serialization and identical-input replay produce equal state hashes, including pointer menu selections. Stale menu contexts are rejected. Rewinding to tick 240 and replaying to tick 330 reproduced the same hashes; input replay reported `matches: true`. Seeded bot wins. All changing rules state resides in Clone/serde SimState. |
| Extensibility and handoff | Encounter stats, placement, landmarks, layout, lighting and controls are TOML data. Shared enemy/action state machines, typed presentation snapshot, `dented/lab`, engine pavilion and HANDOFF.md are included. |
| Original procedural visuals and sound | Instanced architecture, armor, cloth, weapons, vegetation and particles; authored procedural stone/cloth materials; Pavilion event synthesis. Only the licensed font is a new binary source asset. |
| Mobile and wide layouts | Actual WebGPU captures with emulated viewport/touch at 375×667, 1194×834 and 2560×1080, on the desktop GPU (not physical phone/tablet hardware). Coarse-pointer devices enter attract and have no gameplay overlay. No overflow; all captured PNGs inspected. |
| WebKit check | iPhone SE and iPad landscape compatibility screens inspected; no overflow, page errors or failed requests. This WSL WebKit lacks WebGPU, so this does **not** prove Safari gameplay support. |
| Source information | README.md and web/index.html contain date, source, engine pin, environment and confirmed model-family information, separately from ENGINE_FEEDBACK.md. |

## Commands and results

- `cargo test --locked -p pav_core --test dented --test games`: 17 game tests and 4 hook/starter tests passed, including complete runs on three seeds and repeated attract runs.
- `cargo test --locked -p pav_core --lib nav::`: navigation test passed. The game suite additionally checks that a thin gate does not obstruct its interaction approach.
- `cargo test --locked -p pav_app input::pad_script_tests`: both input tests passed, including every authored room binding resolving correctly.
- `cargo clippy --locked -p pav_core -p pav_view -p pav_app --all-targets -- -D warnings`: passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Native Linux build and optimized wasm32 build: passed. Room validation: 41×57 layout, valid.
- Packaged native Xvfb smoke, launched outside the source tree: renderer, UI, simulation and first frame initialized in 0.92 seconds; terminated by its 10-second timeout as intended. WSL used llvmpipe; no ALSA device was available. An XDG portal timeout was logged before successful boot. These native limits are separate from the NVIDIA hardware browser run.
- Headless Windows Chrome loaded the self-contained HTML directly from disk. No content server or public tunnel was used.

## Hardware and visual review

The machine reports an NVIDIA GeForce RTX 3060 Ti (8 GiB). The browser reports NVIDIA / Ampere. The final complete 1920×1080 run sampled 157 one-second windows: **60.03 FPS average, 59.28 minimum sampled average**. Both boss phases and Victory rendered. Page errors, console errors and browser log errors were absent. Shader/startup warmup is outside gameplay samples. The raw measurements and capture hashes are in [browser-run.json](evidence/browser-run.json).

Title, sanctuary, pause, journal, settings, combat, both boss phases, ending, phone, tablet and ultrawide PNGs were inspected. Review led to weathered materials, reduced foreground walls, disabling the side-view front cut for this third-person scene, a wider boss camera, removing hints that overlapped the boss bar, and clearing underlying HUD/instructions from the ending.

The browser distribution uses the optimized `dist` profile. The Linux companion is a development-profile build with debug symbols stripped. Physical DualSense hardware and real iOS Safari were not available. Controller coverage is a standard-mapping browser emulation, not a hardware certification. The Linux build was checked on Ubuntu 26.04/WSL; older Linux compatibility and a Windows native executable are not claimed. Chrome/Edge with WebGPU is the portable playable target.

## Release pass

- Game code: `ba4f23e40cb5440167cd35ce288ab1b59d936ef0`. Subsequent delivery edits are documentation/evidence only.
- [GitHub Actions](https://github.com/michaelcrosato/DentedDestiny-Z7/actions/runs/38976889526): **success**. Formatting, 17 game tests, 4 engine-hook tests, navigation and browser compilation passed.
- [Ten fresh seeded campaigns](evidence/ten-seeds.json): **10/10 wins**, 149.65–167.08 simulation seconds. The browser run additionally exercised a campaign after real menu/controller interactions.
- [Rewind and replay](evidence/rewind-replay.json): exact hashes at both checkpoints, final replay matched at tick 330.
- [Browser save restoration](evidence/browser-save.json), [practice isolation](evidence/practice-save.json), [viewport results](evidence/viewports.json), [WebKit compatibility](evidence/webkit.json) and [console errors](evidence/console.json) are inspectable records.
- [Synthesized audio](evidence/audio.json): 3.5 seconds of non-silent PCM, peak 0.232, RMS 0.0134. WSL's missing physical audio output is not mistaken for a synthesis failure.
- [Native package smoke](evidence/native-smoke.json): standalone executable booted with embedded data, outside the checkout.
- [Artifact sizes and SHA-256 hashes](evidence/artifacts.json) identify the tested HTML and Linux archive. The archive contains only the executable, instructions and font license; no test saves or local settings are included.
- Distribution: [v0.1.0](https://github.com/michaelcrosato/DentedDestiny-Z7/releases/tag/v0.1.0). The single HTML opens from disk and needs no content server.

The completion audit covers every requirement in the original brief: naming/repository, engine reuse, the complete level and ending, extensible systems, real repeated AI play and cancellation, controller-first controls, mobile presentation, procedural visuals/animation/sound, frame timing and target-hardware performance, source provenance, builds and handoff. No requested gameplay or infrastructure work remains.
