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
            reload_gun
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
    let mut timer = Timer::from_seconds(1.0 / 3.0, TimerMode::Once);
    timer.finish();
    (
        Sprite::from_image(gun_assets.gun.clone()),
        Gun {
            ammo: 7,
            max_ammo: 7,
            ..Default::default()
        },
        Transform::from_xyz(32.0, 0.0, 0.0),
    )
}

#[derive(Component, Default)]
struct Gun {
    state: GunState,
    ammo: u32,
    max_ammo: u32,
    angle: f32,
}

#[derive(Default, PartialEq, Eq)]
enum GunState {
    #[default]
    Ready,
    Shooting(Timer),
    Reloading(Timer),
}

fn update_gun(
    mut gun: Single<(&mut Gun, &mut Transform, &mut Sprite)>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let mut extra_rotation = 0.0;
    match &mut gun.0.state {
        GunState::Shooting(timer) => {
            timer.tick(time.delta());
            extra_rotation = timer.remaining_secs();
            if timer.is_finished() {
                gun.0.state = GunState::Ready;
            }
        }
        GunState::Reloading(timer) => {
            timer.tick(time.delta());
            extra_rotation = timer.remaining_secs() * PI;
            if timer.is_finished() {
                gun.0.state = GunState::Ready;
                gun.0.ammo = gun.0.max_ammo;
            }
        }
        GunState::Ready => (),
    }

    if let Some(position) = window.cursor_position() {
        let mouse_vector = position - Vec2::new(window.width() / 2.0, window.height() / 2.0);
        let angle = Vec2::new(mouse_vector.x, -mouse_vector.y).to_angle();
        gun.0.angle = angle;
        gun.1.translation = Vec3::X * 32.0;
        gun.1.rotation = Quat::default();
        gun.1
            .rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle));
        if mouse_vector.x.is_sign_positive() {
            gun.2.flip_y = false;
            gun.1.rotate_z(extra_rotation);
        } else {
            gun.2.flip_y = true;
            gun.1.rotate_z(-extra_rotation);
        }
    }
}

fn shoot_gun(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    gun_assets: Res<GunAssets>,
    mut gun: Single<(&GlobalTransform, &mut Gun)>,
) {
    if mouse.pressed(MouseButton::Left) && gun.1.state == GunState::Ready && gun.1.ammo > 0 {
        commands.spawn(bullet_bundle(
            &gun_assets,
            Transform::from_translation(gun.0.translation()).with_rotation(gun.0.rotation()),
            Vec2::from_angle(gun.1.angle) * 320.0,
        ));
        gun.1.state = GunState::Shooting(Timer::from_seconds(0.5, TimerMode::Once));
        gun.1.ammo -= 1;
    }
}

fn reload_gun(key: Res<ButtonInput<KeyCode>>, mut gun: Single<&mut Gun>) {
    if key.pressed(KeyCode::KeyR) && gun.state == GunState::Ready && gun.ammo < gun.max_ammo {
        gun.state = GunState::Reloading(Timer::from_seconds(2.0, TimerMode::Once));
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
