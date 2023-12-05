use crate::world::PromisedScene;
use bevy::prelude::*;

use super::{spawn_point::SpawnPoint, ProvinceState};

#[derive(Component)]
struct Affiliation;

pub struct ShootingRangePlugins;

impl Plugin for ShootingRangePlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProvinceState::ShootingRange), load)
            .add_systems(OnExit(ProvinceState::ShootingRange), unload);
    }
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpawnPoint::new(Vec3::new(0., 30., 0.)));

    commands
        .spawn((PointLightBundle {
            point_light: PointLight {
                intensity: 1500.,
                shadows_enabled: true,
                radius: 100.,
                range: 60.,
                ..default()
            },
            transform: Transform::from_xyz(0., 40., 0.),
            ..default()
        },))
        .insert(Affiliation);

    let scene = asset_server.load("test_province.glb#Scene0");

    commands.spawn((
        SceneBundle { scene, ..default() },
        PromisedScene,
        Affiliation,
        Name::new("ShootingRange"),
    ));
}

fn unload(mut commands: Commands, affiliation_query: Query<Entity, With<Affiliation>>) {
    for entity in affiliation_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
