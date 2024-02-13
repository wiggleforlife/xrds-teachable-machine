use bevy::asset::AssetMetaCheck;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::WindowResolution;
use bevy_easings::*;
use bevy_mod_reqwest::bevy_eventlistener::callbacks::ListenerInput;
use bevy_mod_reqwest::reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use bevy_mod_reqwest::reqwest::{Request, Url};
use bevy_mod_reqwest::*;
use rand::Rng;

const CAR_SIZE_HALF: Vec2 = Vec2::new(100., 100.);
const CAR_LEFT_LANE: Vec3 = Vec3::new(-160., -200., 2.);
const CAR_RIGHT_LANE: Vec3 = Vec3::new(160., -200., 2.);

const OBSTACLE_SPAWN: Transform = Transform {
    translation: Vec3::new(0., 240., 1.),
    scale: Vec3::new(0.05, 0.05, 1.),
    rotation: Quat::from_xyzw(0., 0., 0., 0.),
};
const OBSTACLE_DESPAWN: ([Vec3; 2], Vec3) = (
    [Vec3::new(-200., -350., 1.), Vec3::new(160., -350., 1.)],
    Vec3::new(1.8, 1.8, 1.),
);

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Bing bong"),
                    resolution: WindowResolution::new(1280., 720.)
                        .with_scale_factor_override(1.),
                    resizable: false,

                    ..default()
                }),
                ..default()
            }),
            ReqwestPlugin::default(),
            EasingsPlugin,
        ))
        .add_state::<GameState>()
        .add_event::<PositionGetEvent>()
        .add_event::<CrashEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                //TODO surely there's a better way
                spawn_obstacles.run_if(
                    in_state(GameState::Playing)
                        .and_then(on_timer(Duration::from_secs(2))),
                ),
                update_car_position.run_if(in_state(GameState::Playing)),
                update_obstacle_position.run_if(in_state(GameState::Playing)),
                get_body_position.run_if(
                    in_state(GameState::Playing)
                        .and_then(on_timer(Duration::from_millis(100))),
                ),
                handle_crash.run_if(in_state(GameState::Playing)),
            ),
        )
        .run();
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Playing,
    Crashed,
}

/// Marker for the primary 2D camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Car;
#[derive(Component)]
struct Obstacle;
#[derive(Component)]
struct HasEase(bool);

#[derive(Event)]
struct CrashEvent;

#[derive(Clone, Default, Resource)]
struct Assets {
    background: Handle<Image>,
    car: Handle<Image>,
    obstacles: Vec<Handle<Image>>,
    crash: Handle<Image>,
}

fn setup(
    mut commands: Commands, client: BevyReqwest, server: Res<AssetServer>,
) {
    let assets = Assets {
        background: server.load("background.png"),
        car: server.load("car.png"),
        obstacles: vec![
            server.load("obstacles/barrels.png"),
            server.load("obstacles/pothole.png"),
        ],
        crash: server.load("boom.png"),
    };
    commands.insert_resource(assets.clone());

    let url: Url = "http://localhost:5000/api/pos".try_into().unwrap();
    let req = client
        .client()
        .get(url)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .build()
        .expect("Failed to make GET request");

    commands.insert_resource(RequestResource(req));

    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands.spawn(SpriteBundle { texture: assets.background, ..default() });
    commands.spawn((
        SpriteBundle {
            texture: assets.car,
            transform: Transform { translation: CAR_LEFT_LANE, ..default() },
            ..default()
        },
        Car,
    ));
}

fn spawn_obstacles(
    mut commands: Commands, assets: Res<Assets>,
    mut obstacles: Query<(&Transform, &Sprite), With<Obstacle>>,
) {
    let mut rng = rand::thread_rng();
    let mut obstacles = obstacles.iter_mut().collect::<Vec<_>>();
    obstacles.sort_by(|a, b| {
        a.0.translation
            .y
            .partial_cmp(&b.0.translation.y)
            .expect("Failed to order obstacles")
    });

    commands.spawn((
        SpriteBundle {
            texture: assets.obstacles[rng.gen_range(0..2)].clone(),
            transform: OBSTACLE_SPAWN,
            ..default()
        },
        HasEase(false),
        Obstacle,
    ));
}

