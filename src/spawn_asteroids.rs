use std::ops::Neg;

use bevy::prelude::*;
use bevy_rapier2d::parry::query;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::prelude::LockedAxes;

use crate::CurrentGame;
use crate::GameState;
use crate::spawn_doublers::EnemyHelth;
use crate::spawn_doublers::TowerField;

use super::BoardSize;
use super::DropAfter;
use super::Player;
use super::Shot;

#[derive(Component)]
pub struct SpriteCopy;

#[derive(Component)]
pub struct SpriteClone {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource)]
pub struct AsteroidQueue {
    pub queue: Vec<Timer>,
}

#[derive(Component)]
pub struct Factory(pub Timer);

pub struct SpawnTimesAsteroids;

impl Plugin for SpawnTimesAsteroids {
    fn build(&self, app: &mut App) {
        app.add_startup_system(test_add_to_queue)
            .add_startup_system(build_factory)
            .insert_resource(AsteroidQueue { queue: vec![] })
            .add_system(animate_sprite)
            .add_system(fix_visibility_factory)
            .add_system(update_spawn_queue)
            .add_system(correct_child_on_size_change)
            .add_system(target_move_player)
            .add_system(check_shooted);
    }
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn test_add_to_queue(mut queue: ResMut<AsteroidQueue>) {
    // queue.queue.push(Timer::from_seconds(5., TimerMode::Once));
}

fn fix_visibility_factory(
    mut query: Query<(&mut Factory, &mut Visibility)>,
    mut queue: Res<AsteroidQueue>,
    time: Res<Time>,
) {
    let mut min_time: f32 = 5.0;
    for timer in queue.queue.iter() {
        if timer.remaining_secs() < min_time {
            min_time = timer.remaining_secs();
        }
    }

    if min_time > 2.0 {
        for mut elem in query.iter_mut() {
            *elem.1 = Visibility { is_visible: false };
        }
        return;
    }

    for mut elem in query.iter_mut() {
        if elem.0 .0.tick(time.delta()).just_finished() {
            elem.1.toggle();
        }
    }
}

fn build_factory(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_adlases: ResMut<Assets<TextureAtlas>>,
    board_size: Res<BoardSize>,
) {
    let texture_handle = asset_server.load("asteroid.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2 { x: 31.0, y: 31.0 }, 2, 2, None, None);
    let texture_atlas_handle = texture_adlases.add(texture_atlas);

    let mut astroid_copies: Vec<Entity> = vec![];

    let rows = 10;
    let column = 5;
    let offset_rows = board_size.size * ((rows - 1) / 2) as f32; // 2*size - placex * size
    let offset_column = board_size.size * ((column - 1) / 2) as f32; // size

    for place_x in 0..rows {
        for place_y in 0..column {
            astroid_copies.push(
                commands
                    .spawn((
                        // SpriteSheetBundle {
                        //     texture_atlas: texture_atlas_handle.clone(),
                        //     transform: Transform::from_xyz(
                        //         offset_rows - (place_x as f32 * board_size.size),
                        //         offset_column - (place_y as f32 * board_size.size),
                        //         0.,
                        //     ),
                        //     ..Default::default()
                        // },
                        SpriteBundle {
                            texture: asset_server.load("buggy.png"),
                            sprite: Sprite { custom_size: Some(Vec2{ x: 55.0, y: 55.0}), ..default() },
                            transform: Transform::from_xyz(
                                offset_rows - (place_x as f32 * board_size.size),
                                offset_column - (place_y as f32 * board_size.size),
                                0.,
                            ),
                            ..Default::default()
                        },
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        SpriteCopy,
                        Factory(Timer::from_seconds(0.3, TimerMode::Repeating)),
                        SpriteClone {
                            x: place_x,
                            y: place_y,
                        },
                    ))
                    .id(),
            )
        }
    }

    commands
        .spawn((
            // SpriteSheetBundle {
            //     texture_atlas: texture_atlas_handle,
            //     transform: Transform::from_xyz(10., 10., 0.),
            //     ..Default::default()
            // },
            SpatialBundle {
                transform: Transform::from_xyz(10., 10., 0.),
                ..Default::default()
            },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ))
        .push_children(&astroid_copies);
}

fn update_spawn_queue(
    mut queue: ResMut<AsteroidQueue>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_adlases: ResMut<Assets<TextureAtlas>>,
    board_size: Res<BoardSize>,
) {
    for (i, timer) in queue.queue.iter_mut().enumerate() {
        if timer.tick(time.delta()).finished() {
            spawn_one_asteroid(commands, asset_server, texture_adlases, board_size);
            queue.queue.swap_remove(i);
            break; // hmm hacky
        }
    }
}

fn spawn_one_asteroid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_adlases: ResMut<Assets<TextureAtlas>>,
    board_size: Res<BoardSize>,
) {
    let texture_handle = asset_server.load("asteroid.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2 { x: 31.0, y: 31.0 }, 2, 2, None, None);
    let texture_atlas_handle = texture_adlases.add(texture_atlas);

    let mut astroid_copies: Vec<Entity> = vec![];

    let rows = 10;
    let column = 5;
    let offset_rows = board_size.size * ((rows - 1) / 2) as f32; // 2*size - placex * size
    let offset_column = board_size.size * ((column - 1) / 2) as f32; // size

    for place_x in 0..rows {
        for place_y in 0..column {
            astroid_copies.push(
                commands
                    .spawn((
                        // SpriteSheetBundle {
                        //     texture_atlas: texture_atlas_handle.clone(),
                        //     transform: Transform::from_xyz(
                        //         offset_rows - (place_x as f32 * board_size.size),
                        //         offset_column - (place_y as f32 * board_size.size),
                        //         0.,
                        //     ),
                        //     ..Default::default()
                        // },
                        SpriteBundle {
                            texture: asset_server.load("buggy.png"),
                            sprite: Sprite { custom_size: Some(Vec2{ x: 55.0, y: 55.0}), ..default() },
                            transform: Transform::from_xyz(
                                offset_rows - (place_x as f32 * board_size.size),
                                offset_column - (place_y as f32 * board_size.size),
                                0.,
                            ),
                            ..Default::default()
                        },
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        SpriteCopy,
                        SpriteClone {
                            x: place_x,
                            y: place_y,
                        },
                    ))
                    .id(),
            )
        }
    }
    

    commands
        .spawn((
            // SpriteSheetBundle {
            //     texture_atlas: texture_atlas_handle,
            //     transform: Transform::from_xyz(10., 10., 0.),
            //     ..Default::default()
            // },
            Collider::ball(15.),
            SpatialBundle {
                transform: Transform::from_xyz(10., 10., 0.0),
                ..Default::default()
            },
            Velocity {
                angvel: 0.0,
                linvel: Vec2 { x: 50.0, y: 80.0 },
            },
            Damping {
                linear_damping: 1.0,
                angular_damping: 50.5,
            },
            RigidBody::Dynamic,
            AdditionalMassProperties::Mass(1.),
            ExternalImpulse {
                impulse: Vec2::ZERO,
                torque_impulse: 0.0,
            },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            // Sensor,
            ActiveEvents::COLLISION_EVENTS,
            GravityScale(0.),
            Asteroid,
            EnemyHelth { health: 3 },
        )).insert(LockedAxes::ROTATION_LOCKED)
        .push_children(&astroid_copies);
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn check_shooted(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    shots: Query<(&mut Shot, Entity)>,
    mut asteroids: Query<(&mut Asteroid, Entity, Option<&mut EnemyHelth>, Option<&TowerField>)>,
    player: Query<(&mut Player, Entity)>,
    mut board_size: ResMut<BoardSize>,
    mut game_state: ResMut<CurrentGame>,
) {
    // honestly please don't bother making this better - don't waste your time
    let asteroid_entities: Vec<Entity> = asteroids.iter().map(|entry| entry.1).collect();
    let shot_entities: Vec<Entity> = shots.iter().map(|entry| entry.1).collect();

    for collision in collisions.iter() {
        if let CollisionEvent::Started(a, b, _info) = collision {
            let player_id = player.single().1.index();

            if player_id == a.index() || player_id == b.index() {
                if player_id == a.index() {
                    if asteroid_entities.contains(b) {
                        game_state.state = GameState::Died
                    }
                } else {
                    if asteroid_entities.contains(a) {
                        game_state.state = GameState::Died
                    }
                }
                continue;
            }

            if asteroids.contains(*a) && asteroids.contains(*b) {
                continue;
            }


            if let Ok(mut asteroid) = asteroids.get_mut(*a) {
                if let Some(mut health) = asteroid.2 {
                    health.health -= 1;
                    if health.health <= 0 {
                        if let Some(tower_field) = asteroid.3 {
                            board_size.size += tower_field.timer.elapsed().as_secs_f32();
                        }
                        commands.entity(*a).despawn_recursive();
                    }
                    commands.entity(*b).despawn_recursive();
                    continue;
                }
            }

            if let Ok(mut asteroid) = asteroids.get_mut(*b) {
                if let Some(mut health) = asteroid.2 {
                    health.health -= 1;
                    if health.health <= 0 {
                        if let Some(tower_field) = asteroid.3 {
                            board_size.size += tower_field.timer.elapsed().as_secs_f32();
                        }
                        commands.entity(*b).despawn_recursive();
                    }
                    commands.entity(*a).despawn_recursive();
                    continue;
                }
            }

            // commands.entity(*a).despawn_recursive();
            // commands.entity(*b).despawn_recursive();
        }
    }
}

fn correct_child_on_size_change(
    mut q_child: Query<(&Parent, &mut Transform, &SpriteClone)>,
    q_parent: Query<&GlobalTransform>,
    board_size: Res<BoardSize>,
) {
    if !board_size.is_changed() {
        return;
    }

    for (_parent, mut child_transform, location) in q_child.iter_mut() {
        let updated_x = 4.0 * board_size.size - (location.x as f32 * board_size.size);
        let updated_y = 2.0 * board_size.size - (location.y as f32 * board_size.size);

        child_transform.translation.x = updated_x;
        child_transform.translation.y = updated_y;
    }
}

fn target_move_player(
    mut commands: Commands,
    player: Query<(&Transform, With<Player>)>,
    board_size: Res<BoardSize>,
    mut asteroids: Query<(
        &mut Velocity,
        &Transform,
        &mut ExternalImpulse,
        With<Asteroid>,
    )>,
) {
    let player_pos = player.single().0.translation;

    for (mut asteroid_vel, asteroid_trans, mut asteroid_impulse, _) in asteroids.iter_mut() {
        let x_target =
            find_impulse_direction(board_size.size, asteroid_trans.translation.x, player_pos.x);
        let y_target =
            find_impulse_direction(board_size.size, asteroid_trans.translation.y, player_pos.y);

        asteroid_impulse.impulse = Vec2 {
            x: x_target,
            y: y_target,
        }
        .normalize()
            * 10.0;
    }
}

fn find_impulse_direction(board_size: f32, position: f32, target: f32) -> f32 {
    let distance = target - position;
    if distance.abs() < board_size / 2.0 {
        return distance;
    }
    let mirrowed_travel = board_size - distance.abs();
    if distance < 0.0 {
        return mirrowed_travel;
    }
    mirrowed_travel.neg()
}
