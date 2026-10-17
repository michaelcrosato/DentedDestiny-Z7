//! Versioned checkpoint saves. The demo never writes or mutates the saved pilgrimage.
use pav_core::{
    Sim,
    games::dented::{Dented, Options, Progress},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Save {
    progress: Progress,
    options: Options,
}
#[cfg(not(target_arch = "wasm32"))]
fn read() -> Option<String> {
    std::fs::read_to_string(crate::boot::exe_dir().join("dented-destiny-save.json")).ok()
}
#[cfg(target_arch = "wasm32")]
fn read() -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item("dented-destiny-z7-v1").ok()?
}
pub fn restore(sim: &mut Sim) {
    let Some(save) = read().and_then(|s| serde_json::from_str::<Save>(&s).ok()) else {
        return;
    };
    let Some(progress) = save.progress.validate() else {
        return;
    };
    let Some(mut play) = sim.state.play.take() else {
        return;
    };
    if let Some(g) = play.get_mut::<Dented>() {
        g.options = save.options;
        if !g.lab {
            g.restore_progress(progress, sim);
        }
    }
    sim.state.play = Some(play);
}
pub fn save(g: &Dented) {
    if g.demo || g.lab {
        return;
    }
    let Ok(text) = serde_json::to_string(&Save { progress: g.progress.clone(), options: g.options.clone() }) else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = crate::boot::exe_dir().join("dented-destiny-save.json");
        let tmp = path.with_extension("tmp");
        match std::fs::write(&tmp, text).and_then(|_| std::fs::rename(tmp, path)) {
            Ok(()) => {}
            Err(e) => log::warn!("Checkpoint save failed: {e}"),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
            if let Err(e) = storage.set_item("dented-destiny-z7-v1", &text) {
                log::warn!("Checkpoint save failed: {e:?}");
            }
        }
    }
}

/// Read-only observability for automated browser checks and performance captures.
#[cfg(target_arch = "wasm32")]
pub fn publish_diagnostics(g: &Dented, fps: f32, frame_ms: f32, adapter: &str) {
    use wasm_bindgen::JsValue;
    let value = serde_json::json!({"mode":g.mode,"demo":g.demo,"menu":g.menu,"hp":g.hp,
        "stamina":g.stamina,"player":g.player_pos,"time":g.time,"idle":g.idle,"flasks":g.flasks,"boss_active":g.boss_active,"options":g.options,"area":g.area,
        "action":g.action,"action_time":g.action_time,"action_total":g.action_total,"combo":g.combo,"hitstop":g.hitstop,
        "impacts":g.impacts,"progress":g.progress,"metrics":g.metrics,"fps":fps,"frame_ms":frame_ms,"adapter":adapter});
    if let (Some(window), Ok(js)) = (web_sys::window(), js_sys::JSON::parse(&value.to_string())) {
        let _ = js_sys::Reflect::set(&window, &JsValue::from_str("__DENTED_DIAGNOSTICS__"), &js);
    }
}
