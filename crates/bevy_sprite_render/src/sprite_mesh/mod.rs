use bevy_app::{Plugin, Update};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed, Or},
    schedule::IntoScheduleConfigs,
    system::{Commands, Local, Query, Res, ResMut},
};
use tracing::error;

use bevy_image::TextureAtlasLayout;
use bevy_math::{primitives::Rectangle, vec2};
use bevy_mesh::{Mesh, Mesh2d, MeshTag};

use bevy_platform::collections::HashMap;
use bevy_sprite::{prelude::SpriteMesh, Anchor};

mod sprite_material;
pub use sprite_material::*;

use crate::MeshMaterial2d;

pub struct SpriteMeshPlugin;

impl Plugin for SpriteMeshPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(SpriteMaterialPlugin);

        app.add_systems(Update, (add_mesh, add_material).chain());
    }
}

// Insert a Mesh2d quad each time the SpriteMesh component is added.
// The meshhandle is kept locally so they can be cloned.
fn add_mesh(
    sprites: Query<Entity, Added<SpriteMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut quad: Local<Option<Handle<Mesh>>>,
    mut commands: Commands,
) {
    if quad.is_none() {
        *quad = Some(meshes.add(Rectangle::from_size(vec2(1.0, 1.0))));
    }
    for entity in sprites {
        if let Some(quad) = &*quad {
            commands.entity(entity).insert(Mesh2d(quad.clone()));
        } else {
            error!("Cannot find quad handle for SpriteMesh!");
        }
    }
}

// Change the material when SpriteMesh is added / changed.
//
// NOTE: This also adds the SpriteAtlasLayout into the SpriteMaterial,
// but this should instead be read later, similar to the images, allowing
// for hot reload.
fn add_material(
    sprites: Query<
        (Entity, &SpriteMesh, &Anchor),
        Or<(Changed<SpriteMesh>, Changed<Anchor>, Added<Mesh2d>)>,
    >,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut cached_materials: Local<HashMap<SpriteMaterial, Handle<SpriteMaterial>>>,
    mut materials: ResMut<Assets<SpriteMaterial>>,
    mut commands: Commands,
) {
    for (entity, sprite, anchor) in sprites {
        let mut material = SpriteMaterial::from_sprite_mesh(sprite.clone());
        material.anchor = **anchor;

        if let Some(texture_atlas) = &sprite.texture_atlas
            && let Some(texture_atlas_layout) = texture_atlas_layouts.get(texture_atlas.layout.id())
        {
            material.texture_atlas_layout = Some(texture_atlas_layout.clone());
        }

        if let Some(handle) = cached_materials.get(&material) {
            commands
                .entity(entity)
                .insert(MeshMaterial2d(handle.clone()));
        } else {
            let handle = materials.add(material.clone());
            cached_materials.insert(material, handle.clone());

            commands
                .entity(entity)
                .insert(MeshMaterial2d(handle.clone()));
        }

        if let Some(atlas) = &sprite.texture_atlas {
            commands.entity(entity).insert(MeshTag(atlas.index as u32));
        } else {
            commands.entity(entity).insert(MeshTag(u32::MAX));
        }
    }
}
