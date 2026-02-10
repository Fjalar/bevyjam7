use std::f32::consts::PI;

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GunAssets>();

    app.add_systems(
        Update,
        (
            update_gun.in_set(PausableSystems),
            shoot_gun
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
        ),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GunAssets {
    #[dependency]
    image: Handle<Image>,
}

impl FromWorld for GunAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            image: assets.load_with_settings(
                "images/gun.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

pub fn gun_bundle(gun_assets: &GunAssets) -> impl Bundle {
    (
        Sprite::from_image(gun_assets.image.clone()),
        Gun::default(),
        Transform::from_xyz(32.0, 0.0, 0.0),
    )
}

#[derive(Component, Default)]
struct Gun {
    angle: f32,
}

fn update_gun(
    mut gun: Single<(&mut Gun, &mut Transform, &mut Sprite)>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if let Some(position) = window.cursor_position() {
        let mouse_vector = position - Vec2::new(window.width() / 2.0, window.height() / 2.0);
        let angle = (-mouse_vector.y / mouse_vector.x).atan();
        gun.0.angle = angle;
        gun.1.translation = Vec3::X * 32.0;
        gun.1.rotation = Quat::default();
        gun.1
            .rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle));
        if mouse_vector.x.is_sign_negative() {
            gun.1.rotate_around(Vec3::ZERO, Quat::from_rotation_z(PI));
            gun.2.flip_y = true;
        } else {
            gun.2.flip_y = false;
        }
    }
}

fn shoot_gun(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    gun: Single<(&GlobalTransform, &Gun)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        commands.spawn((
            Sprite::from_color(Color::WHITE, Vec2::splat(32.0)),
            Transform::from_translation(gun.0.translation()).with_rotation(gun.0.rotation()),
        ));
    }
}
