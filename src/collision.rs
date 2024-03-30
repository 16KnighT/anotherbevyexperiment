use bevy::prelude::*;

//origin will be obtained from a Transform query
//#[derive(Component, PartialEq)]
#[derive(Component)]
pub struct Collider {
    shape: Shapes,
    local_points: Vec<Vec3>,
    transformed_points: Vec<Vec3>,
}

//#[derive(PartialEq)]
pub enum Shapes {
    Sphere,
    Polyhedron,
}

impl Collider {
    fn support(&self, d: Vec3) -> Vec3 {
        match self.shape {
            Shapes::Sphere => {
                let radius = self.transformed_points[0];
                return d * radius.length();
            },
            Shapes::Polyhedron => {
                let mut max_dot = std::f32::NEG_INFINITY;
                let mut max_vec = Vec3::ZERO;

                for point in &self.transformed_points {
                    let dotted = point.dot(d);
                    if max_dot < dotted {
                        max_dot = dotted;
                        max_vec = *point;
                    };
                }
                return max_vec;
                //I FINISHED HERE
                /*
                notes:
                should the bounding shape be relative to the translation of the object?
                if not is there a way to link the Transform and the boudning shape?
                I think I will have to do this as the transform will allow me to rotate the shapes and stuff
                 */
            }
        }
    }
    //these constructor functions will ensure that different shape types will be made in specific ways
    fn sphere_from_radius(radius: f32) -> Self {
        return Self {
            shape: Shapes::Sphere,
            local_points: vec![Vec3::new(radius, 0.0, 0.0)],
            transformed_points: vec![Vec3::new(radius, 0.0, 0.0)],
        }
    }
    fn poly_from_points(points: Vec<Vec3>) -> Self {
        return Self {
            shape: Shapes::Polyhedron,
            local_points: points.clone(),
            transformed_points: points.clone(),
        }
    }
} 

//transform should be applied before the support function is called because currently the transformed_points will be wrong for the first frame
pub fn apply_transform_collider (
    mut colliders_and_transforms: Query<(&mut Collider, &Transform)>,
) {
    for (mut col, trans,) in colliders_and_transforms.iter_mut() {
        match col.shape {
            Shapes::Sphere => {
                col.transformed_points[0] = col.local_points[0] + trans.translation;
            },
            Shapes::Polyhedron => {
                for i in 0..col.local_points.len() {
                    col.transformed_points[i] = col.local_points[i] + trans.translation;
                }
            }
        }
    }
}

pub fn collision_update (
    col: Query<&Collider>
) {
    //checks every collider against every other collider
    for [s1, s2] in col.iter_combinations() {
        gjk(s1, s2);
    }
}

pub fn gjk (
    s1: &Collider,
    s2: &Collider,
) {
    support(s1, s2, Vec3::new(1.0,1.0,1.0));
}

fn support (
    s1: &Collider,
    s2: &Collider,
    d: Vec3,
) -> Vec3 {
    //for debug case
    assert_eq!(s1.support(d), Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(s2.support(d), Vec3::new(1.0, 1.0, 1.0));

    return s1.support(d) - s2.support(-d);
}

pub fn col_test_case (
    mut commands: Commands,
) {
    let points = vec![
        Vec3::new(1.0,1.0,1.0),
        Vec3::new(1.0,1.0,-1.0),
        Vec3::new(1.0,-1.0,1.0),
        Vec3::new(1.0,-1.0,-1.0),
        Vec3::new(-1.0,1.0,1.0),
        Vec3::new(-1.0,1.0,-1.0),
        Vec3::new(-1.0,-1.0,1.0),
        Vec3::new(-1.0,-1.0,-1.0),
    ];
    
    //test cube at (0,0,0), length of all sides are 2
    commands.spawn( (
        Collider::poly_from_points(points),
        Transform::from_translation(Vec3::ZERO),
    ));

    //test sphere at (0,0,0), radius of 1
    commands.spawn((
        Collider::sphere_from_radius(1.0),
        Transform::from_translation(Vec3::ZERO),
    ));
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            collision_update,
            apply_transform_collider
                .before(collision_update),
        ))
            .add_systems(Startup, col_test_case);
    }
}