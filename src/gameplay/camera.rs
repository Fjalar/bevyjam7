use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, gameplay::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        camera_follow_player
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

pub fn camera_follow_player(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    camera.translation = player.translation
}
