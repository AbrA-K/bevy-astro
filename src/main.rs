use std::{ops::Neg, time::Duration};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input,
    prelude::*,
    winit::WinitSettings,
};

use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

mod spawn_asteroids;
mod spawn_doublers;
use rand::prelude::*;
use spawn_asteroids::{AsteroidQueue, FactoryParent, SpriteClone};
use spawn_asteroids::{Factory, SpriteCopy};
use spawn_doublers::{EnemyHelth, TowerQueue, TowerTimer};

#[derive(Component)]
struct ShootingSpeed {
    speed: Timer,
}

#[derive(Resource)]
struct Poller(Timer);

#[derive(Component)]
pub struct Shot;

#[derive(Component)]
struct DeathScreenUi;

#[derive(Component)]
struct DebugRec;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct DropAfter {
    time: f32,
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct TimeDisplay;

#[derive(Resource)]
pub struct BoardSize {
    size: f32,
}

#[derive(Resource)]
pub struct Score {
    score: f32,
}

#[derive(Resource)]
pub struct TimeCounter {
    score: f32,
}

#[derive(PartialEq)]
pub enum GameState {
    TitleScreen,
    Running,
    Died,
    Won,
}

#[derive(Resource)]
pub struct CurrentGame {
    pub state: GameState,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                // mode: WindowMode::Fullscreen,
                ..default()
            },
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))
        .insert_resource(BoardSize { size: 800.0 })
        .insert_resource(WinitSettings::game())
        .insert_resource(Score { score: 0.0 })
        .insert_resource(TimeCounter { score: 0.0 })
        .insert_resource(AudioDefault)
        .insert_resource(Audio1)
        .insert_resource(Audio2)
        .insert_resource(Audio3)
        .insert_resource(AudioLast)
        .insert_resource(TowerQueuer {
            single: Timer::from_seconds(5., TimerMode::Once),
            double: Timer::from_seconds(25., TimerMode::Once),
            iteration: 0,
        })
        .insert_resource(AsteroidQueuer {
            single: Timer::from_seconds(5., TimerMode::Once),
            tripple: Timer::from_seconds(20., TimerMode::Once),
            iteration: 0,
        })
        .insert_resource(CurrentGame {
            state: GameState::Running,
        })
        .insert_resource(Poller(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_audio_channel::<AudioDefault>()
        .add_audio_channel::<Audio1>()
        .add_audio_channel::<Audio2>()
        .add_audio_channel::<Audio3>()
        .add_audio_channel::<AudioLast>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugin(spawn_asteroids::SpawnTimesAsteroids)
        .add_plugin(spawn_doublers::SpawnTimesDoublers)
        .add_plugin(AudioPlugin)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(spawn_base_cubes)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(setup_audio)
        .add_startup_system(show_score)
        .add_system(end_screen)
        .add_system(check_win_condition)
        .add_system(fix_volume)
        .add_system(update_time)
        .add_system(reset)
        .add_system(on_death)
        .add_system(queue_enemies)
        .add_system(rezise_base_cube)
        .add_system(handle_input)
        .add_system(move_camera)
        // .add_system(collide)
        .add_system(drop_them)
        .add_system(blink_system)
        .run();
}

#[derive(Resource)]
struct AudioDefault;

#[derive(Resource)]
struct Audio1;

#[derive(Resource)]
struct Audio2;

#[derive(Resource)]
struct Audio3;

#[derive(Resource)]
struct AudioLast;

fn setup_audio(
    audio_def: Res<AudioChannel<AudioDefault>>,
    audio_1: Res<AudioChannel<Audio1>>,
    audio_2: Res<AudioChannel<Audio2>>,
    audio_3: Res<AudioChannel<Audio3>>,
    audio_last: Res<AudioChannel<AudioLast>>,
    asset_server: Res<AssetServer>,
) {
    audio_def
        .play(asset_server.load("astroaudio/default.wav"))
        .looped()
        .fade_in(AudioTween::new(
            Duration::from_secs(5),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.3);
    audio_1
        .play(asset_server.load("astroaudio/1.wav"))
        .looped()
        .with_volume(0.0)
        .fade_in(AudioTween::new(
            Duration::from_secs(5),
            AudioEasing::OutPowi(2),
        ));
    audio_2
        .play(asset_server.load("astroaudio/2.wav"))
        .looped()
        .fade_in(AudioTween::new(
            Duration::from_secs(5),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.0);
    audio_3
        .play(asset_server.load("astroaudio/3.wav"))
        .looped()
        .fade_in(AudioTween::new(
            Duration::from_secs(5),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.0);
    audio_last
        .play(asset_server.load("astroaudio/last.wav"))
        .looped()
        .fade_in(AudioTween::new(
            Duration::from_secs(5),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.0);
}

fn fix_volume(
    audio_def: Res<AudioChannel<AudioDefault>>,
    audio_1: Res<AudioChannel<Audio1>>,
    audio_2: Res<AudioChannel<Audio2>>,
    audio_3: Res<AudioChannel<Audio3>>,
    audio_last: Res<AudioChannel<AudioLast>>,
    enemy_helth: Query<&EnemyHelth>,
    mut polling: ResMut<Poller>,
    time: Res<Time>,
) {
    if !polling.0.tick(time.delta()).just_finished() {
        return;
    }
    let mut sum_health: u16 = 0;
    for enemy in enemy_helth.iter() {
        sum_health += enemy.health as u16;
    }
    let mut tention = 0;
    if (5..10).contains(&sum_health) {
        tention = 1;
    } else if (10..20).contains(&sum_health) {
        tention = 2;
    } else if (20..30).contains(&sum_health) {
        tention = 3;
    } else if (30..).contains(&sum_health) {
        tention = 4;
    }

    audio_1
        .set_volume(0.0)
        .fade_in(AudioTween::linear(Duration::from_secs(3)));
    audio_2
        .set_volume(0.0)
        .fade_in(AudioTween::linear(Duration::from_secs(3)));
    audio_3
        .set_volume(0.0)
        .fade_in(AudioTween::linear(Duration::from_secs(3)));
    audio_last
        .set_volume(0.0)
        .fade_in(AudioTween::linear(Duration::from_secs(3)));

    if tention == 0 {
        audio_def
            .set_volume(0.3)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
    }
    if tention >= 1 {
        audio_1
            .set_volume(0.3)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
    }
    if tention >= 2 {
        audio_2.set_volume(0.3);
        audio_def
            .set_volume(0.3)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
    }
    if tention >= 3 {
        audio_3
            .set_volume(0.3)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
        audio_def
            .set_volume(0.0)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
    }
    if tention == 4 {
        audio_last
            .set_volume(0.3)
            .fade_in(AudioTween::linear(Duration::from_secs(3)));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn move_camera(
    mut camera: Query<(&mut Transform, With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
) {
    camera.single_mut().0.translation = player.single().translation.clone();
}

fn spawn_base_cubes(
    mut commands: Commands,
    board_size: Res<BoardSize>,
    asset_server: Res<AssetServer>,
) {
    let rows = 10;
    let column = 5;
    let offset_rows = board_size.size * ((rows - 1) / 2) as f32; // 2*size - placex * size
    let offset_column = board_size.size * ((column - 1) / 2) as f32; // size
    let mut ground_copies: Vec<Entity> = vec![];
    for place_x in 0..rows {
        for place_y in 0..column {
            let g = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(
                                board_size.size + 5.0,
                                board_size.size + 5.0,
                            )),
                            ..default()
                        },
                        texture: asset_server.load("ground.png"),
                        transform: Transform::from_xyz(
                            offset_rows - (place_x as f32 * board_size.size),
                            offset_column - (place_y as f32 * board_size.size),
                            -100.,
                        ),
                        ..default()
                    },
                    DebugRec,
                    SpriteCopy,
                    SpriteClone {
                        x: place_x,
                        y: place_y,
                    },
                ))
                .id();
            ground_copies.push(g);
        }
    }

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(board_size.size + 5.0, board_size.size + 5.0)),
                    ..default()
                },
                texture: asset_server.load("ground.png"),
                transform: Transform::from_xyz(0.0, 0.0, -100.0),
                ..default()
            },
            SpriteCopy,
            DebugRec,
        ))
        .push_children(&ground_copies);
}

