use zengine::core::join::Join;
use zengine::core::system::*;
use zengine::core::*;
use zengine::event::Bindings;
use zengine::event::EventStream;
use zengine::event::InputHandler;
use zengine::event::InputSystem;
use zengine::event::InputType;
use zengine::event::SubscriptionToken;
use zengine::graphics::camera::Camera;
use zengine::graphics::camera::{ActiveCamera, CameraMode};
use zengine::graphics::color::Color;
use zengine::graphics::texture::SpriteDescriptor;
use zengine::graphics::texture::SpriteType;
use zengine::graphics::texture::TextureManager;
use zengine::log::LevelFilter;
use zengine::math::transform::Transform;
use zengine::math::vector2::Vector2;
use zengine::math::vector3::Vector3;
use zengine::physics::collision::Collision;
use zengine::physics::collision::CollisionSystem;
use zengine::physics::collision::{Shape2D, ShapeType};
use zengine::platform::*;
use zengine::render::*;
use zengine::serde::Deserialize;
use zengine::serde_yaml;
use zengine::timing::*;
use zengine::Component;
use zengine::Engine;
use zengine::InputType;
use zengine::Resource;
use zengine::SpriteType;

static WIDTH: f32 = 600.0;
static HEIGHT: f32 = 800.0;
static PAD_HALF_WIDTH: f32 = 75.0;
static PAD_HALF_HEIGHT: f32 = 15.0;
static BALL_RADIUS: f32 = 12.5;
static BALL_VEL: f32 = 150.0;

#[derive(Deserialize, InputType, Hash, Eq, PartialEq, Clone)]
pub enum UserInput {
    Player1XAxis,
}

#[derive(Hash, Eq, SpriteType, PartialEq, Clone, Debug)]
pub enum Sprites {
    Background,
    Pad,
    Ball,
}

#[derive(Debug, Resource)]
pub struct Player1 {
    entity: Entity,
}

#[derive(Debug, Resource)]
pub struct FieldBorder {
    sx: Entity,
    dx: Entity,
    top: Entity,
    bottom: Entity,
}

#[derive(Debug, Component, Default)]
pub struct AI {}

#[derive(Debug, Component, Default, Clone)]
pub struct Pad {
    force: f32,
    cur_acc: f32,
    velocity: f32,
    mass: f32,
}

#[derive(Debug, Component, Default, Clone)]
pub struct Ball {
    vel: Vector2,
}

#[derive(Debug, Default, Resource)]
pub struct GameSettings {
    drag_constant: f32,
}

fn main() {
    Engine::init_logger(LevelFilter::Trace);

    let content = "
        axis_mappings: 
            Player1XAxis:
            - source:
                Keyboard:
                    key: D
              scale: 1.0
            - source:
                Keyboard:
                    key: A
              scale: -1.0
            - source:
                ControllerStick:
                    device_id: 1
                    which: Left
                    axis: X
              scale: 1.0
    ";

    let bindings: Bindings<UserInput> = serde_yaml::from_str(&content).unwrap();

    Engine::default()
        .with_system(PlatformSystem::default())
        .with_system(InputSystem::<UserInput>::new(bindings))
        .with_system(CollisionSystem::default())
        .with_system(PlayerPadControl::default())
        .with_system(AIPadControl::default())
        .with_system(PadMovement::default())
        .with_system(BallMovement::default())
        .with_system(CollisionResponse::default())
        .with_system(RenderSystem::<Sprites>::new(
            WindowSpecs::new("PONG".to_owned(), 600, 800, false),
            CollisionTrace::Inactive,
        ))
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(60)))
        .run(Game {});
}

fn initial_ball_movement() -> Vector2 {
    let angle =
        (fastrand::i32(45..135) as f32 + if fastrand::bool() { 180.0 } else { 0.0 }).to_radians();
    Vector2::new(BALL_VEL * angle.cos(), BALL_VEL * angle.sin())
}

pub struct Game {}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        {
            let mut textures = store.get_resource_mut::<TextureManager<Sprites>>().unwrap();

