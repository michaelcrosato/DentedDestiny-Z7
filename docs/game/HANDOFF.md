# Expanding Dented Destiny

Read README.md and PLAN.md first. The engine entry point is `dented`; `/play` skips the title and `/lab` provides the relic, opened routes and upgrade currency. The public project is `michaelcrosato/DentedDestiny-Z7`, not the upstream engine repository.

## Content and state contracts

All mutable gameplay belongs to `Dented` in `SimState.play`, with `Clone` and serde. Presentation receives a clone through `HudFrame.dented`. Never put health, timers, progression or AI decisions in the renderer or app. `Options` are included in snapshots; save serialization is versioned and isolated at the host boundary.

`dented.toml` is a standard Pavilion room plus a `[game]` table. Add `[[game.encounters]]` entries for existing archetypes; change their positions, HP, reward and sight. Room layers build collision and walkable floor. Props drawn only by the view are decorative unless their fixed colliders are created by build(). Keep render geometry and colliders aligned.

The reusable enemy flow is Idle / Chase / Windup / Recover / Stagger / Dead. A strike has one damage activation per committed action. Player action buffering, stamina gates, guard direction, parry window, roll invulnerability and hit immunity are independent state variables. Extend this state machine when adding an archetype. Teach the bot each new mechanic; do not bypass damage, currency or doors for demo play.

Progress includes checkpoint, upgrades, currency, equipment, unlocks, boss defeat and the death cache. Rest respawns ordinary enemies. Save changes at progression milestones, while demo saves are suppressed. Adding a save field needs a safe default; breaking format changes need a new version and migration.

Menu focus is simulation state. Device adapters translate into input bits; no desktop-only gameplay actions. Menus pause game decisions. `ACTIVITY` represents physical input even if it has no action binding, and cancels the attract loop before the same event can start play.

Attract uses `Game::bot` against real combat and movement, preserving a copy of personal progression. Death and victory linger, then the run resets. Input returns to the title. The engine already bounds replay recording and rewind history.

## Tool-first edit loop

1. Edit encounter/layout TOML and inspect with `pav room_check`.
2. Run `scripts/test-dented.sh` before browser or native window testing.
3. Use `pav play`, `input`, `snapshot_save`, `snapshot_load`, `rewind`, `record_save`, `replay`, `capture` and `filmstrip` for narrow investigations.
4. Inspect captures yourself. Geometry, camera framing, transitions, weapon motion and readable telegraphs require visual inspection.
5. Build browser output and check keyboard, gamepad, title idle timing, input cancellation, saves and rendering errors. Use an isolated browser profile and uncommon port if a server is necessary.
6. Record adapter identity when quoting FPS. Linux Vulkan software rendering under WSL is not evidence about RTX 3060 Ti hardware performance.

## Procedural art

`pav_view/src/dented.rs` uses Pavilion MeshInstance primitives, deterministic geometric variation, skeletal-like joint transforms, cape segments and instancing. Colors in authored code are sRGB converted with `Color::hex`. Effects and audio originate from SimEvents. New binary art is unnecessary; the included licensed font is the only new binary source asset.

## Engine integration

The app changes are input dispatch, custom UI routing, game camera, versioned save/load, title/window naming and default scene. The renderer skips the stock puppet and invisible game collider geometry, then emits the game's presentation. Existing Pavilion and Shardfall scenes remain accessible through CLI arguments. Feedback for the upstream engine is separate in ENGINE_FEEDBACK.md.
