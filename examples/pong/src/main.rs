extern crate zengine;

use zengine::basic::input::Bindings;
use zengine::basic::input::InputType;
use zengine::basic::input::{InputHandler, InputSystem};
use zengine::basic::platform::PlatformSystem;
use zengine::basic::render::Background;
use zengine::basic::render::WindowSpecs;
use zengine::basic::render::{RenderSystem, Sprite};
use zengine::basic::timing::{FrameLimiter, TimingSystem};
use zengine::core::system::Read;
use zengine::core::system::{ReadSet, WriteSet};
use zengine::core::Component;
use zengine::core::Scene;
use zengine::core::Store;
use zengine::core::System;
use zengine::core::Trans;
use zengine::event::event_stream::EventStream;
use zengine::event::event_stream::SubscriptionToken;
use zengine::graphics::color::Color;
use zengine::log::{info, trace, LevelFilter};
use zengine::math::transform::Transform;
use zengine::math::vector3::Vector3;
use zengine::serde::Deserialize;
use zengine::serde_yaml;
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
        .with_system(RenderSystem::new(WindowSpecs::default()))
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

pub struct Game {
    execution_numer: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        trace!("Game scene on start");

        store.insert_resource(Background {
            color: Color::white(),
        });

        store
            .build_entity()
            .with(Sprite {
                width: 40.0,
                height: 40.0,
                origin: Vector3::zero(),
                color: Color::black(),
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

    fn on_stop(&mut self, store: &mut Store) {
        trace!("Game scene on stop");
    }

    fn update(&mut self, store: &Store) -> Trans {
        match self.execution_numer {
            0 => Trans::None,
            _ => {
                self.execution_numer -= 1;
                Trans::None
            }
        }
    }
}

#[derive(Debug)]
pub struct Component1 {
    data: u32,
}

impl Component for Component1 {}

#[derive(Debug)]
pub struct Component2 {
    data2: u32,
}

impl Component for Component2 {}

#[derive(Debug)]
pub struct Component3 {
    data2: u32,
}

impl Component for Component3 {}

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

    fn init(&mut self, store: &mut Store) {
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

    fn dispose(&mut self, store: &mut Store) {
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

    fn run(&mut self, (set, stream): Self::Data) {
        trace!("run {} system 2", self.run_count);

        trace!("data 2: {:?}", set);
        //trace!("data 2: {:?}", stream.read(&self.token.unwrap()));

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        trace!("dispose system 2");
    }
}
