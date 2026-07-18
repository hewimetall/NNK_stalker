use bevy::prelude::*;
use nnk_domain::{HexCoord, LocationId};

const HEX_SIZE: Vec2 = Vec2::new(28.0, 28.0);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLocation(LocationId::Cordon))
            .add_systems(Startup, setup_map);
    }
}

#[derive(Resource)]
struct CurrentLocation(LocationId);

#[derive(Component)]
struct HexCell {
    hex: HexCoord,
}

fn axial_to_world(hex: HexCoord) -> Vec2 {
    // pointy-top axial → pixel
    let x = HEX_SIZE.x * (3.0_f32.sqrt() * hex.q as f32 + 3.0_f32.sqrt() / 2.0 * hex.r as f32);
    let y = HEX_SIZE.y * (3.0 / 2.0 * hex.r as f32);
    Vec2::new(x, y)
}

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // Soft atmosphere (Recraft-generated)
    commands.spawn((
        Sprite {
            image: asset_server.load("generated/zone_atmosphere.png"),
            custom_size: Some(Vec2::new(1600.0, 900.0)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.35),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    // Sector cluster radius 2 ≈ 19 hexes (term.md.md)
    for q in -2i32..=2 {
        for r in -2i32..=2 {
            let s = -q - r;
            if s < -2 || s > 2 {
                continue;
            }
            let hex = HexCoord::new(q, r);
            let pos = axial_to_world(hex);
            let color = if hex == HexCoord::ZERO {
                Color::srgb(0.55, 0.62, 0.35)
            } else {
                Color::srgb(0.22, 0.28, 0.18)
            };
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(HEX_SIZE.x * 1.7, HEX_SIZE.y * 1.5)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.0),
                HexCell { hex },
            ));
        }
    }

    commands.spawn((
        Sprite {
            image: asset_server.load("generated/stalker_token.png"),
            custom_size: Some(Vec2::splat(36.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}