            textures
                .create("bg.png")
                .with_sprite(
                    Sprites::Background,
                    SpriteDescriptor {
                        width: 600,
                        height: 800,
                        x: 0,
                        y: 0,
                    },
                )
                .load();
            textures
                .create("pad.png")
                .with_sprite(
                    Sprites::Pad,
                    SpriteDescriptor {
                        width: 150,
                        height: 30,
                        x: 0,
                        y: 0,
                    },
                )
                .load();
            textures
                .create("ball.png")
                .with_sprite(
                    Sprites::Ball,
                    SpriteDescriptor {
                        width: 25,
                        height: 25,
                        x: 0,
                        y: 0,
                    },
                )
                .load();
        }

        store.insert_resource(GameSettings {
            drag_constant: 10.0,
        });

        store.insert_resource(Background {
            color: Color::black(),
        });

        let pad = Pad {
            force: 1000.0,
            mass: 5.0,
            cur_acc: 0.0,
            velocity: 0.0,
        };

        let camera = store
            .build_entity()
            .with(Camera {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                mode: CameraMode::Mode2D,
            })
            .with(Transform::new(
                Vector3::new(WIDTH / 2.0, HEIGHT / 2.0, 50.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .build();

        store.insert_resource(ActiveCamera { entity: camera });

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: WIDTH,
                height: HEIGHT,
                origin: Vector3::new(0.0, 0.0, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Background,
            })
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .build();

        let sx = store
            .build_entity()
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(1.0, 0.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: 300.0,
                    height: HEIGHT,
                },
            })
            .build();
        let dx = store
            .build_entity()
            .with(Transform::new(
                Vector3::new(WIDTH, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.0, 0.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: 300.0,
                    height: HEIGHT,
                },
            })
            .build();

        let top = store
            .build_entity()
            .with(Transform::new(
                Vector3::new(0.0, HEIGHT, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.0, 0.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: WIDTH,
                    height: 300.0,
                },
            })
            .build();

        let bottom = store
            .build_entity()
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.0, 1.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: WIDTH,
                    height: 300.0,
                },
            })
            .build();

        store.insert_resource(FieldBorder {
            sx,
            dx,
            top,
            bottom,
        });

        let pad1 = store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: PAD_HALF_WIDTH * 2.0,
                height: PAD_HALF_HEIGHT * 2.0,
                origin: Vector3::new(0.5, 0.5, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Pad,
            })
            .with(Transform::new(
                Vector3::new(WIDTH / 2.0, 0.0 + 20.0 + PAD_HALF_HEIGHT, 1.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.5, 0.5, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: PAD_HALF_WIDTH * 2.0,
                    height: PAD_HALF_HEIGHT * 2.0,
                },
            })
            .with(pad.clone())
            //.with(AI {})
            .build();
        store.insert_resource(Player1 { entity: pad1 });

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: PAD_HALF_WIDTH * 2.0,
                height: PAD_HALF_HEIGHT * 2.0,
                origin: Vector3::new(0.5, 0.5, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Pad,
            })
            .with(Transform::new(
                Vector3::new(WIDTH / 2.0, HEIGHT - 20.0 - PAD_HALF_HEIGHT, 1.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.5, 0.5, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: PAD_HALF_WIDTH * 2.0,
                    height: PAD_HALF_HEIGHT * 2.0,
                },
            })
            .with(pad.clone())
            .with(AI {})
            .build();

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: BALL_RADIUS * 2.0,
                height: BALL_RADIUS * 2.0,
                origin: Vector3::new(0.5, 0.5, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Ball,
            })
            .with(Transform::new(
                Vector3::new(WIDTH / 2.0, HEIGHT / 2.0, 2.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.5, 0.5, 0.0),
                shape_type: ShapeType::Circle {
                    radius: BALL_RADIUS,
                },
            })
            .with(Ball {
                vel: initial_ball_movement(),
            })
            .build();
    }

    fn on_stop(&mut self, _store: &mut Store) {}

    fn update(&mut self, _store: &Store) -> Trans {
        Trans::None
    }
}

#[derive(Debug, Default)]
pub struct PlayerPadControl {
    collision_token: Option<SubscriptionToken>,
}

type PlayerPadControlData<'a> = (
    ReadOption<'a, Player1>,
    WriteSet<'a, Pad>,
    Read<'a, InputHandler<UserInput>>,
);

impl<'a> System<'a> for PlayerPadControl {
    type Data = PlayerPadControlData<'a>;

    fn init(&mut self, _store: &mut Store) {
        let mut collisions = _store.get_resource_mut::<EventStream<Collision>>().unwrap();
        self.collision_token = Some(collisions.subscribe());
    }

    fn run(&mut self, (player1, mut pads, input): Self::Data) {
        player1
            .and_then(|player1| pads.get_mut(&player1.entity))
            .map(|pad| {
                pad.cur_acc = input.axis_value(UserInput::Player1XAxis) * pad.force / pad.mass;
            });
    }

    fn dispose(&mut self, _store: &mut Store) {}
}

#[derive(Debug, Default)]
pub struct PadMovement {}

type PadMovementData<'a> = (
    WriteSet<'a, Transform>,
    WriteSet<'a, Pad>,
    Read<'a, GameSettings>,
    Read<'a, Time>,
);

