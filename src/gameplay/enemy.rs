use std::time::Duration;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, gameplay::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();

    app.add_systems(
        Update,
        (
            update_enemy_animation_timer.in_set(AppSystems::TickTimers),
            (update_enemy_atlas).chain().in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        spawn_enemy_on_spacebar
            .in_set(PausableSystems)
            .in_set(AppSystems::RecordInput),
    );

    app.add_systems(Update, follow_player);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Enemy {
    health: f32,
    speed: f32,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct EnemyAssets {
    #[dependency]
    image: Handle<Image>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            health: 20.0,
            speed: 30.0,
        }
    }
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            image: assets.load_with_settings(
                "images/tetra.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

fn spawn_enemy_on_spacebar(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    enemy_assets: If<Res<EnemyAssets>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if input.just_pressed(KeyCode::Space) {
        commands.spawn(enemy(&enemy_assets, Vec2::ZERO, &mut texture_atlas_layouts));
    }
}

fn enemy(
    enemy_assets: &EnemyAssets,
    location: Vec2,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 12, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let enemy_animation = EnemyAnimation::new();

    (
        Name::new("Enemy"),
        Enemy::default(),
        Sprite::from_atlas_image(
            enemy_assets.image.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        ),
        Transform::from_scale(Vec2::splat(1.0).extend(1.0)).with_translation(location.extend(0.0)),
        enemy_animation,
    )
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct EnemyAnimation {
    timer: Timer,
    frame: usize,
}
impl EnemyAnimation {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            frame: 0,
        }
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        self.frame = (self.frame + 1) % 12;
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_enemy_atlas(mut query: Query<(&EnemyAnimation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.frame;
        }
    }
}

/// Update the animation timer.
fn update_enemy_animation_timer(time: Res<Time>, mut query: Query<&mut EnemyAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

fn follow_player(
    enemies: Query<(&mut Transform, &Enemy), Without<Player>>,
    player: Single<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut transform, enemy) in enemies {
        let toward_player = (player.translation.xy() - transform.translation.xy()).normalize()
            * enemy.speed
            * time.delta_secs();

        transform.translation += toward_player.extend(0.0);
    }
}
