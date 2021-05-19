extern crate zengine;

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
use zengine::SpriteType;

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

#[derive(Debug, Component, Default)]
pub struct Player1 {}

#[derive(Debug, Component, Default)]
pub struct AI {}

fn main() {
    Engine::init_logger(LevelFilter::Info);

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
        .with_system(RenderSystem::<Sprites>::new(
            WindowSpecs::new("PONG".to_owned(), 600, 800, false),
            CollisionTrace::Active,
        ))
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(60)))
        .run(Game {});
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

        store.insert_resource(Background {
            color: Color::black(),
        });

        let camera = store
            .build_entity()
            .with(Camera {
                width: 600,
                height: 800,
                mode: CameraMode::Mode2D,
            })
            .with(Transform::new(
                Vector3::new(300.0, 400.0, 50.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .build();

        store.insert_resource(ActiveCamera { entity: camera });

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 600.0,
                height: 800.0,
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

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 150.0,
                height: 30.0,
                origin: Vector3::new(0.0, 0.0, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Pad,
            })
            .with(Transform::new(
                Vector3::new(225.0, 20.0, 1.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.0, 0.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: 150.0,
                    height: 30.0,
                },
            })
            .with(Player1 {})
            .build();

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 150.0,
                height: 30.0,
                origin: Vector3::new(0.0, 0.0, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Pad,
            })
            .with(Transform::new(
                Vector3::new(225.0, 750.0, 1.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.0, 0.0, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: 150.0,
                    height: 30.0,
                },
            })
            .with(AI {})
            .build();

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 25.0,
                height: 25.0,
                origin: Vector3::new(0.5, 0.5, 0.0),
                color: Color::white(),
                sprite_type: Sprites::Ball,
            })
            .with(Transform::new(
                Vector3::new(300.0, 400.0, 2.0),
                Vector3::zero(),
                1.0,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.5, 0.5, 0.0),
                shape_type: ShapeType::Circle {
                    radius: 12.5,
                },
            })
            .build();
    }

    fn on_stop(&mut self, _store: &mut Store) {}

    fn update(&mut self, _store: &Store) -> Trans {
        Trans::None
    }
}

#[derive(Debug, Default)]
pub struct PadMovement {
    collision_token: Option<SubscriptionToken>,
}

type PadMovementData<'a> = (
    WriteSet<'a, Transform>,
    ReadSet<'a, Player1>,
    Read<'a, InputHandler<UserInput>>,
    Read<'a, EventStream<Collision>>,
);

impl<'a> System<'a> for PadMovement {
    type Data = PadMovementData<'a>;

    fn init(&mut self, _store: &mut Store) {
        let mut collisions = _store.get_resource_mut::<EventStream<Collision>>().unwrap();
        self.collision_token = Some(collisions.subscribe());
    }

    fn run(&mut self, (mut transform, player1, input, collisions): Self::Data) {}

    fn dispose(&mut self, _store: &mut Store) {}
}
