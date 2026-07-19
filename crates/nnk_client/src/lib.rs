//! Bevy client — presentation only (SOLID: depends on domain, not server).

mod map;
mod ui;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NNK Stalker".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                canvas: Some("#nnk-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        })
        .set(bevy::asset::AssetPlugin {
            file_path: asset_file_path(),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins((map::MapPlugin, ui::UiPlugin))
        .run();
}

fn asset_file_path() -> String {
    if cfg!(target_arch = "wasm32") {
        "/assets".to_string()
    } else {
        "assets".to_string()
    }
}
