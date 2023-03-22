use std::time::Duration;

use crate::spawn_asteroids::Asteroid;
use crate::spawn_asteroids::Factory;
use crate::Score;

use super::BoardSize;
use super::DropAfter;
use super::Player;
use super::SpriteClone;
use super::SpriteCopy;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_rapier2d::parry::shape::Ball;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

pub struct SpawnTimesDoublers;

impl Plugin for SpawnTimesDoublers {
    fn build(&self, app: &mut App) {
        app.add_startup_system(test_spawn_towers)
            .insert_resource(TowerQueue { queue: vec![] })
            .add_system(spawn_and_destroy_factories)
            .add_system(flicker)
            .add_system(fix_projectile_size)
            .add_system(spawn_projectile)
            .add_system(spawn_after_finished_queue)
            // .add_system(correct_position_on_change)
            .add_system(time_and_adjust_board);
    }
}

#[derive(Component)]
pub struct TowerField {
    pub timer: Timer,
}
#[derive(Component)]
struct Flicker(Timer);

#[derive(Component)]
struct Projectile;

#[derive(Component, Clone)]
pub struct TowerTimer {
    timer: Timer,
    factory_timer: Timer,
    projectile_timer: Timer,
    xpos: f32,
    ypos: f32,
}

impl TowerTimer {
    pub fn new(board_size: &f32) -> Self {
        let mut board_size = board_size.clone();
        if board_size < 0.0 {
            board_size = 110.0
        }
        let mut rng = rand::thread_rng();
        let range = (-board_size / 2.0) + 50.0..(board_size / 2.0) - 50.0;
        TowerTimer {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
            factory_timer: Timer::from_seconds(3.0, TimerMode::Once),
            projectile_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            xpos: rng.gen_range(range.clone()),
            ypos: rng.gen_range(range.clone()),
        }
    }
}

#[derive(Resource)]
pub struct TowerQueue {
    pub queue: Vec<TowerTimer>,
}

#[derive(Component)]
pub struct EnemyHelth {
    pub health: u8,
}

#[derive(Component)]
pub struct TowerCircle;

fn fix_projectile_size(mut projectiles: Query<(&DropAfter, Option<&mut Collider>, Option<&mut Sprite>), With<Projectile>>) {
    for mut projectile in projectiles.iter_mut() {
        let size_perc = projectile.0.time / 2.0;
        if let Some(mut collider) = projectile.1 {
            *collider = Collider::ball(15.0 * size_perc);
        }
        if let Some(mut sprite) = projectile.2 {
            sprite.custom_size = Some(Vec2 { x: 20.0, y: 20.0 }*size_perc);
        }
    }
}

fn spawn_projectile(
    mut commands: Commands,
    mut tower_timers: Query<&mut TowerTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    board_size: Res<BoardSize>,
) {
    for mut t_timer in tower_timers.iter_mut() {
        if t_timer.projectile_timer.tick(time.delta()).just_finished() {
            let mut rng = rand::thread_rng();
            let mut xvel = rng.gen_range(-1..1);
            let yvel = rng.gen_range(-1..1);
            if (xvel, yvel) == (0, 0) {
                xvel = 1;
            }

            let vel = Vec2 {
                x: xvel as f32,
                y: yvel as f32,
            }
            .normalize()
                * 300.0;

            let rows = 10;
            let column = 5;
            let offset_rows = board_size.size * ((rows - 1) / 2) as f32; // 2*size - placex * size
            let offset_column = board_size.size * ((column - 1) / 2) as f32; // size

            let mut tower_copies: Vec<Entity> = vec![];
            for place_x in 0..rows {
                for place_y in 0..column {
                    if (place_x, place_y) == (0, 0) {
                        continue;
                    }
                    let tower = commands
                        .spawn((
                            DropAfter { time: 2.0 },
                            Asteroid,
                            Projectile,
                            SpriteBundle {
                                texture: asset_server.load("tower.png"),
                                transform: Transform::from_xyz(
                                    offset_rows - (place_x as f32 * board_size.size),
                                    offset_column - (place_y as f32 * board_size.size),
                                    0.,
                                ),
                                ..Default::default()
                            },
                            SpriteCopy,
                            SpriteClone {
                                x: place_x,
                                y: place_y,
                            },
                        ))
                        .id();
                    tower_copies.push(tower);
                }
            }

            commands
                .spawn((
                    DropAfter { time: 2.0 },
                    Asteroid,
                    Projectile,
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::ball(15.0),
                    RigidBody::KinematicVelocityBased,
                    LockedAxes::ROTATION_LOCKED,
                    Velocity {
                        linvel: vel,
                        ..default()
                    },
                    SpatialBundle {
                        transform: Transform::from_xyz(t_timer.xpos, t_timer.ypos, 0.),
                        ..default()
                    },
                    TowerField {
                        timer: Timer::new(Duration::from_secs(500), TimerMode::Once),
                    },
                ))
                .push_children(&tower_copies);
        }
    }
}

fn test_spawn_towers(mut tower_queue: ResMut<TowerQueue>, board_size: Res<BoardSize>) {
    // tower_queue.queue.push(TowerTimer::new(&board_size.size));
}

fn flicker(mut flickerers: Query<(&mut Flicker, &mut Visibility)>, time: Res<Time>) {
    for mut flickerer in flickerers.iter_mut() {
        if flickerer.0 .0.tick(time.delta()).just_finished() {
            flickerer.1.toggle();
        }
    }
}

