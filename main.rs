use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

pub const MAX_TILE_X: u32 = 10;
pub const MAX_TILE_Y: u32 = 10;
pub const TILE_SIZE: f32 = 4.0;
pub const CAMERA_SPEED: f32 = 15.0;

#[derive(Resource)]
pub struct TileMaterials {
    dead_material: Handle<StandardMaterial>,
    alive_material: Handle<StandardMaterial>,
    verdant_material: Handle<StandardMaterial>,
}

pub enum Flora {
    SmallPlant,
    BigPlant,
}

impl FromWorld for TileMaterials {
    fn from_world(world: &mut World) -> TileMaterials {
        let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().unwrap();
        TileMaterials {
            dead_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            alive_material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            verdant_material: materials.add(Color::rgb(0.2, 0.8, 0.2).into()),
        }
    }
}

#[derive(Component, Default)]
pub struct Tile {
    //tilex: u32,
    //tiley: u32,
    green: f32,
    plants: u32,
}

#[derive(Component)]
pub struct Player {
    plant_uses: u32,
}

#[derive(Component)]
pub struct Alive;

#[derive(Event)]
pub struct Plant {
    x: f32,
    z: f32,
}

#[derive(Event)]
pub struct SpawnFlora {
    tile: Entity,
    flora: Flora,
}

#[derive(Event)]
pub struct PlantUseRecharge;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<TileMaterials>()
        .add_event::<Plant>()
        .add_event::<SpawnFlora>()
        .add_event::<PlantUseRecharge>()
        .add_systems(Startup, ui_setup)
        .add_systems(Startup, scene_setup)
        .add_systems(Startup, tile_spawner)
        .add_systems(Update, controller)
        .add_systems(Update, check_plant)
        .add_systems(Update, check_alive_tiles)
        .add_systems(Update, spawn_flora_event)
        .add_systems(Update, check_recharge_event)
        .run();
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
        projection: OrthographicProjection {
            scale: 5.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(-15.0, 15.0, -15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).id();

    let player = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player { plant_uses: 10 },
    )).id();

    commands.entity(player).push_children(&[camera]);
}

fn ui_setup(
    mut commands: Commands,
) {
    commands.spawn(
        TextBundle::from_section(
            "Plants Left: 10",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

pub fn tile_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TileMaterials>,
) {
    let relative_origin = Vec3::new((MAX_TILE_X as f32) * TILE_SIZE/ 2.0, 0.0, (MAX_TILE_Y as f32) * TILE_SIZE / 2.0);
    for x in 0..MAX_TILE_X {
        for y in 0..MAX_TILE_Y {
            let transform = Vec3::new((x as f32) * TILE_SIZE, 0.0, (y as f32) * TILE_SIZE) - relative_origin;

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Plane::from_size(TILE_SIZE).into()),
                    material: materials.dead_material.clone(),
                    transform: Transform::from_translation(transform),
                    ..default()
                },
                Tile::default(),
            ));
        }
    }
}

pub fn controller(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut plant_event: EventWriter<Plant>,
    mut text: Query<&mut Text>,
    time: Res<Time>,
) {
    let (mut player_transform, mut player) = player_query.single_mut();
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

    if keyboard_input.just_pressed(KeyCode::E) && player.plant_uses > 0 {
        player.plant_uses -= 1;

        let mut text = text.single_mut();
        text.sections[0].value = format!("Plants Left: {:?}", player.plant_uses);

        plant_event.send(Plant {x: player_transform.translation.x, z: player_transform.translation.z});
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    player_transform.translation += direction * CAMERA_SPEED * time.delta_seconds();

}

pub fn check_plant (
    mut plant_event_listener: EventReader<Plant>,
    mut tiles_query: Query<(&mut Handle<StandardMaterial>, &Transform, Entity, &mut Tile)>,
    mut commands: Commands,
    mut spawn_flora_event_writer: EventWriter<SpawnFlora>,
    materials: Res<TileMaterials>,

) {
    let halftile = TILE_SIZE / 2.0;

    for event in plant_event_listener.iter() {
        println!("SLAY!");
        'tileloop: for (mut tile_mat, tile_transform, entity_id, mut tile) in tiles_query.iter_mut() {
            let tilepos = tile_transform.translation;

            //if both checks are true then the player is within the bounds of this tile
            if (event.x < tilepos.x + halftile) && (event.x > tilepos.x - halftile) {
                if (event.z < tilepos.z + halftile) && (event.z > tilepos.z - halftile) {
                    //creates grass
                    spawn_flora_event_writer.send(SpawnFlora { tile: entity_id, flora: Flora::SmallPlant });
                    tile.plants += 1;

                    //check to prevent changing the material of a verdant tile
                    if tile.green == 0.0 {
                        *tile_mat = materials.alive_material.clone(); //changes the tile's material to the 'alive' material
                        commands.entity(entity_id).insert(Alive);
                    }
                    println!("{tilepos}");

                    break 'tileloop;
                }
            }
        }
    }
}

pub fn check_alive_tiles (
    mut tiles_query: Query<(&mut Tile, &mut Handle<StandardMaterial>, Entity), With<Alive>>,
    mut spawn_flora_event_writer: EventWriter<SpawnFlora>,
    mut recharge_event_writer: EventWriter<PlantUseRecharge>,
    mut commands: Commands,
    materials: Res<TileMaterials>,
) {
    for (mut tile, mut tile_mat, entity_id) in tiles_query.iter_mut() {
        let add_green = 0.1 * tile.plants as f32;
        tile.green += add_green;
        let debug = add_green;
        println!("{debug}");
        if tile.green > 1000.0 {
            *tile_mat = materials.verdant_material.clone();
            commands.entity(entity_id).remove::<Alive>();
            spawn_flora_event_writer.send(SpawnFlora { tile: entity_id, flora: Flora::BigPlant });
            recharge_event_writer.send(PlantUseRecharge);
        }
    }
}

pub fn spawn_flora_event (
    mut commands: Commands,
    mut spawn_flora_event_listener: EventReader<SpawnFlora>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TileMaterials>,
) {
    for event in spawn_flora_event_listener.iter() {
        match event.flora {
            Flora::BigPlant => {
                let bush = commands.spawn(
                    PbrBundle {
                        material: materials.alive_material.clone(),
                        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                        transform: Transform {
                            translation: Vec3::new((rand::random::<f32>() * TILE_SIZE) - (TILE_SIZE / 2.0), 0.0, (rand::random::<f32>() * TILE_SIZE) - (TILE_SIZE / 2.0)),
                            ..default()
                        },
                        ..default()
                    },
                ).id();
                commands.entity(event.tile).push_children(&[bush]);
            },
            Flora::SmallPlant => {
                let grass = commands.spawn(
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {radius: 0.1, ..default()})),
                        material: materials.alive_material.clone(),
                        transform: Transform {
                            translation: Vec3::new((rand::random::<f32>() * TILE_SIZE) - (TILE_SIZE / 2.0), 0.0, (rand::random::<f32>() * TILE_SIZE) - (TILE_SIZE / 2.0)),
                            ..default()
                        },
                        ..default()
                    }
                ).id();
                commands.entity(event.tile).push_children(&[grass]);
            },
        }
    }
}

pub fn check_recharge_event(
    mut player_query: Query<&mut Player>,
    mut recharge_event_listener: EventReader<PlantUseRecharge>,
    mut text: Query<&mut Text>,
) {
    for event in recharge_event_listener.iter() {
        let mut player = player_query.single_mut();

        let mut text = text.single_mut();

        player.plant_uses += 2;

        text.sections[0].value = format!("Plants Left: {:?}", player.plant_uses);
    }
}