impl<'a> System<'a> for PadMovement {
    type Data = PadMovementData<'a>;

    fn init(&mut self, _store: &mut Store) {}

    fn run(&mut self, (transforms, mut pads, game_settings, time): Self::Data) {
        for (_, pad, transform) in pads.join_mut(transforms) {
            let drag_acc = -game_settings.drag_constant * pad.velocity / pad.mass;
            pad.velocity +=
                pad.cur_acc * time.delta.as_secs_f32() + drag_acc * time.delta.as_secs_f32();
            transform.position.x += pad.velocity * time.delta.as_secs_f32();
        }
    }

    fn dispose(&mut self, _store: &mut Store) {}
}

#[derive(Debug, Default)]
pub struct BallMovement {}

type BallMovementData<'a> = (WriteSet<'a, Transform>, WriteSet<'a, Ball>, Read<'a, Time>);

impl<'a> System<'a> for BallMovement {
    type Data = BallMovementData<'a>;

    fn init(&mut self, _store: &mut Store) {}

    fn run(&mut self, (transforms, mut balls, time): Self::Data) {
        for (_, ball, transform) in balls.join_mut(transforms) {
            transform.position.x += ball.vel.x * time.delta.as_secs_f32();
            transform.position.y += ball.vel.y * time.delta.as_secs_f32();
        }
    }

    fn dispose(&mut self, _store: &mut Store) {}
}

#[derive(Debug, Default)]
pub struct CollisionResponse {
    collisiion_token: Option<SubscriptionToken>,
}

type CollisionResponseData<'a> = (
    WriteSet<'a, Transform>,
    WriteSet<'a, Pad>,
    WriteSet<'a, Ball>,
    Read<'a, EventStream<Collision>>,
    ReadOption<'a, FieldBorder>,
);

enum CollisionType {
    PadBorder { pad: Entity, border: Side },
    BallBorder { ball: Entity, border: Side },
    BallPad { pad: Entity, ball: Entity },
}

enum Side {
    SX(Entity),
    DX(Entity),
    BOTTOM(Entity),
    TOP(Entity),
}

impl CollisionResponse {
    fn get_collision_type(
        &self,
        collision: &Collision,
        pads: &WriteSet<Pad>,
        balls: &WriteSet<Ball>,
        field_border: &FieldBorder,
    ) -> Option<CollisionType> {
        let get_field_border = |entity: Entity| -> Option<Side> {
            if field_border.sx == entity {
                return Some(Side::SX(entity));
            } else if field_border.dx == entity {
                return Some(Side::DX(entity));
            } else if field_border.bottom == entity {
                return Some(Side::BOTTOM(entity));
            } else if field_border.top == entity {
                return Some(Side::TOP(entity));
            }

            return None;
        };

        if pads.contains_key(&collision.entity_a) {
            if let Some(border) = get_field_border(collision.entity_b) {
                return Some(CollisionType::PadBorder {
                    pad: collision.entity_a,
                    border,
                });
            } else if balls.contains_key(&collision.entity_b) {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_a,
                    ball: collision.entity_b,
                });
            }
        } else if pads.contains_key(&collision.entity_b) {
            if let Some(border) = get_field_border(collision.entity_a) {
                return Some(CollisionType::PadBorder {
                    pad: collision.entity_b,
                    border,
                });
            } else if balls.contains_key(&collision.entity_a) {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_b,
                    ball: collision.entity_a,
                });
            }
        } else if balls.contains_key(&collision.entity_a) {
            if let Some(border) = get_field_border(collision.entity_b) {
                return Some(CollisionType::BallBorder {
                    ball: collision.entity_a,
                    border,
                });
            } else if pads.contains_key(&collision.entity_b) {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_b,
                    ball: collision.entity_a,
                });
            }
        } else if balls.contains_key(&collision.entity_b) {
            if let Some(border) = get_field_border(collision.entity_a) {
                return Some(CollisionType::BallBorder {
                    ball: collision.entity_b,
                    border,
                });
            } else if pads.contains_key(&collision.entity_a) {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_a,
                    ball: collision.entity_b,
                });
            }
        }

        return None;
    }
}

impl<'a> System<'a> for CollisionResponse {
    type Data = CollisionResponseData<'a>;

    fn init(&mut self, store: &mut Store) {
        self.collisiion_token = store
            .get_resource_mut::<EventStream<Collision>>()
            .map(|mut collision_stream| collision_stream.subscribe());
    }