#[derive(Event)]
struct PositionGetEvent(String);
impl From<ListenerInput<ReqResponse>> for PositionGetEvent {
    fn from(value: ListenerInput<ReqResponse>) -> Self {
        let s = value.as_string().unwrap();
        PositionGetEvent(s)
    }
}

#[derive(Resource)]
struct RequestResource(Request);

fn get_body_position(mut client: BevyReqwest, req: Res<RequestResource>) {
    client.send(
        req.0.try_clone().expect("Failed to clone request"),
        On::send_event::<PositionGetEvent>(),
    );
}

fn update_car_position(
    mut events: EventReader<PositionGetEvent>,
    mut car: Query<(&mut Transform, &mut Sprite), With<Car>>,
) {
    if events.is_empty() {
        return;
    }

    let event = events.read().last();
    let event = event.unwrap();
    let mut car = car.single_mut();

    let x = i32::from_str(&*event.0)
        .expect(&format!("Failed to parse response body - {:?}", &*event.0));

    (car.0.translation, car.1.flip_x) =
        if x > 0 { (CAR_RIGHT_LANE, true) } else { (CAR_LEFT_LANE, false) };
}

fn update_obstacle_position(
    mut commands: Commands,
    mut obstacle_sprites: Query<
        (&mut Sprite, &mut HasEase, Entity),
        (With<Obstacle>, Without<Car>),
    >,
    mut obstacle_transforms: Query<
        (&mut Transform, Entity),
        (With<Obstacle>, Without<Car>),
    >,
    mut event_writer: EventWriter<CrashEvent>,
    car: Query<&Transform, With<Car>>,
) {
    let car: Vec3 = car.single().translation;
    let car_ranges = (
        car.x - CAR_SIZE_HALF.x..car.x + CAR_SIZE_HALF.x,
        car.y - CAR_SIZE_HALF.y..car.y + CAR_SIZE_HALF.y,
    );

    let obstacle_transforms: Vec<_> = obstacle_transforms.iter_mut().collect();
    let mut obstacle_sprites: Vec<_> = obstacle_sprites.iter_mut().collect();

    for i in 0..obstacle_transforms.len() {
        if car_ranges.0.contains(&obstacle_transforms[i].0.translation.x)
            && car_ranges.0.contains(&obstacle_transforms[i].0.translation.y)
        {
            event_writer.send(CrashEvent {});
        }
        if obstacle_transforms[i].0.translation.y <= -350. {
            commands.entity(obstacle_transforms[i].1).despawn();
            return;
        }
        //TODO some of this might've been so unnecessary, i forgot to add the plugin... try inserting when spawning og entity
        if !obstacle_sprites[i].1 .0 {
            commands.entity(obstacle_transforms[i].1).insert(
                obstacle_transforms[i].0.ease_to(
                    Transform::default()
                        .with_translation(
                            OBSTACLE_DESPAWN.0
                                [rand::thread_rng().gen_range(0..2)],
                        )
                        .with_scale(OBSTACLE_DESPAWN.1),
                    EaseFunction::QuarticIn,
                    EasingType::Once { duration: Duration::from_secs(4) },
                ),
            );
            obstacle_sprites[i].1 .0 = true;
        }
    }
}

fn handle_crash(
    mut commands: Commands, events: EventReader<CrashEvent>,
    assets: Res<Assets>, mut state: ResMut<NextState<GameState>>,
    mut entities: Query<Entity, With<Sprite>>,
) {
    if events.is_empty() {
        return;
    }

    for entity in entities.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    state.set(GameState::Crashed);
    commands.spawn(SpriteBundle { texture: assets.crash.clone(), ..default() });
}
