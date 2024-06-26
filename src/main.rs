use bevy::prelude::*;
use bevy::window::PrimaryWindow;

mod collision;
use collision::*;

pub const CAMERA_SPEED: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CollisionPlugin))
        .add_event::<MouseFire>()
        .init_resource::<CursorToPlane>()
        .add_systems(Startup, scene_setup)
        .add_systems(Startup, cursor_setup)
        .add_systems(Update, controller)
        .add_systems(Update, cursor_update)
        .add_systems(Update, wand_aiming)
        .add_systems(Update, spell_update)
        .add_systems(Update, apply_vel)
        .run();
}

#[derive(Event)]
pub struct MouseFire;

#[derive(Resource, Default)]
pub struct CursorToPlane {
    pos: Vec3,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct GroundPlane;

#[derive(Component)]
pub struct GameCursor;

#[derive(Component)]
pub struct Spell {
    direction: Vec3,
    speed: f32,
    acc: f32,
    ttl: Timer,
}

#[derive(Component)]
pub struct Velocity {
    vel: Vec3,
}

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
        Velocity {vel: Vec3::ZERO},
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

    //obstacles
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube { size: 10.0 }.into()),
            material: materials.add(Color::rgb(0.0, 1.0, 0.5).into()),
            transform: Transform::from_translation(Vec3::new(12.0, 0.0, 12.0)),
            ..default()
        },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube { size: 10.0 }.into()),
            material: materials.add(Color::rgb(0.0, 1.0, 0.5).into()),
            transform: Transform::from_translation(Vec3::new(-12.0, 0.0, 12.0)),
            ..default()
        },
    ));

}

fn cursor_setup (
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut window = q_window.single_mut();
    window.cursor.visible = false;

    commands.spawn(
        (
            ImageBundle {
                transform: Transform::from_translation(Vec3::ZERO),
                image: asset_server.load("PNG/white/crosshair005.png").into(),
                style: Style {
                    position_type: PositionType::Absolute,
                    ..default()
                    },
                ..default()
            },
            GameCursor,
        )
    );
}

pub fn controller(
    mut e_mouse_fire: EventWriter<MouseFire>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    let mut player_velocity = player_query.single_mut();
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

    player_velocity.vel = direction * CAMERA_SPEED;

    if mouse_input.pressed(MouseButton::Left){
        e_mouse_fire.send(MouseFire);
    }

}

pub fn apply_vel (
    mut vel_and_transform: Query<(&Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (vel, mut transform) in vel_and_transform.iter_mut() {
        transform.translation += vel.vel * time.delta_seconds();
    }
}

pub fn cursor_update (
    mut q_cursor: Query<&mut Style, With<GameCursor>>,
    mut r_cursor: ResMut<CursorToPlane>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
    q_plane: Query<&GlobalTransform, With<GroundPlane>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let ground_transform = q_plane.single();
    let window = q_window.single();
    let mut cursor = q_cursor.single_mut();

    //get the cursor position if it exists
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    //change the position of the custom cursor
    cursor.left = Val::Px(cursor_position.x - 32.0);
    cursor.top = Val::Px(cursor_position.y - 32.0);

    //we can define the plane based on it's origin and a normal vector
    let plane_origin = ground_transform.translation();
    let plane_normal = Vec3::Y;

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    //intersect_plane handles the vector math and works out where the camera ray intersects the ground
    let Some(distance) = ray.intersect_plane(plane_origin, plane_normal) else {
        return;
    };

    //we can now get the position of the cursor from the distance it is down the ray
    let global_cursor = ray.get_point(distance);
    r_cursor.pos = global_cursor;

    eprintln!("Global cursor coords: {}/{}/{}",
        global_cursor.x, global_cursor.y, global_cursor.z
    );
}

fn wand_aiming (
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut e_mouse_fire: EventReader<MouseFire>,
    q_player: Query<&Transform, With<Player>>,
    r_cursor: Res<CursorToPlane>,
) {
    let player_pos = q_player.get_single().unwrap().translation;
    let mut local_cursor_dir = (r_cursor.pos - player_pos).normalize();
    local_cursor_dir.y = 0.0;

    gizmos.ray(
        player_pos,
        local_cursor_dir  * 5.0,
        Color::BLUE,
    );

    //settings for current spell
    let sp = 50.0;

    for _fire in e_mouse_fire.iter() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.1, ..default() })),
                material: materials.add(Color::rgb(0.0, 0.1, 0.8).into()),
                transform: Transform::from_translation(player_pos + (local_cursor_dir * 5.0)),
                ..default()
            },
            Spell {
                direction: local_cursor_dir.normalize(),
                speed: sp,
                acc: 0.0,
                ttl: Timer::from_seconds(5.0, TimerMode::Once),
            },
            Velocity {
                vel: local_cursor_dir.normalize() * sp,
            },
            Collider::sphere_from_radius(0.1),
        ));
    }
}

pub fn spell_update (
    mut q_spells: Query<(Entity, &mut Spell, &mut Transform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut spell, mut spell_transform) in q_spells.iter_mut() {
        //tick the spell's despawn timer
        spell.ttl.tick(time.delta());

        //despawn if the timer's finished
        if spell.ttl.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        //otherwise, move the spell
        /*
        spell_transform.translation += spell.direction * time.delta_seconds() * spell.speed;
        spell.speed += spell.acc * time.delta_seconds();
        */
    }
}