    fn run(&mut self, (mut transforms, mut pads, mut balls, collisions, field_border): Self::Data) {
        if let Some(token) = self.collisiion_token {
            if let Some(field_border) = field_border {
                for c in collisions.read(&token) {
                    match self.get_collision_type(c, &pads, &balls, &field_border) {
                        Some(CollisionType::PadBorder {
                            pad: pad_entity,
                            border: Side::SX(_),
                        }) => {
                            pads.get_mut(&pad_entity).map(|pad| {
                                pad.velocity = 0.0;
                                transforms
                                    .get_mut(&pad_entity)
                                    .map(|transform| transform.position.x = 0.0 + PAD_HALF_WIDTH);
                            });
                        }
                        Some(CollisionType::PadBorder {
                            pad: pad_entity,
                            border: Side::DX(_),
                        }) => {
                            pads.get_mut(&pad_entity).map(|pad| {
                                pad.velocity = 0.0;
                                transforms
                                    .get_mut(&pad_entity)
                                    .map(|transform| transform.position.x = WIDTH - PAD_HALF_WIDTH);
                            });
                        }
                        Some(CollisionType::BallBorder {
                            ball: ball_entity,
                            border: Side::SX(border_entity),
                        })
                        | Some(CollisionType::BallBorder {
                            ball: ball_entity,
                            border: Side::DX(border_entity),
                        }) => {
                            balls
                                .get_mut(&ball_entity)
                                .map(|ball| ball.vel = Vector2::new(-ball.vel.x, ball.vel.y));
                            transforms.get_mut(&ball_entity).map(|transform| {
                                transform.position.x = if border_entity == field_border.sx {
                                    0.0 + BALL_RADIUS
                                } else {
                                    WIDTH - BALL_RADIUS
                                }
                            });
                        }
                        Some(CollisionType::BallBorder {
                            ball: ball_entity,
                            border: Side::BOTTOM(_),
                        })
                        | Some(CollisionType::BallBorder {
                            ball: ball_entity,
                            border: Side::TOP(_),
                        }) => {
                            transforms.get_mut(&ball_entity).map(|transform| {
                                transform.position.x = WIDTH / 2.0;
                                transform.position.y = HEIGHT / 2.0;
                            });
                            balls
                                .get_mut(&ball_entity)
                                .map(|ball| ball.vel = initial_ball_movement());
                        }
                        Some(CollisionType::BallPad {
                            pad: pad_entity,
                            ball: ball_entity,
                        }) => {
                            if let Some(pad_transform) = transforms
                                .get(&pad_entity)
                                .map(|pad_transform| -> Transform { pad_transform.clone() })
                            {
                                match (
                                    transforms.get_mut(&ball_entity),
                                    balls.get_mut(&ball_entity),
                                ) {
                                    (Some(ball_transform), Some(ball)) => {
                                        ball.vel = Vector2::new(
                                            ball.vel.x
                                                + (if ball_transform.position.x
                                                    < pad_transform.position.x
                                                {
                                                    -1.0
                                                } else {
                                                    1.0
                                                } * ((ball_transform.position.x
                                                    - pad_transform.position.x)
                                                    .abs()
                                                    / PAD_HALF_WIDTH
                                                    * 100.0)),
                                            -ball.vel.y,
                                        );
                                        ball_transform.position.y = pad_transform.position.y
                                            + if ball_transform.position.y
                                                > pad_transform.position.y
                                            {
                                                PAD_HALF_HEIGHT + BALL_RADIUS
                                            } else {
                                                -(PAD_HALF_HEIGHT + BALL_RADIUS)
                                            };
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    };
                }
            }
        }
    }

    fn dispose(&mut self, _store: &mut Store) {}
}

#[derive(Debug, Default)]
pub struct AIPadControl {
    collision_token: Option<SubscriptionToken>,
}

type AIPadControlData<'a> = (
    ReadSet<'a, AI>,
    WriteSet<'a, Pad>,
    ReadSet<'a, Ball>,
    ReadSet<'a, Transform>,
);

impl<'a> System<'a> for AIPadControl {
    type Data = AIPadControlData<'a>;

    fn init(&mut self, _store: &mut Store) {
        let mut collisions = _store.get_resource_mut::<EventStream<Collision>>().unwrap();
        self.collision_token = Some(collisions.subscribe());
    }

    fn run(&mut self, (ais, pads, balls, transforms): Self::Data) {
        if let Some(ball_transform) = balls
            .join(&transforms)
            .next()
            .map(|(_, _, ball_transform)| ball_transform.clone())
        {
            for (_, _, pad, transform) in ais.join((pads, &transforms)) {
                pad.cur_acc = if ball_transform.position.x > transform.position.x {
                    1.0
                } else {
                    -1.0
                } * pad.force
                    / pad.mass;
            }
        }
    }

    fn dispose(&mut self, _store: &mut Store) {}
}
