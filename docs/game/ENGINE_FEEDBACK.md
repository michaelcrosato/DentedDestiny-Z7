# Pavilion engine feedback — Dented Destiny

Upstream: https://github.com/michaelcrosato/shardfall at dd5584bc0dbd0d3fb27de7059f97430132256b31 (workspace 0.1.0).

The game uses `Game`, standalone room TOML, Rapier character movement and fixed colliders, NavGrid, seeded RNG, snapshots, replays, SimEvents, procedural mesh instancing, lights, shadows, fog, bloom, synthesized audio, and the CLI. It does not use Pavilion Lite.

The main API gap was rich per-game presentation. `Game::hud` exposes only text lines. A typed immutable snapshot was added to `HudFrame`; `pav_view/src/dented.rs` emits original geometry, and `pav_app/src/dented_ui.rs` renders controller-focused egui menus. The general engine would benefit most from a game registration API that supplies presentation and input adapters without editing app.rs.

Engine source had to be read to integrate the new game into `app.rs`'s render/input loop, to find Character's slow/haste/face/dash fields, to understand NavGrid's fixed-object bounds, and to use MeshInstance directly. These are integration surfaces not covered by MAKE_A_GAME's short examples. The controller layout and startup/saving logic were coupled to Shardfall; a reusable host adapter would make a second game much simpler.

Dented Destiny implements reusable deterministic combat/AI states, buffered actions, checkpoint progression, save validation, controller focus navigation, and an attract mode that uses the same bot/input path as headless verification. These belong in game-level examples rather than renderer internals. Encounter balance and positions are authored in dented.toml.

Known inherited constraints: WebGPU availability varies by browser and driver; the native Linux path in this WSL environment may use software Vulkan. GPU performance must be measured on the adapter actually used. Save failures are logged, not silently claimed successful.

A navigation defect surfaced in the full-route bot: fixed entities used a square footprint whose radius was the larger X/Z half extent. A wide, thin gate therefore blocked several metres of its own approach, preventing the bot from reaching an interactable door even though physics allowed it. `pav_core/src/nav.rs` now grows each axis of the rotated AABB independently. This is an engine correction, not a game-specific route workaround.

The engine's low-camera `front_cut` default removes floor in front of the focus. This is useful in a side-view room but made the title camera show an apparently floating player. The abbey explicitly disables `view.cutaway.front_cut` in its level data and retains the narrower occlusion fade. A third-person preset should disable front cutting by default.

The game also adds `InputFrame.menu` (`MenuSelection`: screen, epoch, choice) and consumes it once in SimHost. Mouse selection now follows the same recorded simulation path as controller confirmation; stale UI contexts are rejected. This avoids unrecorded host-side menu mutations. `play` reports menu context/choices for headless inspection. Input-only replays begin from a fresh scene; use a captured initial snapshot when diagnosing a resumed campaign.