fn spawn_and_destroy_factories(
    mut commands: Commands,
    mut tower_queue: ResMut<TowerQueue>,
    mut factories: Query<Entity, With<TowerTimer>>,
    time: Res<Time>,
    board_size: Res<BoardSize>,
    asset_server: Res<AssetServer>,
) {
    for tower_factory in tower_queue.queue.iter_mut() {
        if tower_factory
            .factory_timer
            .tick(time.delta())
            .just_finished()
        {
            let rows = 10;
            let column = 5;
            let offset_rows = board_size.size * ((rows - 1) / 2) as f32;
            let offset_column = board_size.size * ((column - 1) / 2) as f32;

            let mut tower_copies: Vec<Entity> = vec![];
            for place_x in 0..rows {
                for place_y in 0..column {
                    if (place_x, place_y) == (0, 0) {
                        continue;
                    }
                    let tower = commands
                        .spawn((
                            Flicker(Timer::from_seconds(0.1, TimerMode::Repeating)),
                            DropAfter { time: 2.0 },
                            SpriteBundle {
                                texture: asset_server.load("tower.png"),
                                transform: Transform::from_xyz(
                                    offset_rows - (place_x as f32 * board_size.size),
                                    offset_column - (place_y as f32 * board_size.size),
                                    0.,
                                ),
                                ..Default::default()
                            },
                            SpriteCopy,
                            SpriteClone {
                                x: place_x,
                                y: place_y,
                            },
                        ))
                        .id();
                    tower_copies.push(tower);
                }
            }

            commands
                .spawn((
                    DropAfter { time: 2.0 },
                    Flicker(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    SpriteBundle {
                        texture: asset_server.load("tower.png"),
                        transform: Transform::from_xyz(tower_factory.xpos, tower_factory.ypos, 0.),
                        ..Default::default()
                    },
                    TowerField {
                        timer: Timer::new(Duration::from_secs(500), TimerMode::Once),
                    },
                ))
                .push_children(&tower_copies);
        }
    }
}

fn spawn_after_finished_queue(
    time: Res<Time>,
    mut tower_queue: ResMut<TowerQueue>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_size: Res<BoardSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (i, tower) in tower_queue.queue.iter_mut().enumerate() {
        if tower.timer.tick(time.delta()).finished() {
            spawn_one_doubler(
                commands,
                asset_server,
                board_size,
                meshes,
                materials,
                tower.xpos,
                tower.ypos,
                tower.clone(),
            );
            tower_queue.queue.swap_remove(i);
            break;
        }
    }
}

fn spawn_one_doubler(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_size: Res<BoardSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    xpos: f32,
    ypos: f32,
    tower_timer: TowerTimer,
) {
    let rows = 10;
    let column = 5;
    let offset_rows = board_size.size * ((rows - 1) / 2) as f32;
    let offset_column = board_size.size * ((column - 1) / 2) as f32;
    let mut rng = thread_rng();
    let random_x_pos = xpos;
    // (rng.gen_range(0..board_size.size as i32) - (board_size.size / 2.0) as i32) as f32;
    let random_y_pos = xpos;
    // (rng.gen_range(0..board_size.size as i32) - (board_size.size / 2.0) as i32) as f32;

    let mut tower_copies: Vec<Entity> = vec![];

    for place_x in 0..rows {
        for place_y in 0..column {
            let tower = commands
                .spawn((
                    Asteroid,
                    SpriteBundle {
                        texture: asset_server.load("tower.png"),
                        transform: Transform::from_xyz(
                            offset_rows - (place_x as f32 * board_size.size),
                            offset_column - (place_y as f32 * board_size.size),
                            0.,
                        ),
                        ..Default::default()
                    },
                    SpriteCopy,
                    SpriteClone {
                        x: place_x,
                        y: place_y,
                    },
                    TowerField {
                        timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
                    },
                ))
                .id();
            tower_copies.push(tower);
        }
    }

    commands
        .spawn((
            tower_timer,
            Asteroid,
            AdditionalMassProperties::Mass(0.0),
            ActiveEvents::COLLISION_EVENTS,
            Collider::ball(15.),
            EnemyHelth { health: 10 },
            SpriteBundle {
                texture: asset_server.load("tower.png"),
                transform: Transform::from_xyz(xpos, ypos, 0.),
                ..Default::default()
            },
            TowerField {
                timer: Timer::new(Duration::from_secs(500), TimerMode::Once),
            },
        ))
        .push_children(&tower_copies);
}

fn time_and_adjust_board(
    mut timers: Query<&mut TowerField, Without<SpriteCopy>>,
    time: Res<Time>,
    mut board_size: ResMut<BoardSize>,
    mut score: ResMut<Score>,
) {
    let change_threshold = 1.0;
    let original_size = 800.0;
    let mut sum_tower_size = 0.0;

    for mut timer in timers.iter_mut() {
        timer.timer.tick(time.delta());
        score.score += time.delta().as_secs_f32() / 2.0;
        sum_tower_size += timer.timer.elapsed().as_secs_f32();
    }

    let next_size: f32 = original_size - sum_tower_size;
    let change_threshold_range =
        board_size.size - change_threshold..board_size.size + change_threshold;

    if !(change_threshold_range).contains(&next_size) {
        board_size.size = next_size;
    }
}

fn correct_position_on_change(
    mut towers: Query<(
        &mut Transform,
        &SpriteClone,
        Or<(&TowerField, &TowerCircle)>,
    )>,
    board_size: Res<BoardSize>,
) {
    if board_size.is_changed() {
        for (mut transform, location, _) in towers.iter_mut() {
            let updated_x = 2.0 * board_size.size - (location.x as f32 * board_size.size);
            let updated_y = board_size.size - (location.y as f32 * board_size.size);

            transform.translation.x = updated_x;
            transform.translation.y = updated_y;
        }
    }
}