fn rezise_base_cube(
    mut debug_cubes: Query<(&mut Transform, &mut Sprite, With<DebugRec>)>,
    board_size: Res<BoardSize>,
) {
    if board_size.is_changed() {
        for mut cube in debug_cubes.iter_mut() {
            cube.1.custom_size = Some(Vec2 {
                x: board_size.size + 5.0,
                y: board_size.size + 5.0,
            })
        }
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        ShootingSpeed {
            speed: Timer::from_seconds(0.05, TimerMode::Repeating),
        },
        SpriteBundle {
            texture: asset_server.load("player.png"),
            transform: Transform::from_xyz(0., 100., 0.),
            ..Default::default()
        },
        RigidBody::Dynamic,
        AdditionalMassProperties::Mass(1.),
        GravityScale(0.),
        Velocity {
            linvel: Vec2 { x: 0.0, y: 0.0 },
            angvel: 0.0,
        },
        Damping {
            linear_damping: 2.0,
            angular_damping: 0.5,
        },
        ExternalImpulse {
            ..Default::default()
        },
        Collider::ball(8.),
    ));
}

fn blink_system(
    mut query: Query<(&mut Transform, Without<SpriteCopy>, Without<Player>)>,
    board_size: Res<BoardSize>,
    mut player: Query<(&mut Transform, With<Player>)>,
    mut shots: Query<(
        &mut Transform,
        With<DropAfter>,
        Without<Player>,
        With<SpriteCopy>,
    )>,
) {
    for (mut transform, _, _) in query.iter_mut() {
        blink(&mut transform.translation, board_size.size);
    }
    let old_player_pos = player.single().0.translation;
    if blink(&mut player.single_mut().0.translation, board_size.size) {
        for (mut shot, _, _, _) in shots.iter_mut() {
            shot.translation -= old_player_pos - player.single().0.translation;
        }
    }
}

