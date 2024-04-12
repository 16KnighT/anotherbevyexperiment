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
                let centre = radius - self.local_points[0];

                return centre + (d * self.local_points[0][0]);
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
    pub fn sphere_from_radius(radius: f32) -> Self {
        return Self {
            shape: Shapes::Sphere,
            local_points: vec![Vec3::new(radius, 0.0, 0.0)],
            transformed_points: vec![Vec3::new(radius, 0.0, 0.0)],
        }
    }
    pub fn poly_from_points(points: Vec<Vec3>) -> Self {
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
            if p.dot(d) <= 0.0 {
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
        return line_case(simplex, d);
    } else if simplex.len() == 3 {
        println!("triangle case");
        return triangle_case(simplex, d);
    }
    println!("tetrahedron case");
    return tetrahedron_case(&mut simplex, d); 
}

/*
notation
p => point
e.g. pa referes to point a
two points => vector
e.g. ao is the vector from a to the origin, ab is a to b, etc...
 */
fn line_case(
    simplex: &Vec<Vec3>,
    direction: &mut Vec3,
) -> bool {
    let (pb, pa) = (simplex[0], simplex[1]);
    let (ab, ao) = (pb - pa, -pa);
    *direction = ab.cross(ao.cross(ab));
    return false;
}

fn triangle_case(
    simplex: &mut Vec<Vec3>,
    direction: &mut Vec3,
) -> bool {
    //i think i can get rid of the variable ao
    let (pc, pb, pa) = (simplex[0], simplex[1], simplex[2]);
    let (ac, ab, ao) = (pc-pa, pb-pa, -pa);
    let abc = ab.cross(ac);
    
    if abc.cross(ac).dot(ao) > 0.0 {
        if ac.dot(ao) > 0.0 {
            simplex.remove(1); //removes b
            *direction = ac.cross(ao).cross(ac).normalize();
            return false;
        } else {
            simplex.remove(0);
            return line_case(simplex, direction);
        }
    } else {
        if ab.cross(abc).dot(ao) > 0.0 {
            simplex.remove(0);
            return line_case(simplex, direction);
        } else {
            if abc.dot(ao) > 0.0 {
                *direction = abc;
            } else {
                simplex.swap(0,1);
                *direction = -abc;
            }
        }
    }

    return false;
    
}

fn tetrahedron_case(
    simplex: &mut Vec<Vec3>,
    direction: &mut Vec3,
) -> bool {
    let (pd, pc, pb, pa) = (simplex[0], simplex[1], simplex[2], simplex[3]);
    //region abc
    let (ab, ac, ad, ao) = (pb - pa, pc - pa, pd - pa, -pa);

    let (abc, acd, adb) = (ab.cross(ac), ac.cross(ad), ad.cross(ab));

    if abc.dot(ac) > 0.0 {
        simplex.remove(0);
        return triangle_case(simplex, direction)
    }

    if acd.dot(ao) > 0.0 {
        simplex.remove(2);
        return triangle_case(simplex, direction)
    }

    if adb.dot(ac) > 0.0 {
        simplex.remove(1);
        return triangle_case(simplex, direction)
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
        ));
            //.add_systems(Startup, col_test_case);
    }
}

#[cfg(test)]
mod tests {
    use bevy::render::render_resource::encase::rts_array::Length;

    use super::*;

    const POINTS: &[Vec3] = &[
            Vec3::new(1.0,1.0,1.0),
            Vec3::new(1.0,1.0,-1.0),
            Vec3::new(1.0,-1.0,1.0),
            Vec3::new(1.0,-1.0,-1.0),
            Vec3::new(-1.0,1.0,1.0),
            Vec3::new(-1.0,1.0,-1.0),
            Vec3::new(-1.0,-1.0,1.0),
            Vec3::new(-1.0,-1.0,-1.0),
        ];

    fn tranform_helper_function(points: &Vec<Vec3>, translation: Vec3) -> Vec<Vec3> {
        let mut translated_points: Vec<Vec3> = Vec::new();
        for point in 0..points.length() {
            translated_points.push(points[point] + translation);
        }
        return translated_points;
    }

