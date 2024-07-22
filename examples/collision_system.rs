use std::time::{Duration, Instant};
use zengine::{
    asset::AssetModule,
    core::{Time, Transform},
    ecs::{
        query::{Query, QueryIter, QueryIterMut},
        system::{Commands, Local, Res, ResMut},
        Entity,
    },
    graphic::{Background, Camera, CameraMode, Color, GraphicModule},
    input::{Bindings, InputHandler},
    math::{Vec2, Vec3},
    physics::{CollisionModule, Shape2D, ShapeType},
    window::{WindowConfig, WindowModule},
    Component, Engine, InputType, Resource,
};

#[derive(InputType, Hash, Eq, PartialEq, Clone, Default, Debug)]
pub enum UserInput {
    #[default]
    XAxis,
    YAxis,
    Change,
}

#[derive(Component, Default, Debug)]
pub struct Controlled;

#[derive(Resource, Default, Debug)]
pub struct State {
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone)]
struct LocalInstant(Instant);
impl Default for LocalInstant {
    fn default() -> Self {
        LocalInstant(Instant::now())
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    use zengine_core::TimeModule;
    use zengine_input::{device::Key, ActionBind, AxisBind, Input, InputModule};

    let bindings: Bindings<UserInput> = Bindings::default()
        .add_axis(
            UserInput::XAxis,
            vec![
                AxisBind::with_source(Input::Keyboard { key: Key::KeyD }),
                AxisBind::with_source(Input::Keyboard { key: Key::KeyA }).invert_input(),
                AxisBind::with_source(Input::Keyboard {
                    key: Key::ArrowRight,
                }),
                AxisBind::with_source(Input::Keyboard {
                    key: Key::ArrowLeft,
                })
                .invert_input(),
            ],
        )
        .add_axis(
            UserInput::YAxis,
            vec![
                AxisBind::with_source(Input::Keyboard { key: Key::KeyW }),
                AxisBind::with_source(Input::Keyboard { key: Key::KeyS }).invert_input(),
                AxisBind::with_source(Input::Keyboard { key: Key::ArrowUp }),
                AxisBind::with_source(Input::Keyboard {
                    key: Key::ArrowDown,
                })
                .invert_input(),
            ],
        )
        .add_action(
            UserInput::Change,
            vec![ActionBind::with_source(Input::Keyboard { key: Key::Space })],
        );

    Engine::default()
        .add_module(WindowModule(WindowConfig {
            title: "Collision System".to_owned(),
            width: 800,
            height: 600,
            fullscreen: false,
            vsync: false,
        }))
        .add_module(AssetModule::new("assets"))
        .add_module(TimeModule(None))
        .add_module(GraphicModule)
        .add_module(InputModule(bindings))
        .add_module(CollisionModule::with_tracer())
        .add_startup_system(setup)
        .add_system(logics)
        .run();
}

fn setup(mut state: ResMut<State>, mut commands: Commands) {
    commands.create_resource(Background {
        color: Color::new(35, 31, 32, 255),
    });

    commands.spawn((
        Camera {
            mode: CameraMode::Mode2D(Vec2::new(1.33, 1.0)),
        },
        Transform::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    let e = commands.spawn((
        Shape2D {
            origin: Vec3::new(-1.0, -1.0, 0.0),
            shape_type: ShapeType::Circle { radius: 0.2 },
        },
        Transform::new(Vec3::new(-0.5, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
        Controlled,
    ));
    state.entities.push(e);

    let e = commands.spawn((
        Shape2D {
            origin: Vec3::new(0.0, 0.0, 0.0),
            shape_type: ShapeType::Circle { radius: 0.1 },
        },
        Transform::new(Vec3::new(-0.2, -0.2, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.3),
    ));
    state.entities.push(e);

    let e = commands.spawn((
        Shape2D {
            origin: Vec3::new(0.0, 0.0, 0.0),
            shape_type: ShapeType::Rectangle {
                width: 0.2,
                height: 0.1,
            },
        },
        Transform::new(Vec3::new(0.2, 0.2, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));
    state.entities.push(e);

    let e = commands.spawn((
        Shape2D {
            origin: Vec3::new(1.0, 0.0, 0.0),
            shape_type: ShapeType::Rectangle {
                width: 0.1,
                height: 0.05,
            },
        },
        Transform::new(Vec3::new(0.4, -0.3, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.2),
    ));
    state.entities.push(e);
}

fn logics(
    mut query: Query<(Entity, &mut Transform, &Controlled)>,
    input: Res<InputHandler<UserInput>>,
    state: Res<State>,
    mut commands: Commands,
    time: Res<Time>,
    last_change_time: Local<LocalInstant>,
) {
    let controlled = query.iter().next().unwrap().0;

    if input.action_value(UserInput::Change)
        && last_change_time.0.elapsed() > Duration::from_millis(500)
    {
        let skip = state
            .entities
            .iter()
            .enumerate()
            .find_map(|(i, e)| if e == controlled { Some(i) } else { None })
            .unwrap();

        let skip = if skip == state.entities.len() - 1 {
            0
        } else {
            skip + 1
        };

        let next = state.entities.iter().skip(skip).cycle().next().unwrap();

        commands.remove_components::<Controlled>(*controlled);
        commands.add_components(*next, Controlled);

        last_change_time.0 = Instant::now();
    }

    let delta = time.delta().as_secs_f32();

    for (_, transform, _) in query.iter_mut() {
        transform.position.x += 0.1 * delta * input.axis_value(UserInput::XAxis);
        transform.position.y += 0.1 * delta * input.axis_value(UserInput::YAxis);
    }
}
