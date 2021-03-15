use crate::core::join::Join;
use crate::core::system::ReadSet;
use crate::core::system::Write;
use crate::core::Component;
use crate::core::Entity;
use crate::core::System;
use crate::event::EventStream;
use crate::math::transform::Transform;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;

#[derive(Debug)]
pub enum ShapeType {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

#[derive(Component, Debug)]
pub struct Shape2D {
    pub origin: Vector3,
    pub shape_type: ShapeType,
}

pub struct Collision {
    entity_a: Entity,
    entity_b: Entity,
}

pub struct CollisionSystem {}

impl<'a> System<'a> for CollisionSystem {
    type Data = (
        ReadSet<'a, Shape2D>,
        ReadSet<'a, Transform>,
        Write<'a, EventStream<Collision>>,
    );

    fn run(&mut self, (shapes, transforms, mut collisions): Self::Data) {
        for a in shapes.join(&transforms) {
            for b in shapes.join(&transforms).filter(|e| e.0 != a.0) {
                match (&a.1.shape_type, &b.1.shape_type) {
                    (
                        ShapeType::Circle { radius: a_radius },
                        ShapeType::Circle { radius: b_radius },
                    ) => {
                        let distance = a.2.position.distance(&a.2.position).abs();
                        let radius_lenghts = a_radius + b_radius;
                        if distance <= radius_lenghts {
                            collisions.publish(Collision {
                                entity_a: a.0.clone(),
                                entity_b: b.0.clone(),
                            });
                        }
                    }
                    (
                        ShapeType::Circle { radius: a_radius },
                        ShapeType::Rectangle {
                            width: b_width,
                            height: b_height,
                        },
                    ) => {}
                    (
                        ShapeType::Rectangle {
                            width: a_width,
                            height: a_height,
                        },
                        ShapeType::Circle { radius: b_radius },
                    ) => {}
                    (
                        ShapeType::Rectangle {
                            width: a_width,
                            height: a_height,
                        },
                        ShapeType::Rectangle {
                            width: b_width,
                            height: b_height,
                        },
                    ) => {}
                }
            }
        }
    }
}
