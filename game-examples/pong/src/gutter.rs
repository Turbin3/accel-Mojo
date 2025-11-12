use bevy::prelude::*;

use crate::components::{Collider, Position};

#[derive(Component)]
#[require(Position, Collider)]
pub struct Gutter;

pub const GUTTER_COLOR: Color = Color::srgb(1., 1., 1.);
pub const GUTTER_THICKNESS: f32 = 20.;

pub fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let material = materials.add(GUTTER_COLOR);
    let padding = 20.;

    let vertical_gutter_shape = Rectangle::new(window.resolution.width(), GUTTER_THICKNESS);
    let mesh = meshes.add(vertical_gutter_shape);

    let top_gutter_position = Vec2::new(0., window.resolution.height() / 2. - padding);

    commands.spawn((
        Gutter,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(top_gutter_position),
        Collider(vertical_gutter_shape),
    ));

    let bottom_gutter_position = Vec2::new(0., -window.resolution.height() / 2. + padding);

    commands.spawn((
        Gutter,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(bottom_gutter_position),
        Collider(vertical_gutter_shape),
    ));
}