fn blink(translation: &mut Vec3, board_size: f32) -> bool {
    let mut blinked = false;
    let mut curr_translation = translation.clone();
    let max_val = board_size / 2.0;
    let min_val = max_val.neg();

    if min_val > curr_translation.x || curr_translation.x > max_val {
        blinked = true;
        curr_translation.x = ((curr_translation.x + max_val).rem_euclid(board_size)) - max_val
    };
    if min_val > curr_translation.y || curr_translation.y > max_val {
        blinked = true;
        curr_translation.y = ((curr_translation.y + max_val).rem_euclid(board_size)) - max_val
    };
    *translation = Vec3 {
        x: curr_translation.x,
        y: curr_translation.y,
        z: 0.,
    };
    blinked
}

fn handle_input(
    time: Res<Time>,
    game_state: Res<CurrentGame>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(
        &mut ExternalImpulse,
        &mut Transform,
        &mut ShootingSpeed,
        With<Player>,
    )>,
    asset_server: Res<AssetServer>,
    mut board_size: ResMut<BoardSize>,
) {
    if game_state.state != GameState::Running {
        return;
    }
    let forward = query.single().1.local_x();
    let window = windows.get_primary().unwrap();

    if let Some(targ) = window.physical_cursor_position() {
        let angle = (targ.as_vec2()
            - Vec2 {
                x: window.width() / 2.,
                y: window.height() / 2.,
            })
        .angle_between(Vec2::X);

        query.single_mut().1.rotation = Quat::from_rotation_z(-(angle + 1.571));
    }

    if keys.just_pressed(KeyCode::Q) {
        board_size.size = board_size.size - 10.;
    }

    if keys.just_pressed(KeyCode::E) {
        board_size.size = board_size.size + 10.
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        query.single_mut().0.impulse = Vec2 {
            x: forward.y * -2000. * time.delta_seconds(),
            y: forward.x * 2000. * time.delta_seconds(),
        };
    }

    if query
        .single_mut()
        .2
        .speed
        .tick(time.delta())
        .just_finished()
        && mouse_buttons.pressed(MouseButton::Right)
    {
        let mut rng = rand::thread_rng();
        let random_f32 = rng.gen_range(-0.1..0.1);
        let mut direction = forward.normalize();
        direction.x += random_f32;
        let random_f32 = rng.gen_range(-0.1..0.1);
        direction.y += random_f32;
        let speed = 2000.0;

        commands
            .spawn(RigidBody::Dynamic)
            .insert(SpatialBundle {
                transform: query.single_mut().1.clone(),
                ..Default::default()
            })
            .insert(Velocity {
                angvel: 0.,
                linvel: Vec2 {
                    x: direction.y * -speed,
                    y: direction.x * speed,
                },
            })
            .insert(Collider::cuboid(1.0, 5.0))
            .insert(DropAfter { time: 1. })
            .insert(GravityScale(0.));

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("shot.png"),
                transform: query.single_mut().1.clone(),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Velocity {
                angvel: 0.,
                linvel: Vec2 {
                    x: direction.y * -speed,
                    y: direction.x * speed,
                },
            },
            SpriteCopy,
            Shot,
            GravityScale(0.),
            DropAfter { time: 1. },
        ));
    }
}

