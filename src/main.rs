use bevy::prelude::*;
use bevy::render::mesh::shape::Plane;
use bevy::window::PrimaryWindow;

pub const CAMERA_SPEED: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, scene_setup)
        .add_systems(Update, controller)
        .add_systems(Update, cursor_to_ground)
        .run();
}

#[derive(Resource, Default)]

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct GroundPlane;

#[derive(Component)]
pub struct CursorPoint;

fn scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //spawns a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    //spawns a camera
    let camera = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-15.0, 5.0, -15.0).looking_at(Vec3::new(0.0, 5.0,0.0), Vec3::Y),
        ..default()
    }).id();

    let player = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule { radius: 1.0, ..default() })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player,
    )).id();

    commands.entity(player).push_children(&[camera]);

    //floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(200.0).into()),
            material: materials.add(Color::rgb(0.0, 1.0, 0.5).into()),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        GroundPlane,
    ));

    //floating cursor poinr
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {radius: 0.5, ..default() })),
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            ..default()
        },
        CursorPoint,
    ));
}

pub fn controller(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
        direction += Vec3::new(1.0, 0.0, -1.0);
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += Vec3::new(-1.0, 0.0, 1.0);
    }
    if keyboard_input.pressed(KeyCode::W) {
        direction += Vec3::new(1.0, 0.0, 1.0);
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += Vec3::new(-1.0, 0.0, -1.0);
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    player_transform.translation += direction * CAMERA_SPEED * time.delta_seconds();

    if mouse_input.just_pressed(MouseButton::Left){
        println!("yass");
    }

}

pub fn cursor_to_ground (
    mut q_cursor: Query<&mut Transform, With<CursorPoint>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
    q_plane: Query<&GlobalTransform, With<GroundPlane>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let ground_transform = q_plane.single();
    let window = q_window.single();
    let mut cursor = q_cursor.single_mut();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let plane_origin = ground_transform.translation();
    let plane = Vec3::Y;

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        return;
    };

    let global_cursor = ray.get_point(distance);

    eprintln!("Global cursor coords: {}/{}/{}",
        global_cursor.x, global_cursor.y, global_cursor.z
    );

    cursor.translation = global_cursor;
}