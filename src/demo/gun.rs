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
    )
    .add_systems(FixedUpdate, update_bullet);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GunAssets {
    #[dependency]
    gun: Handle<Image>,
    #[dependency]
    bullet: Handle<Image>,
}

impl FromWorld for GunAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            gun: assets.load_with_settings(
                "images/gun.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            bullet: assets.load_with_settings(
                "images/bullet.png",
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
        Sprite::from_image(gun_assets.gun.clone()),
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
        let angle = Vec2::new(mouse_vector.x, -mouse_vector.y).to_angle();
        gun.0.angle = angle;
        gun.1.translation = Vec3::X * 32.0;
        gun.1.rotation = Quat::default();
        gun.1
            .rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle));
        gun.2.flip_y = mouse_vector.x.is_sign_negative();
    }
}

fn shoot_gun(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    gun_assets: Res<GunAssets>,
    gun: Single<(&GlobalTransform, &Gun)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        commands.spawn(bullet_bundle(
            &gun_assets,
            Transform::from_translation(gun.0.translation()).with_rotation(gun.0.rotation()),
            Vec2::from_angle(gun.1.angle) * 320.0,
        ));
    }
}

#[derive(Component)]
struct Bullet {
    velocity: Vec2,
}

fn bullet_bundle(gun_assets: &GunAssets, transform: Transform, velocity: Vec2) -> impl Bundle {
    (
        Sprite::from_image(gun_assets.bullet.clone()),
        Bullet { velocity },
        transform,
    )
}

fn update_bullet(bullets: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in bullets {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_secs()
    }
}