fn drop_them(mut commands: Commands, mut query: Query<(Entity, &mut DropAfter)>, time: Res<Time>) {
    for mut q in query.iter_mut() {
        if q.1.time > 0. {
            q.1.time = q.1.time - time.delta_seconds_f64() as f32;
        } else {
            commands.entity(q.0).despawn_recursive();
        }
    }
}

fn update_time(
    mut score: ResMut<Score>,
    mut time_counter: ResMut<TimeCounter>,
    time: Res<Time>,
    mut score_display: Query<&mut Text, With<ScoreDisplay>>,
    mut time_display: Query<(&mut Text, With<TimeDisplay>, Without<ScoreDisplay>)>,
) {
    time_counter.score += time.delta().as_secs_f32();
    score.score += time.delta().as_secs_f32();
    score_display.single_mut().sections[0].value = format!("Score: {}", score.score);
    time_display.single_mut().0.sections[0].value = format!("Time:  {}", time_counter.score);
}

fn show_score(mut commands: Commands, score: Res<Score>, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                max_size: Size::UNDEFINED,
                flex_grow: 1.0,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ScoreDisplay,
                TextBundle::from_section(
                    format!("Score: {}", &score.score),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        font: asset_server
                            .load("JetBrains Mono Medium Nerd Font Complete Mono.ttf"),
                    },
                )
                .with_style(Style {
                    flex_shrink: 0.,
                    size: Size::new(Val::Undefined, Val::Px(20.)),
                    ..Default::default()
                }),
            ));
            parent.spawn((
                TimeDisplay,
                TextBundle::from_section(
                    format!("Time:  {}", &score.score),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        font: asset_server
                            .load("JetBrains Mono Medium Nerd Font Complete Mono.ttf"),
                    },
                )
                .with_style(Style {
                    flex_shrink: 0.,
                    size: Size::new(Val::Undefined, Val::Px(20.)),
                    ..Default::default()
                }),
            ));
        });
}

fn end_screen(
    mut commands: Commands,
    score: Res<Score>,
    end_screen: Query<Entity, With<DeathScreenUi>>,
    asset_server: Res<AssetServer>,
    game_state: Res<CurrentGame>,
) {
    if !game_state.is_changed() {
        return;
    }

    let mut title = "";
    match game_state.state {
        GameState::Died => {
            title = "nah you bad. r to restart";
        }
        GameState::Won => title = "gg",
        _ => {
            println!("nothing to do");
            if let Ok(ui) = end_screen.get_single() {
                commands.entity(ui).despawn_recursive();
            }
            return;
        }
    }

    if let Ok(_) = end_screen.get_single() {
        // don't do a thing if it exists
        return;
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Auto,
                        height: Val::Px(60.),
                    },
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            DeathScreenUi,
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle::from_section(
                format!("{title}, Score: {}", &score.score),
                TextStyle {
                    font_size: 70.0,
                    color: Color::WHITE,
                    font: asset_server.load("JetBrains Mono Medium Nerd Font Complete Mono.ttf"),
                },
            ),));
        });
}

#[derive(Resource)]
struct AsteroidQueuer {
    single: Timer,
    tripple: Timer,
    iteration: u8,
}

#[derive(Resource)]
struct TowerQueuer {
    single: Timer,
    double: Timer,
    iteration: u8,
}

