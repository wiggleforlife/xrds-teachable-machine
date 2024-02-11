use std::str::FromStr;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use rand::Rng;
use reqwest::blocking::Client;

const RES: UVec2 = UVec2::new(1280, 720);

const CAR_LEFT_LANE: Vec3 = Vec3::new(-160., -200., 0.);
const CAR_RIGHT_LANE: Vec3 = Vec3::new(160., -200., 0.);

const OBSTACLE_SPAWN: [Vec3; 2] =
    [Vec3::new(-160., 300., 0.), Vec3::new(160., 300., 0.)];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Bing bong"),
                resolution: WindowResolution::new(1280., 720.)
                    .with_scale_factor_override(1.),
                resizable: false,

                ..default()
            }),
            ..default()
        }),))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (spawn_obstacles, update_car_position, update_obstacle_position),
        )
        .run();
}

/// Marker for the primary 2D camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Car;
#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct HttpClient(Client);

#[derive(Clone, Default, Resource)]
struct Assets {
    background: Handle<Image>,
    car: Handle<Image>,
    obstacles: Vec<Handle<Image>>,
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    let assets = Assets {
        background: server.load("background.png"),
        car: server.load("car.png"),
        obstacles: vec![
            server.load("obstacles/barrels.png"),
            server.load("obstacles/pothole.png"),
        ],
    };
    commands.insert_resource(assets.clone());
    commands.insert_resource(HttpClient(Client::new()));

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

fn update_car_position(
    mut car: Query<(&mut Transform, &mut Sprite), With<Car>>,
    client: Res<HttpClient>,
) {
    let mut car = car.single_mut();

    //TODO slow down
    let res = client
        .0
        .get("http://127.0.0.1:5000/api/pos")
        .send()
        .expect("Failed to make GET request");
    let body = &*res.text().expect("Failed to read response body");
    let x = i32::from_str(body)
        .expect(&format!("Failed to parse response body - {:?}", body));

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
