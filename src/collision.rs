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
                //println!("support point 1: {}", d * radius.length());
                return d * radius.length();
            },
            Shapes::Polyhedron => {
                let mut max_dot = std::f32::NEG_INFINITY;
                let mut max_vec = Vec3::ZERO;

                //println!("selected direction: {}", d);
                for point in &self.transformed_points {
                    //print!("point {} dotted: ", point);
                    let dotted = point.dot(d);
                    //println!("{}", dotted);
                    if max_dot < dotted {
                        max_dot = dotted;
                        max_vec = *point;
                    };
                }
                //println!("support point 2: {}", max_vec);
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
        let result = gjk(s1, s2);
        println!("collision?: {}", result);
    }
}



/**
 * Returns true if objects have collided otherwise false.
 */
pub fn gjk (
    s1: &Collider,
    s2: &Collider,
) -> bool {
    let mut d = Vec3::ONE.normalize();
    let mut simplex = vec![support(s1, s2, &d)];
    d = Vec3::ZERO - simplex[0];
        loop {
            d = d.normalize();
            println!("selecting new point");
            let p = support(s1, s2, &d);
            if p.dot(d) < 0.0 {
                println!("new point not past origin");
                return false;
            }
            println!("new point is past origin");
            simplex.push(p);
            if handle_simplex(&mut simplex, &mut d) {
                return true;
            }
            println!("the origin is not yet surrounded");
        }
}

fn handle_simplex(
    mut simplex: &mut Vec<Vec3>,
    d: &mut Vec3,
) -> bool {
    if simplex.len() == 2 {
        println!("line case");
        *d = line_case(simplex);
        return false
    } else if simplex.len() == 3 {
        println!("triangle case");
        *d = triangle_case(simplex);
        return false;
    }
    println!("tetrahedron case");
    return tetrahedron_case(&mut simplex); 
}

/*
notation
p => point
e.g. pa referes to point a
two points => vector
e.g. ao is the vector from a to the origin, ab is a to b, etc...
 */
fn line_case(
    simplex: &Vec<Vec3>
) -> Vec3 {
    let (pb, pa) = (simplex[0], simplex[1]);
    let (ab, ao) = (pb - pa, -pa);
    return ab.cross(ao.cross(ab)).normalize();
}

fn triangle_case(
    simplex: &Vec<Vec3>
) -> Vec3 {
    //i think i can get rid of the variable ao
    let (pc, pb, pa) = (simplex[0], simplex[1], simplex[2]);
    let (ac, ab, ao) = (pc-pa, pb-pa, -pa);
    //I believe this should be the normal to the triangle
    return ab.cross(ac).normalize();
}

fn tetrahedron_case(
    simplex: &mut Vec<Vec3>
) -> bool {
    let (pd, pc, pb, pa) = (simplex[0], simplex[1], simplex[2], simplex[3]);
    //region abc
    let (ab, ac, ad, ao) = (pb - pa, pc - pa, pd - pa, -pa);
    if (ao.dot(ab.cross(ac))) > 0.0 {
        //the origin is past region abc therefore we need to remove point d
        println!("removing point d");
        simplex.remove(0);
        return false
    }
    //region acd
    if (ao.dot(ac.cross(ad))) > 0.0 {
        //the origin is past the region acd therefore we need to get rid of point b
        println!("removing point b");
        simplex.remove(2);
        return false;
    }
    //region adb
    if (ao.dot(ab.cross(ad))) > 0.0 {
        //the origin is past the region adb therefore we need to get rid of point c
        println!("removing point c");
        simplex.remove(1);
        return false;
    }

    return true;
}

fn support (
    s1: &Collider,
    s2: &Collider,
    d: &Vec3,
) -> Vec3 {
    return s1.support(*d) - s2.support(-(*d));
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
        Transform::from_translation(Vec3::new(2.0, 2.0, 2.0)),
    ));

    //test sphere at (0,0,0), radius of 1
    commands.spawn((
        Collider::sphere_from_radius(2.0),
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