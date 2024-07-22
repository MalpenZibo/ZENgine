use glam::{Mat4, Vec3};
use rustc_hash::FxHashSet;
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{EventPublisher, Local},
    Entity,
};
use zengine_macro::Component;

/// types of Shape
#[derive(Debug)]
pub enum ShapeType {
    /// A circle with the given radius
    Circle { radius: f32 },
    /// A rectangle with the given width and height
    Rectangle { width: f32, height: f32 },
}

/// [Component](zengine_ecs::Component) that rappresent a 2D collision shape
///
/// A collision shape has an origin and a type
/// that could be a [circle](ShapeType::Circle) or a [rectagle](ShapeType::Rectangle)
#[derive(Component, Debug)]
pub struct Shape2D {
    pub origin: Vec3,
    pub shape_type: ShapeType,
}

/// An event that rappresent a collision between two entities
/// with a [Shape2D]
#[derive(Debug)]
pub struct Collision {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

fn check_rectangle_and_circle(
    (a_width, a_height, a_position, a_origin): (&f32, &f32, &Vec3, Vec3),
    (b_radius, b_position, b_origin): (&f32, &Vec3, Vec3),
) -> bool {
    let half_width = *a_width / 2.0;
    let half_height = *a_height / 2.0;

    let a_origin =
        Mat4::from_translation(a_origin * Vec3::new(half_width, half_height, 0.0)).inverse();
    let a_position = a_origin.mul_vec4(a_position.extend(1.0));

    let b_origin = Mat4::from_translation(b_origin * Vec3::splat(*b_radius)).inverse();
    let b_position = b_origin.mul_vec4(b_position.extend(1.0));

    let delta_x = b_position.x
        - f32::max(
            a_position.x - half_width,
            f32::min(b_position.x, a_position.x + half_width),
        );
    let delta_y = b_position.y
        - f32::max(
            a_position.y - half_height,
            f32::min(b_position.y, a_position.y + half_height),
        );
    delta_x * delta_x + delta_y * delta_y < (b_radius * b_radius)
}

/// A simple collision system between [Shape2D]
///
/// This system doesn't take in consideration the entity transform
/// rotation for rectangular shape
pub(crate) fn collision_system(
    query: Query<(Entity, &Shape2D, &Transform)>,
    mut collision_event: EventPublisher<Collision>,
    already_collided: Local<FxHashSet<(Entity, Entity)>>,
) {
    already_collided.clear();
    for (a_entity, a_shape, a_transform) in query.iter() {
        for (b_entity, b_shape, b_transform) in query.iter().filter(|e| e.0 != a_entity) {
            if match (&a_shape.shape_type, &b_shape.shape_type) {
                (
                    ShapeType::Circle { radius: a_radius },
                    ShapeType::Circle { radius: b_radius },
                ) => {
                    let a_origin =
                        Mat4::from_translation(a_shape.origin * Vec3::splat(*a_radius)).inverse();
                    let b_origin =
                        Mat4::from_translation(b_shape.origin * Vec3::splat(*b_radius)).inverse();
                    let a_pos = a_origin.mul_vec4(a_transform.position.extend(1.0));
                    let b_pos = b_origin.mul_vec4(b_transform.position.extend(1.0));

                    let distance = (a_pos).distance(b_pos).abs();

                    let radius_lenghts =
                        a_radius * a_transform.scale + b_radius * b_transform.scale;
                    distance < radius_lenghts
                }
                (
                    ShapeType::Circle { radius: a_radius },
                    ShapeType::Rectangle {
                        width: b_width,
                        height: b_height,
                    },
                ) => check_rectangle_and_circle(
                    (
                        &(b_width * b_transform.scale),
                        &(b_height * b_transform.scale),
                        &b_transform.position,
                        b_shape.origin,
                    ),
                    (
                        &(a_radius * a_transform.scale),
                        &a_transform.position,
                        a_shape.origin,
                    ),
                ),
                (
                    ShapeType::Rectangle {
                        width: a_width,
                        height: a_height,
                    },
                    ShapeType::Circle { radius: b_radius },
                ) => check_rectangle_and_circle(
                    (
                        &(a_width * a_transform.scale),
                        &(a_height * a_transform.scale),
                        &a_transform.position,
                        a_shape.origin,
                    ),
                    (
                        &(b_radius * b_transform.scale),
                        &b_transform.position,
                        b_shape.origin,
                    ),
                ),
                (
                    ShapeType::Rectangle {
                        width: a_width,
                        height: a_height,
                    },
                    ShapeType::Rectangle {
                        width: b_width,
                        height: b_height,
                    },
                ) => {
                    let a_half_width = *a_width * a_transform.scale / 2.0;
                    let a_half_height = *a_height * a_transform.scale / 2.0;

                    let a_origin = Mat4::from_translation(
                        a_shape.origin * Vec3::new(a_half_width, a_half_height, 0.0),
                    )
                    .inverse();
                    let a_position = a_origin.mul_vec4(a_transform.position.extend(1.0));

                    let b_half_width = *b_width / 2.0;
                    let b_half_height = *b_height / 2.0;

                    let b_origin = Mat4::from_translation(
                        b_shape.origin * Vec3::new(b_half_width, b_half_height, 0.0),
                    )
                    .inverse();
                    let b_position = b_origin.mul_vec4(b_transform.position.extend(1.0));

                    let x = a_position.x - a_half_width;
                    let y = a_position.y - a_half_height;

                    let extent_x = a_position.x + a_half_width;
                    let extent_y = a_position.y + a_half_height;

                    let point_in_shape = |point: Vec3| {
                        point.x > x && point.x < extent_x && point.y > y && point.y < extent_y
                    };

                    point_in_shape(Vec3::new(
                        b_position.x - b_half_width,
                        b_position.y - b_half_height,
                        0.0,
                    )) || point_in_shape(Vec3::new(
                        b_position.x - b_half_width,
                        b_position.y + b_half_height,
                        0.0,
                    )) || point_in_shape(Vec3::new(
                        b_position.x + b_half_width,
                        b_position.y - b_half_height,
                        0.0,
                    )) || point_in_shape(Vec3::new(
                        b_position.x + b_half_width,
                        b_position.y + b_half_height,
                        0.0,
                    ))
                }
            } && !already_collided.contains(&(*b_entity, *a_entity))
            {
                collision_event.publish(Collision {
                    entity_a: *a_entity,
                    entity_b: *b_entity,
                });
                already_collided.insert((*a_entity, *b_entity));
            }
        }
    }
}

fn point_in_rectangle(
    point: Vec3,
    shape_width: f32,
    shape_height: f32,
    shape_origin: Vec3,
    shape_transform: &Transform,
) -> bool {
    let left = shape_width * shape_origin.x * shape_transform.scale;
    let right = shape_width * shape_transform.scale - left;
    let bottom = shape_height * shape_origin.y * shape_transform.scale;
    let top = shape_height * shape_transform.scale - bottom;

    let x = shape_transform.position.x - left;
    let y = shape_transform.position.y - bottom;

    let extent_x = shape_transform.position.x + right;
    let extent_y = shape_transform.position.y + top;

    point.x > x && point.x < extent_x && point.y > y && point.y < extent_y
}

/// Check if a point is inside a target shape
pub fn point_in_shape(point: Vec3, shape: &Shape2D, shape_transform: &Transform) -> bool {
    match shape.shape_type {
        ShapeType::Rectangle { width, height } => {
            point_in_rectangle(point, width, height, shape.origin, shape_transform)
        }
        ShapeType::Circle { radius } => {
            let diameter = radius * 2.0 * shape_transform.scale;
            let delta = Vec3::new(
                diameter * -(-0.5 + shape.origin.x),
                diameter * -(-0.5 + shape.origin.y),
                0.0,
            );

            let distance = (shape_transform.position + delta).distance(point).abs();

            let radius_lenghts = radius * shape_transform.scale;
            distance < radius_lenghts
        }
    }
}
