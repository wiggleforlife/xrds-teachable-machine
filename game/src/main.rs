use bevy::asset::AssetMetaCheck;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::WindowResolution;
use bevy_mod_reqwest::bevy_eventlistener::callbacks::ListenerInput;
use bevy_mod_reqwest::reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use bevy_mod_reqwest::reqwest::{Request, Url};
use bevy_mod_reqwest::*;
use rand::Rng;

const RES: UVec2 = UVec2::new(1280, 720);

const CAR_LEFT_LANE: Vec3 = Vec3::new(-160., -200., 0.);
const CAR_RIGHT_LANE: Vec3 = Vec3::new(160., -200., 0.);

const OBSTACLE_SPAWN: [Vec3; 2] =
    [Vec3::new(-160., 300., 0.), Vec3::new(160., 300., 0.)];

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
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_obstacles,
                update_car_position,
                update_obstacle_position,
                get_body_position.run_if(on_timer(Duration::from_millis(100))),
            ),
        )
        .add_event::<PositionGetEvent>()
        .run();
}

/// Marker for the primary 2D camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Car;
#[derive(Component)]
struct Obstacle;

#[derive(Clone, Default, Resource)]
struct Assets {
    background: Handle<Image>,
    car: Handle<Image>,
    obstacles: Vec<Handle<Image>>,
}

fn setup(
    mut commands: Commands, mut client: BevyReqwest, server: Res<AssetServer>,
) {
    let assets = Assets {
        background: server.load("background.png"),
        car: server.load("car.png"),
        obstacles: vec![
            server.load("obstacles/barrels.png"),
            server.load("obstacles/pothole.png"),
        ],
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

    if obstacles.len() < 3
        && (obstacles.is_empty()
            || obstacles.last().unwrap().0.translation.y < 100.)
        && rng.gen_range(0..100) == 27
    {
        commands.spawn((
            SpriteBundle {
                texture: assets.obstacles[rng.gen_range(0..2)].clone(),
                transform: Transform {
                    translation: OBSTACLE_SPAWN[rng.gen_range(0..2)],
                    ..default()
                },
                ..default()
            },
            Obstacle,
        ));
    }
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
    let event = events.read().last();
    if event.is_none() {
        return;
    }
    let event = event.unwrap();
    let mut car = car.single_mut();

    let x = i32::from_str(&*event.0)
        .expect(&format!("Failed to parse response body - {:?}", &*event.0));

    (car.0.translation, car.1.flip_x) =
        if x > 0 { (CAR_RIGHT_LANE, true) } else { (CAR_LEFT_LANE, false) };
}

fn update_obstacle_position(
    mut commands: Commands,
    mut obstacles: Query<(&mut Transform, &mut Sprite, Entity), With<Obstacle>>,
) {
    for mut obstacle in obstacles.iter_mut() {
        if obstacle.0.translation.y <= -350. {
            commands.entity(obstacle.2).despawn();
        }
        obstacle.0.translation.y -= 1f32;
    }
}
