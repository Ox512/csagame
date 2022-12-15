use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

use crate::camera::CursorPos;
use crate::character::animation::SpriteSheetHandles;
use crate::terrain::bevy_connect::TilemapLayer;
use crate::terrain::node::PathNode;
use crate::terrain::Terrain;
use crate::character::*;
use crate::tile::*;

#[derive(Resource)]
pub enum CommandMode {
    ModifyTerrain,
    PathFinding,
    PlaceEntity,
}

#[derive(Resource, Default)]
pub struct PathState {
    pub start: PathNode,
    pub start_entity: Option<Entity>,
    pub goal: PathNode,
    pub goal_entity: Option<Entity>,
}

impl PathState {
    pub fn display_path(
        &mut self,
        terrain: &Terrain,
        tm_transform: &Transform,
        lines: &mut DebugLines,
    ) {
        // Generate and display path
        println!("Path: start = {:?}\tgoal = {:?}", self.start, self.goal);

        if let Some(path) = terrain.find_path(&self.start, &self.goal) {
            let mut prev = self.start;
            for node in path {
                println!("Path: {:?}", node);

                // Find the world co-ords
                let start = Vec3::new(
                    prev.x as f32 * 8.0 + tm_transform.translation.x,
                    (prev.y as f32 + 1.0) * 8.0 + tm_transform.translation.y,
                    0.0,
                );

                let end = Vec3::new(
                    node.x as f32 * 8.0 + tm_transform.translation.x,
                    (node.y as f32 + 1.0) * 8.0 + tm_transform.translation.y,
                    0.0,
                );

                lines.line(start, end, 3.0);

                prev = node;
            }
        } else {
            println!("No path found!")
        }
    }
}

// Takes all the input from the player and then does
// whatever actions have been desired by the player
pub fn resolve_mouse_input(
    mut commands: Commands,
    mut terrain: ResMut<Terrain>,
    mut path_state: ResMut<PathState>,
    mut lines: ResMut<DebugLines>,
    mut tm_query: Query<(
        Entity,
        &TilemapSize,
        &TilemapGridSize,
        &mut TileStorage,
        &TilemapLayer,
        &Transform,
    )>,
    cursor: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
    mode: Res<CommandMode>,
    handles: Res<SpriteSheetHandles>,
    asset_server: Res<AssetServer>,
) {
    // NOTE: tm_query is ordered by layer, which is very useful for ensuring that only
    //       the currently visible layer is acted upon, however this isn't guarenteed by
    //       and may break in later versions

    if mouse.just_pressed(MouseButton::Left) {
        match *mode {
            // Remove a tile
            CommandMode::ModifyTerrain => {
                for (_entity, tm_size, tm_grid_size, mut tm_storage, tm_layer, tm_transform) in
                    tm_query.iter_mut()
                {
                    let world_pos = (tm_transform.compute_matrix().inverse()
                        * Vec4::from((cursor.0, 0.0, 1.0)))
                    .xy();

                    // If the cusor is at a valid position on the tilemap
                    if let Some(tile_pos) = TilePos::from_world_pos(
                        &world_pos,
                        tm_size,
                        tm_grid_size,
                        &TilemapType::Square,
                    ) {
                        if terrain
                            .remove_tile(&mut commands, &mut tm_storage, tm_layer.0, tile_pos)
                            .is_some()
                        {
                            // Break from the loop if a tile has been removed
                            break;
                        }
                    }
                }
            }

            CommandMode::PathFinding => {
                let (_, tm_size, tm_grid_size, _, _, tm_transform) =
                    tm_query.iter_mut().next().expect("No foreground tilemap");

                let world_pos = (tm_transform.compute_matrix().inverse()
                    * Vec4::from((cursor.0, 0.0, 1.0)))
                .xy();

                // If the cusor is at a valid position on the tilemap
                if let Some(tile_pos) =
                    TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
                {
                    path_state.start = PathNode::new(tile_pos.x, tile_pos.y - 1);

                    // Create indicator entity
                    if let Some(e) = path_state.start_entity {
                        commands.entity(e).despawn_recursive();
                    }

                    let indicator = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("Start.png"),
                            ..Default::default()
                        })
                        .insert(TransformBundle::from(
                            Transform::default().with_translation(cursor.0.extend(0.0)),
                        ))
                        .id();

                    path_state.start_entity = Some(indicator);

                    path_state.display_path(&terrain, tm_transform, &mut lines);
                }
            },

            CommandMode::PlaceEntity => {
                commands.spawn(CharacterBundle::from_id(CharacterId::HumanMale, cursor.0, &handles));
            }
        }
    } else if mouse.just_pressed(MouseButton::Right) {
        match *mode {
            // place a tile
            CommandMode::ModifyTerrain => {
                let (tm_entity, tm_size, tm_grid_size, mut tm_storage, tm_layer, tm_transform) =
                    tm_query.iter_mut().next().expect("No foreground tilemap");

                let world_pos = (tm_transform.compute_matrix().inverse()
                    * Vec4::from((cursor.0, 0.0, 1.0)))
                .xy();

                // If the cusor is at a valid position on the tilemap
                if let Some(tile_pos) =
                    TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
                {
                    if tm_storage.get(&tile_pos).is_none() {
                        // Create a tile
                        let tile = Tile::new(
                            TileId::Ground(Ground::Stone),
                            Some(
                                terrain
                                    .get_surrounds(tm_layer.0, tile_pos.x, tile_pos.y)
                                    .get_texture_offset(),
                            ),
                        );

                        terrain.insert_tile(
                            &mut commands,
                            &mut tm_storage,
                            tm_transform,
                            tm_entity,
                            tm_layer.0,
                            tile_pos,
                            tile,
                        );
                    }
                }
            }

            CommandMode::PathFinding => {
                let (_, tm_size, tm_grid_size, _, _, tm_transform) =
                    tm_query.iter_mut().next().expect("No foreground tilemap");

                let world_pos = (tm_transform.compute_matrix().inverse()
                    * Vec4::from((cursor.0, 0.0, 1.0)))
                .xy();

                // If the cusor is at a valid position on the tilemap
                if let Some(tile_pos) =
                    TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
                {
                    path_state.goal = PathNode::new(tile_pos.x, tile_pos.y - 1);

                    // Create indicator entity
                    if let Some(e) = path_state.goal_entity {
                        commands.entity(e).despawn_recursive();
                    }

                    let indicator = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("Goal.png"),
                            ..Default::default()
                        })
                        .insert(TransformBundle::from(
                            Transform::default().with_translation(cursor.0.extend(0.0)),
                        ))
                        .id();

                    path_state.goal_entity = Some(indicator);

                    path_state.display_path(&terrain, tm_transform, &mut lines);
                }
            },

            CommandMode::PlaceEntity => ()
        }
    }
}

pub fn update_command_mode(kbd: Res<Input<KeyCode>>, mut mode: ResMut<CommandMode>) {
    if kbd.just_pressed(KeyCode::P) {
        *mode = CommandMode::PathFinding;
    } else if kbd.just_pressed(KeyCode::M) {
        *mode = CommandMode::ModifyTerrain;
    } else if kbd.just_pressed(KeyCode::E) {
        *mode = CommandMode::PlaceEntity;
    }
}
