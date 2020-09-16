extern crate zengine;

use zengine::core::system::*;
use zengine::core::*;
use zengine::event::*;
use zengine::graphics::color::Color;
use zengine::graphics::texture::{SpriteDescriptor, SpriteSheet, SpriteType, TextureManager};
use zengine::log::{info, trace, LevelFilter};
use zengine::math::transform::Transform;
use zengine::math::vector3::Vector3;
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
        .with_system(System1::default())
        .with_system(System2::default())
        .with_system(RenderSystem::<String, Sprites>::new(WindowSpecs::default()))
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(60)))
        .run(Game {
            execution_numer: 10,
        });
}

#[derive(Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum UserInput {
    X,
    Jump,
}
impl InputType for UserInput {}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Sprites {
    Duck,
}
impl SpriteType for Sprites {}

pub struct Game {
    execution_numer: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        trace!("Game scene on start");

        {
            let mut textures = store
                .get_resource_mut::<TextureManager<String, Sprites>>()
                .unwrap();

            textures
                .create("duck.png".to_string())
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
        }
        store.insert_resource(Background {
            color: Color::white(),
        });

        store
            .build_entity()
            .with(Sprite::<Sprites> {
                width: 240.0,
                height: 240.0,
                origin: Vector3::zero(),
                color: Color::white(),
                sprite_type: Sprites::Duck,
            })
            .with(Transform::default())
            .build();

        store
            .build_entity()
            .with(Sprite {
                width: 20.0,
                height: 20.0,
                origin: Vector3::zero(),
                color: Color::blue(),
                sprite_type: Sprites::Duck,
            })
            .with(Transform::new(
                Vector3::new(200.0, 100.0, 0.0),
                Vector3::zero(),
                Vector3::one(),
            ))
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
}

impl<'a> System<'a> for System1 {
    type Data = (
        ReadSet<'a, Component1>,
        WriteSet<'a, Component2>,
        Read<'a, InputHandler<UserInput>>,
    );

    fn init(&mut self, _store: &mut Store) {
        trace!("setup system 1");
    }

    fn run(&mut self, data: Self::Data) {
        trace!("run {} system 1", self.run_count);

        let (c1, mut c2, input) = data;

        info!("user x input: {:?}", input.axis_value(UserInput::X));
        info!("user jump input: {:?}", input.action_value(UserInput::Jump));

        for c in c2.values_mut() {
            c.data2 += 1;
        }

        trace!("c1 {:?}", c1);
        trace!("c2 {:?}", c2);

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
