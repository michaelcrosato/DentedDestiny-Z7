# Verification record

This file distinguishes implemented behavior from verified delivery. The goal remains active until all required checks and visual review are complete.

## Current evidence

- Initial engine checkout: `dd5584bc0dbd0d3fb27de7059f97430132256b31`; source uses its Game hook, physics, navigation, wgpu renderer, event audio and deterministic tools.
- Public repository created at https://github.com/michaelcrosato/DentedDestiny-Z7. Source push and delivery pending.
- Six gameplay tests were added. Save validation, insufficient stamina, pause/focus, 10-second attract entry/input exit and snapshot/input replay passed on the first relevant runs.
- Full-route bot testing found and prompted repairs to menu-event consumption, spear-enemy dodge behavior and the engine's oversized gate navigation footprint. The complete route is still being verified.
- Native compilation dependencies were installed in `.cache/native` using apt package extraction because sudo is unavailable. No system packages were modified.
- Rust's wasm32 target is installed; the pinned wasm-bindgen 0.2.129 executable archive was downloaded from its official GitHub release and its SHA-256 verified.

## Remaining completion gates

- Full pilgrimage won across seeds; real death/recovery, guard/parry, flask, checkpoint, upgrade, shortcut and boss-phase tests.
- Autonomous repeated attract cycles, complete input interruption and save isolation.
- Native and browser builds; rendered review of title, combat, exploration, boss, rest and ending.
- Controller navigation and all bindings, keyboard/mouse, persistence/reload, browser runtime errors.
- Small phone/tablet/wide viewport and WebKit checks; no mobile combat overlay.
- FPS/frame time recorded with adapter identity; target RTX 3060 Ti floor verified.
- Required final checks, CI, source push, browser deliverable and reproducible handoff.
