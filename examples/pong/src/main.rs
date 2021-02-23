extern crate zengine;

use std::collections::HashMap;
use zengine::core::system::Read;
use zengine::core::system::ReadEntities;
use zengine::core::system::ReadSet;
use zengine::core::system::WriteSet;
use zengine::core::timing::{FrameLimiter, TimingSystem};
use zengine::core::Component;
use zengine::core::Scene;
use zengine::core::Store;
use zengine::core::System;
use zengine::core::Trans;
use zengine::device::controller::{ControllerButton, Which};
use zengine::device::keyboard::Key;
use zengine::device::mouse::MouseButton;
use zengine::event::input::{Axis, Input};
use zengine::event::input_system::InputSystem;
use zengine::event::Bindings;
use zengine::event::InputHandler;
use zengine::event::InputType;
use zengine::event::{ActionBind, AxisBind};
use zengine::graphics::camera::ActiveCamera;
use zengine::graphics::camera::{Camera, CameraMode};
use zengine::graphics::color::Color;
use zengine::graphics::texture::SpriteDescriptor;
use zengine::graphics::texture::SpriteType;
use zengine::graphics::texture::TextureManager;
use zengine::math::transform::Transform;
use zengine::math::vector3::Vector3;
use zengine::platform::platform_system::PlatformSystem;
use zengine::render::render_system::RenderSystem;
use zengine::render::Background;
use zengine::render::Sprite;
use zengine::render::WindowSpecs;
use zengine::Engine;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum UserInput {
    Jump,
    Move_x,
}
impl InputType for UserInput {}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Sprites {
    Duck,
    Logo,
}
impl SpriteType for Sprites {}

fn main() {
    let mut bindings = Bindings::<UserInput> {
        action_mappings: HashMap::default(),
        axis_mappings: HashMap::default(),
    };

    bindings.action_mappings.insert(
        UserInput::Jump,
        vec![
            ActionBind {
                source: Input::Keyboard { key: Key::Space },
            },
            ActionBind {
                source: Input::ControllerButton {
                    device_id: 1,
                    button: ControllerButton::A,
                },
            },
            ActionBind {
                source: Input::MouseButton {
                    button: MouseButton::Left,
                },
            },
        ],
    );

    bindings.axis_mappings.insert(
        UserInput::Move_x,
        vec![
            AxisBind {
                source: Input::Keyboard { key: Key::A },
                scale: -1.0,
            },
            AxisBind {
                source: Input::Keyboard { key: Key::D },
                scale: 1.0,
            },
            AxisBind {
                source: Input::ControllerStick {
                    device_id: 1,
                    which: Which::Left,
                    axis: Axis::X,
                },
                scale: 1.0,
            },
        ],
    );

    Engine::default()
        .with_system(PlatformSystem::default())
        .with_system(InputSystem::new(bindings))
        .with_system(System1 {})
        .with_system(System2 {})
        .with_system(RenderSystem::<Sprites>::new(WindowSpecs::default()))
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(60)))
        .run(Game {
            execution_number: 10,
        });
}

#[derive(Debug)]
pub struct System1 {}

impl<'a> System<'a> for System1 {
    type Data = (
        WriteSet<'a, Test>,
        WriteSet<'a, Position>,
        ReadSet<'a, Test2>,
        ReadEntities<'a>,
        Read<'a, InputHandler<UserInput>>,
    );

    fn init(&mut self, store: &mut Store) {
        println!("System 1 init");
    }

    fn run(&mut self, (mut test, mut position, test2, entities, input_handler): Self::Data) {
        for e in test.iter() {
            if let Some(p) = position.get(e.0) {
                if let Some(t2) = test2.get(e.0) {

                }
            }
        }

    }

    fn dispose(&mut self, store: &mut Store) {
        println!("System 1 dispose");
    }
}

#[derive(Debug)]
pub struct System2 {}

impl<'a> System<'a> for System2 {
    type Data = ();
    fn init(&mut self, store: &mut Store) {
        println!("System 2 init");
    }

    fn run(&mut self, data: Self::Data) {
        //println!("System 2 run");
    }

    fn dispose(&mut self, store: &mut Store) {
        println!("System 2 dispose");
    }
}

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Test {
    pub data: u32,
}

#[derive(Debug)]
pub struct Test2 {
    pub data2: u32,
}

impl Component for Position {}
impl Component for Test {}
impl Component for Test2 {}

pub struct Game {
    execution_number: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        println!("Game scene on start");

        {
            let mut texture_manager = store.get_resource_mut::<TextureManager<Sprites>>().unwrap();

            texture_manager
                .build("sheet.png")
                .with_sprite(
                    Sprites::Duck,
                    SpriteDescriptor {
                        width: 170,
                        height: 200,
                        x: 0,
                        y: 0,
                    },
                )
                .with_sprite(
                    Sprites::Logo,
                    SpriteDescriptor {
                        width: 128,
                        height: 128,
                        x: 170,
                        y: 0,
                    },
                )
                .load();
        }

        let camera1 = store
            .build_entity()
            .with(Camera {
                width: 800,
                height: 600,
                mode: CameraMode::Mode2D,
            })
            .build();

        let camera2 = store
            .build_entity()
            .with(Camera {
                width: 600,
                height: 600,
                mode: CameraMode::Mode2D,
            })
            .with(Transform::new(
                Vector3::new(300.0, 300.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::one(),
            ))
            .build();

        store.insert_resource(ActiveCamera { entity: camera2 });

        store
            .build_entity()
            .with(Sprite {
                width: 40.0,
                height: 40.0,
                origin: Vector3::zero(),
                color: Color::white(),
                sprite_type: Sprites::Logo,
            })
            .with(Transform::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::one(),
            ))
            .build();

        store
            .build_entity()
            .with(Sprite {
                width: 200.0,
                height: 200.0,
                origin: Vector3::zero(),
                color: Color::red(),
                sprite_type: Sprites::Duck,
            })
            .with(Transform::new(
                Vector3::new(0.0, 200.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::one(),
            ))
            .build();

        store.insert_resource(Background {
            color: Color::white(),
        })
    }

    fn on_stop(&mut self, store: &mut Store) {
        println!("Game scene on stop");
    }

    fn update(&mut self, store: &mut Store) -> Trans {
        match self.execution_number {
            0 => Trans::None,
            _ => {
                //println!("Store {:?}", store);
                self.execution_number -= 1;
                Trans::None
            }
        }
    }
}
