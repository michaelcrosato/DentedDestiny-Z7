# Combat and animation revision

User direction: make the main character animate beautifully, especially combat; make combat feel forceful and responsive; use cel shading inspired by Wind Waker / Jet Set Radio. Implemented for v0.2.0.

## Findings

The shipped custom renderer bypassed the 437-clip embedded animation library. It used sine-wave legs, single-segment arms, a static torso, and independently rotating weapons. Damage fired at a fixed percentage of every move, enemies vanished when killed, and hit feedback had no contact pause or body reaction on most strikes. The camera and low-contrast palette further reduced pose readability.

## Implementation

- Shared clip/contact/recovery contract in `pav_core/src/games/dented_motion.rs`.
- Three distinct sword-chain poses from Quaternius, faster passage through contact, recovery chaining, buffered evasions, and collision-resolved driving steps.
- Snapshot-owned locomotion clocks, hit reactions, contact pauses, and impact events. Input during a pause is retained.
- Library-sampled whole-body player/enemy poses in `pav_view/src/dented_motion.rs`: idle, locomotion, strafe/backpedal, attacks, roll, guard/parry, consume, jump/land, interaction, equipment and death.
- Articulated armor, attached weapons/shields, continuous cloth, swept blade ribbons and contact bursts.
- Actual cel lighting with outlines, cleaner character surfaces, a closer gameplay camera and immediate camera impulses.
- Impact sounds layer a sharp transient, low body and short metallic partials. The captured combat audio peaks at 0.423 without clipping.

Quaternius and Mesh2Motion source clips are the existing embedded CC0 sets; their original credits remain in `anim/quaternius.json` and `anim/mesh2motion.json`. No new binary art assets.

## Completion evidence

- Inspected action sheets for both weapons and actual combat captures. Fixed broad trails, cloth z-fighting, buried contact bursts, recovery pose popping, and the off-hand grip. The reproducible `dented_review` example captures the same renderer and simulation.
- Ten fresh seeded campaigns won 10/10 (137.35–161.15 seconds). Nineteen Dented simulation tests pass, covering the original campaign/save/replay behavior plus hitstop, buffered chaining, and recovery cancellation. Three pose tests cover the embedded clips, whole-body motion, recovery continuity, and greatblade grip.
- Two complete hardware-browser playthroughs passed. The final-build smoke used the keyboard to switch to the greatblade, chain all three lights, perform a heavy strike and roll; its 15 FPS samples averaged 59.9909.
- Inspected phone, tablet and ultrawide Chrome captures and WebKit compatibility captures; no overflow or JavaScript errors.
- Full test suites, clippy, formatting, native and optimized WebAssembly builds passed. Browser evidence, FPS samples and artifact hashes are recorded in [combat-revision.json](evidence/combat-revision.json).

## Verification notes (2026-10-16)

The complete core/view/audio run passed 332 tests. That run exposed a pre-existing golden-list omission: the two abbey materials shipped in v0.1.0 were absent from the material fixture. Both were added; all 18 existing material hashes matched unchanged. A subsequent focused motion run passed three tests, including the additional two-handed-grip regression. Both app input tests also passed. Clippy passed for core, view, audio, app and tools with all targets and warnings denied.

The hardware browser completed the campaign and both boss phases at 1920×1080: 161 one-second samples, 60.0065 FPS average and 59.4295 minimum sampled FPS. Standard Gamepad API input, mixed pointer/keyboard menus, demo cancellation and campaign-save isolation passed. The final supporting-hand correction was then checked separately as pose samples and a browser smoke run; it does not change simulation rules. The final smoke averaged 59.9909 FPS, reported no errors, and confirmed three keyboard light strikes, a heavy and a dodge with the greatblade equipped.

Chrome on the RTX 3060 Ti rendered the final scene at 375×667 with touch emulation, 1194×834 tablet and 2560×1080 ultrawide without overflow. The mobile views entered attract mode and omitted combat controls. Thirteen WebKit viewport checks had no layout or JavaScript errors; this WSL WebKit does not expose WebGPU, so those inspect the compatibility screen, not gameplay. Real iOS Safari and physical-controller hardware were not available.

Automated tests and captures used local tools, with no extra external API calls. Exact session/model billing is not exposed. Captures are review evidence, not proof of every player's subjective combat preference.

The Linux package booted outside the source checkout in 689 ms under Xvfb/software Vulkan, then was stopped by the intentional eight-second timeout. There is no physical ALSA device here; the game continued silently. Synthesized audio was verified separately in the combat capture. The browser remains the measured hardware target.
