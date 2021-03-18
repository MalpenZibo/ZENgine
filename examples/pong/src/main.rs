extern crate zengine;

use zengine::core::system::*;
use zengine::core::*;
use zengine::event::*;
use zengine::graphics::camera::Camera;
use zengine::graphics::camera::{ActiveCamera, CameraMode};
use zengine::graphics::color::Color;
use zengine::graphics::texture::{SpriteDescriptor, SpriteType, TextureManager};
use zengine::log::{info, trace, LevelFilter};
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

fn main() {
    Engine::init_logger(LevelFilter::Info);

    let content = "
        axis_mappings: 
            X:
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
            Y:
              - source:
                  Keyboard:
                      key: W
                scale: 1.0
              - source:
                  Keyboard:
                      key: S
                scale: -1.0
              - source:
                  ControllerStick:
                      device_id: 1
                      which: Left
                      axis: Y
                scale: -1.0
        action_mappings:
            Jump:
            - source: 
                Keyboard: 
                    key: Space
    ";

    let bindings: Bindings<UserInput> = serde_yaml::from_str(&content).unwrap();

    Engine::default()
        .with_system(PlatformSystem::default())
        .with_system(InputSystem::<UserInput>::new(bindings))
        .with_system(CollisionSystem::default())
        .with_system(System1::default())
        .with_system(System2::default())
        .with_system(RenderSystem::<Sprites>::new(
            WindowSpecs::default(),
            CollisionTrace::Inactive,
        ))
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(60)))
        .run(Game {
            execution_numer: 10,
        });
}

#[derive(Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum UserInput {
    X,
    Y,
    Jump,
}
impl InputType for UserInput {}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Sprites {
    Duck,
    DuckFromSheet,
    LogoFromSheet,
}
impl SpriteType for Sprites {}

pub struct Game {
    execution_numer: u32,
}

#[derive(Debug, Component, Default)]
pub struct Player1 {}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        trace!("Game scene on start");

        {
            let mut textures = store.get_resource_mut::<TextureManager<Sprites>>().unwrap();

            textures
                .create("duck.png")
                .with_sprite(
                    Sprites::Duck,
                    SpriteDescriptor {
                        width: 900,
                        height: 1160,
                        x: 0,
                        y: 0,
                    },
                )
                .load();

            textures
                .create("sheet.png")
                .with_sprite(
                    Sprites::DuckFromSheet,
                    SpriteDescriptor {
                        width: 170,
                        height: 200,
                        x: 0,
                        y: 0,
                    },
                )
                .with_sprite(
                    Sprites::LogoFromSheet,
                    SpriteDescriptor {
                        width: 128,
                        height: 128,
                        x: 170,
                        y: 0,
                    },
                )
                .load();
        }
        store.insert_resource(Background {
            color: Color::black(),
        });

        let camera1 = store
            .build_entity()
            .with(Camera {
                width: 800,
                height: 600,
                mode: CameraMode::Mode2D,
            })
            .with(Transform::new(
                Vector3::new(200.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .build();

        let camera2 = store
            .build_entity()
            .with(Camera {
                width: 800,
                height: 600,
                mode: CameraMode::Mode2D,
            })
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.0,
            ))
            .build();

        store.insert_resource(ActiveCamera { entity: camera1 });
        store.insert_resource(ActiveCamera { entity: camera2 });

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 240.0,
                height: 240.0,
                origin: Vector3::zero(),
                color: Color::white(),
                sprite_type: Sprites::DuckFromSheet,
            })
            .with(Transform::new(
                Vector3::new(200.0, 80.0, 0.0),
                Vector3::zero(),
                1.0,
            ))
            // .with(Shape2D {
            //     origin: Vector3::zero(),
            //     shape_type: ShapeType::Rectangle {
            //         width: 240.0,
            //         height: 240.0,
            //     },
            // })
            .with(Shape2D {
                origin: Vector3::new(0.0, 0.0, 0.0),
                shape_type: ShapeType::Circle { radius: 120.0 },
            })
            .with(Player1 {})
            .build();

        store
            .build_entity()
            .with(Sprite {
                width: 50.0,
                height: 50.0,
                origin: Vector3::new(0.5, 0.5, 0.0),
                color: Color::white(),
                sprite_type: Sprites::LogoFromSheet,
            })
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                1.5,
            ))
            .with(Shape2D {
                origin: Vector3::new(0.5, 0.5, 0.0),
                shape_type: ShapeType::Rectangle {
                    width: 50.0,
                    height: 50.0,
                },
            })
            // .with(Shape2D {
            //     origin: Vector3::new(0.5, 0.5, 0.0),
            //     shape_type: ShapeType::Circle { radius: 25.0 },
            // })
            .build();
        store.build_entity().with(Component1 { data: 3 }).build();
        store
            .build_entity()
            .with(Component1 { data: 3 })
            .with(Component2 { data2: 5 })
            .build();
    }

    fn on_stop(&mut self, _store: &mut Store) {
        trace!("Game scene on stop");
    }

    fn update(&mut self, _store: &Store) -> Trans {
        match self.execution_numer {
            0 => Trans::None,
            _ => {
                self.execution_numer -= 1;
                Trans::None
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct Component1 {
    data: u32,
}

#[derive(Component, Debug)]
pub struct Component2 {
    data2: u32,
}

#[derive(Component, Debug)]
pub struct Component3 {
    data2: u32,
}

#[derive(Debug, Default)]
pub struct System1 {
    run_count: u32,
    collision_token: Option<SubscriptionToken>,
}

type System1Data<'a> = (
    WriteSet<'a, Transform>,
    ReadSet<'a, Player1>,
    Read<'a, InputHandler<UserInput>>,
    Read<'a, EventStream<Collision>>,
);

impl<'a> System<'a> for System1 {
    type Data = System1Data<'a>;

    fn init(&mut self, _store: &mut Store) {
        trace!("setup system 1");

        let mut collisions = _store.get_resource_mut::<EventStream<Collision>>().unwrap();
        self.collision_token = Some(collisions.subscribe());
    }

    fn run(&mut self, (mut transform, player1, input, collisions): Self::Data) {
        //trace!("run {} system 1", self.run_count);

        if let Some(token) = self.collision_token {
            for c in collisions.read(&token) {
                info!("collision {:?}", c);
            }
        }

        for t in transform.iter_mut() {
            if player1.get(t.0).is_some() {
                t.1.position.x += input.axis_value(UserInput::X);
                t.1.position.y += input.axis_value(UserInput::Y);
            }
        }

        //info!("user x input: {:?}", input.axis_value(UserInput::X));
        //info!("user jump input: {:?}", input.action_value(UserInput::Jump));

        self.run_count += 1;
    }

    fn dispose(&mut self, _store: &mut Store) {
        trace!("dispose system 1");
    }
}

#[derive(Debug, Default)]
pub struct System2 {
    run_count: u32,
    token: Option<SubscriptionToken>,
}

impl<'a> System<'a> for System2 {
    type Data = (WriteSet<'a, Component2>, Read<'a, EventStream<Trans>>);

    fn init(&mut self, store: &mut Store) {
        trace!("setup system 2");

        self.token = Some(
            store
                .get_resource_mut::<EventStream<Trans>>()
                .unwrap()
                .subscribe(),
        );
    }

    fn run(&mut self, (set, _stream): Self::Data) {
        trace!("run {} system 2", self.run_count);

        trace!("data 2: {:?}", set);
        //trace!("data 2: {:?}", stream.read(&self.token.unwrap()));

        self.run_count += 1;
    }

    fn dispose(&mut self, _store: &mut Store) {
        trace!("dispose system 2");
    }
}