fn queue_enemies(
    mut tower_queuer: ResMut<TowerQueuer>,
    mut asteroid_queuer: ResMut<AsteroidQueuer>,
    time: Res<Time>,
    mut as_que: ResMut<AsteroidQueue>,
    mut tower_queue: ResMut<TowerQueue>,
    board_size: Res<BoardSize>,
) {
    if asteroid_queuer.single.tick(time.delta()).just_finished() {
        as_que.queue.push(Timer::from_seconds(3., TimerMode::Once));
    }
    if asteroid_queuer.tripple.tick(time.delta()).just_finished() {
        let mut single_time = 10. - asteroid_queuer.iteration as f32;
        let mut double_time = 20. - 2.0 * asteroid_queuer.iteration as f32;
        if single_time < 1.0 {
            single_time = 1.0
        }
        if double_time < 5.0 {
            double_time = 5.0
        }
        as_que.queue.push(Timer::from_seconds(3., TimerMode::Once));
        as_que.queue.push(Timer::from_seconds(6., TimerMode::Once));
        as_que.queue.push(Timer::from_seconds(9., TimerMode::Once));
        asteroid_queuer.iteration += 1;
        asteroid_queuer.single = Timer::from_seconds(single_time, TimerMode::Once);
        asteroid_queuer.tripple = Timer::from_seconds(double_time, TimerMode::Once);
    }

    if tower_queuer.single.tick(time.delta()).just_finished() {
        tower_queue.queue.push(TowerTimer::new(&board_size.size));
    }
    if tower_queuer.double.tick(time.delta()).just_finished() {
        let mut single_time = 15. - asteroid_queuer.iteration as f32;
        let mut tripple_time = 30. - 3.0 * asteroid_queuer.iteration as f32;
        if single_time < 1.0 {
            single_time = 1.0
        }
        if tripple_time < 5.0 {
            tripple_time = 5.0
        }
        tower_queue.queue.push(TowerTimer::new(&board_size.size));
        tower_queue.queue.push(TowerTimer::new(&board_size.size));
        tower_queue.queue.push(TowerTimer::new(&board_size.size));
        tower_queuer.iteration += 1;
        tower_queuer.single = Timer::from_seconds(single_time, TimerMode::Once);
        tower_queuer.double = Timer::from_seconds(tripple_time, TimerMode::Once);
    }
}

fn on_death(
    game_state: Res<CurrentGame>,
    mut board_size: ResMut<BoardSize>,
    mut score: ResMut<Score>,
    mut time_counter: ResMut<TimeCounter>,
) {
    if game_state.state == GameState::Died {
        board_size.size = -500.0;

        score.score = 0.0;
        time_counter.score = 0.0;
    }
}

fn reset(
    mut commands: Commands,
    mut game_state: ResMut<CurrentGame>,
    enemies: Query<Entity, With<EnemyHelth>>,
    mut board_size: ResMut<BoardSize>,
    mut tower_queuer: ResMut<TowerQueuer>,
    mut asteroid_queuer: ResMut<AsteroidQueuer>,
    mut as_que: ResMut<AsteroidQueue>,
    mut tower_queue: ResMut<TowerQueue>,
    keys: Res<Input<KeyCode>>,
    mut score: ResMut<Score>,
    mut time_counter: ResMut<TimeCounter>,
    mut factory_transform: Query<&mut Transform, With<FactoryParent>>,
) {
    if !keys.just_pressed(KeyCode::R) {
        return;
    }

    for enemy in enemies.iter() {
        commands.entity(enemy).despawn_recursive();
    }

    as_que.queue = vec![];
    tower_queue.queue = vec![];

    tower_queuer.single = Timer::from_seconds(5., TimerMode::Once);
    tower_queuer.double = Timer::from_seconds(25., TimerMode::Once);
    tower_queuer.iteration = 0;

    asteroid_queuer.single = Timer::from_seconds(5., TimerMode::Once);
    asteroid_queuer.tripple = Timer::from_seconds(20., TimerMode::Once);
    asteroid_queuer.iteration = 0;

    score.score = 0.0;
    time_counter.score = 0.0;

    board_size.size = 800.0;
    factory_transform.single_mut().translation = Vec3 {
        x: 10.,
        y: 10.,
        z: 0.0,
    };
    game_state.state = GameState::Running;
}

fn check_win_condition(mut game_state: ResMut<CurrentGame>, time_counter: Res<TimeCounter>) {
    if time_counter.score > 100.0 {
        game_state.state = GameState::Won;
    }
}