    #[test]
    fn support_test_when_local() { //tests if the support function returns the correct point
        let points = POINTS.to_vec();
        let cube = Collider::poly_from_points(points);
        let sphere = Collider::sphere_from_radius(3.0);

        //direction pointing stright up so the returned point should be the origin plus the radius in the y axis
        let mut d = Vec3::new(0.0, 1.0, 0.0).normalize();
        assert_eq!(sphere.support(d), Vec3::new(0.0, 3.0, 0.0), "support returned {}", sphere.support(d));

        //same test but now in the y axis
        d = Vec3::new(0.0, 0.0, 0.1).normalize();
        assert_eq!(sphere.support(d), Vec3::new(0.0, 0.0, 3.0), "support returned {}", sphere.support(d));

        //d is pointing the the corner of the cube
        d = Vec3::new(1.0, 1.0, 1.0).normalize();
        assert_eq!(cube.support(d), Vec3::ONE, "support returned {}", cube.support(d));

        //d is pointing to another corner
        d = Vec3::new(-1.0,1.0,-1.0).normalize();
        assert_eq!(cube.support(d), Vec3::new(-1.0, 1.0,-1.0), "support returned {}", cube.support(d));
       
    }

    #[test]
    fn support_test_when_translated() {
        let points = POINTS.to_vec();

        let translated_points: Vec<Vec3> = tranform_helper_function(&points, Vec3::new(100.0, 234.5, -63.0));

        let cube = Collider {
            shape: Shapes::Polyhedron,
            local_points: points,
            transformed_points: translated_points,
        };
        let sphere = Collider {
            shape: Shapes::Sphere,
            local_points: vec![Vec3::new(3.0, 0.0, 0.0)],
            transformed_points: vec![Vec3::new(212.0, -12.2, 17.0)],
        };

        //for reference the cube's origin is (100, 234.5, -63) and the sphere's origin is (209, -12.2, 17)

        //when d is pointing straight up the returned value should be the vector to it's origin plus it's radius in the direction of the y axis
        let mut d = Vec3::new(0.0, 1.0, 0.0).normalize();
        assert_eq!(sphere.support(d), Vec3::new(209.0, -9.2, 17.0), "support returned {}", sphere.support(d));
         
        d = Vec3::new(0.0, 0.0, 1.0).normalize();
        assert_eq!(sphere.support(d), Vec3::new(209.0, -12.2, 20.0), "support returned {}", sphere.support(d));
        
        d = Vec3::new(1.0, 1.0, 1.0).normalize();
        assert_eq!(cube.support(d), Vec3::new(101.0, 235.5, -62.0), "support returned {}", cube.support(d));

        d = Vec3::new(-1.0,1.0,-1.0).normalize();
        assert_eq!(cube.support(d), Vec3::new(99.0, 235.5, -64.0), "support returned {}", cube.support(d));
    }

    #[test]
    fn cube_intersect_cube() {
        let points = POINTS.to_vec();

        let translated_points = tranform_helper_function(&points, Vec3::new(1.5, 1.5, 1.5));
        let extra_points = tranform_helper_function(&points, Vec3::new(0.0, 0.0, 0.0));

        let cube1 = Collider {
            shape: Shapes::Polyhedron,
            local_points: points,
            transformed_points: translated_points,
        };

        let cube2 = Collider::poly_from_points(extra_points);

        assert!(gjk(&cube1, &cube2));
    }

    #[test]
    fn cube_intersect_sphere() {
        let points = POINTS.to_vec();
        let translated_points = tranform_helper_function(&points, Vec3::new(0.0, 2.5, 0.0));
        let cube = Collider {
            shape: Shapes::Sphere,
            local_points: points,
            transformed_points: translated_points,
        };
        let sphere = Collider::sphere_from_radius(2.0);

        assert!(gjk(&cube, &sphere));
    }

    #[test]
    fn sphere_intersect_sphere() {
        
    }

    #[test]
    fn close_but_no_intersection() {
        
    }
}