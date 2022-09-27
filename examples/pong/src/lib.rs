use serde::Deserialize;
use zengine::{
    asset::{AssetManager, AssetModule, Assets, Handle},
    audio::{Audio, AudioDevice, AudioInstance, AudioModule, AudioSettings},
    core::{timing_system, Time, Transform},
    ecs::{
        system::{
            Commands, EventPublisher, EventStream, Local, OptionalRes, OptionalResMut, Query,
            QueryIter, QueryIterMut, Res, ResMut,
        },
        Entity,
    },
    graphic::{
        ActiveCamera, Background, Camera, CameraMode, Color, GraphicModule, Sprite, SpriteTexture,
        Texture, TextureAssets, TextureAtlas, TextureAtlasAssets,
    },
    input::{input_system, Bindings, InputHandler},
    log::Level,
    math::{Vec2, Vec3},
    physics::{collision_system, Collision, Shape2D, ShapeType},
    window::{WindowModule, WindowSpecs},
    zengine_main, Component, Engine, InputType, Resource, StageLabel,
};

static WIDTH: f32 = 1280.0;
static HEIGHT: f32 = 720.0;
static BOARD_WIDTH: f32 = HEIGHT / 1.33;

static PAD_HALF_WIDTH: f32 = 75.0;
static PAD_HALF_HEIGHT: f32 = 15.0;
static PAD_FORCE: f32 = 2000.0;
static PAD_MASS: f32 = 5.0;
static BALL_RADIUS: f32 = 12.5;
static BALL_VEL: f32 = 250.0;

#[derive(Deserialize, InputType, Hash, Eq, PartialEq, Clone, Default, Debug)]
pub enum UserInput {
    #[default]
    Player1XAxis,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Sprites {
    Background,
    Pad,
    Ball,
}

#[derive(Resource, Debug)]
pub struct BounceEffect(Handle<Audio>);

#[derive(Resource, Debug)]
pub struct ScoreEffect(Handle<Audio>);

#[derive(Resource, Debug)]
pub struct BgMusic(Handle<AudioInstance>);

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
    vel: Vec2,
}

#[derive(Debug, Default, Resource)]
pub struct GameSettings {
    drag_constant: f32,
}

#[derive(Debug)]
pub enum GameEvent {
    Score,
}

#[zengine_main]
pub fn main() {
    Engine::init_logger(Level::Info);

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
                Keyboard:
                    key: Right
              scale: 1.0
            - source: 
                Keyboard: 
                    key: Left
              scale: -1.0
            - source:
                ControllerStick:
                    device_id: 0
                    which: Left
                    axis: X
              scale: 1.0
    ";

    let bindings: Bindings<UserInput> = serde_yaml::from_str(content).unwrap();

    Engine::default()
        .add_module(WindowModule(WindowSpecs {
            title: "PONG".to_owned(),
            width: WIDTH as u32,
            height: HEIGHT as u32,
            fullscreen: false,
            vsync: false,
        }))
        .add_module(AssetModule::new("assets"))
        .add_module(GraphicModule::default())
        //.add_module(AudioModule::default())
        //.add_startup_system(setup)
        // .add_system(input_system(bindings))
        // .add_system(collision_system)
        // .add_system(ai_pad_control)
        // .add_system(player_pad_control)
        // .add_system(pad_movement)
        // .add_system(ball_movement)
        // .add_system(collision_response)
        .add_system_into_stage(timing_system(None), StageLabel::PostRender)
        .run();
}

fn setup(
    mut commands: Commands,
    mut asset_manager: ResMut<AssetManager>,
    mut textures: OptionalResMut<Assets<Texture>>,
    mut textures_atlas: OptionalResMut<Assets<TextureAtlas>>,
    audio_device: Res<AudioDevice>,
    audio_instances: OptionalRes<Assets<AudioInstance>>,
) {
    let textures = textures.as_mut().unwrap();
    let textures_atlas = textures_atlas.as_mut().unwrap();

    commands.create_resource(BounceEffect(asset_manager.load("audio/bounce.wav")));
    commands.create_resource(ScoreEffect(asset_manager.load("audio/score.wav")));

    let bg = asset_manager.load("audio/bg.ogg");
    let mut bg =
        audio_device.play_with_settings(bg, AudioSettings::default().in_loop().with_volume(0.4));
    bg.make_strong(audio_instances.as_ref().unwrap());
    commands.create_resource(BgMusic(bg));

    let bg = asset_manager.load("images/bg.jpg");
    let board = asset_manager.load("images/board.png");
    let pad_image = asset_manager.load("images/pad.png");
    let ball = asset_manager.load("images/ball.png");

    let bg = textures.create_texture(&bg);
    let atlas = textures_atlas.create_texture_atlas(&[&board, &pad_image, &ball]);

    commands.create_resource(GameSettings {
        drag_constant: 10.0,
    });

    commands.create_resource(Background {
        color: Color::BLACK,
    });

    let pad = Pad {
        force: PAD_FORCE,
        mass: PAD_MASS,
        cur_acc: 0.0,
        velocity: 0.0,
    };

    let camera = commands.spawn((
        Camera {
            mode: CameraMode::Mode2D((WIDTH, HEIGHT)),
        },
        Transform::new(Vec3::new(0.0, 0.0, -50.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    commands.create_resource(ActiveCamera { entity: camera });

    commands.spawn((
        Sprite {
            width: WIDTH,
            height: HEIGHT,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Simple(bg),
        },
        Transform::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    let height = HEIGHT;
    let width = BOARD_WIDTH;

    commands.spawn((
        Sprite {
            width,
            height,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Atlas {
                texture_handle: atlas.clone(),
                target_image: Some(board),
            },
        },
        Transform::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    let sx = commands.spawn((
        Transform::new(
            Vec3::new(-width / 2., 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(1.0, 0.5, 0.0),
            shape_type: ShapeType::Rectangle {
                width: 300.0,
                height,
            },
        },
    ));
    let dx = commands.spawn((
        Transform::new(
            Vec3::new(width / 2., 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(0.0, 0.5, 0.0),
            shape_type: ShapeType::Rectangle {
                width: 300.0,
                height: HEIGHT,
            },
        },
    ));

    let top = commands.spawn((
        Transform::new(
            Vec3::new(0.0, height / 2., 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(0.5, 0.0, 0.0),
            shape_type: ShapeType::Rectangle {
                width: WIDTH,
                height: 300.0,
            },
        },
    ));

    let bottom = commands.spawn((
        Transform::new(
            Vec3::new(0.0, -height / 2., 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(0.5, 1.0, 0.0),
            shape_type: ShapeType::Rectangle {
                width: WIDTH,
                height: 300.0,
            },
        },
    ));

    commands.create_resource(FieldBorder {
        sx,
        dx,
        top,
        bottom,
    });

    let pad1 = commands.spawn((
        Sprite {
            width: PAD_HALF_WIDTH * 2.0,
            height: PAD_HALF_HEIGHT * 2.0,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Atlas {
                texture_handle: atlas.clone(),
                target_image: Some(pad_image.clone_as_weak()),
            },
        },
        Transform::new(
            Vec3::new(0.0, -(height / 2.) + 20.0 + PAD_HALF_HEIGHT, 1.0),
            Vec3::ZERO,
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(0.5, 0.5, 0.0),
            shape_type: ShapeType::Rectangle {
                width: PAD_HALF_WIDTH * 2.0,
                height: PAD_HALF_HEIGHT * 2.0,
            },
        },
        pad.clone(),
    ));
    commands.create_resource(Player1 { entity: pad1 });

    commands.spawn((
        Sprite {
            width: PAD_HALF_WIDTH * 2.0,
            height: PAD_HALF_HEIGHT * 2.0,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Atlas {
                texture_handle: atlas.clone(),
                target_image: Some(pad_image.clone_as_weak()),
            },
        },
        Transform::new(
            Vec3::new(0.0, height / 2. - 20.0 - PAD_HALF_HEIGHT, 1.0),
            Vec3::new(0., 0., 180.),
            1.0,
        ),
        Shape2D {
            origin: Vec3::new(0.5, 0.5, 0.0),
            shape_type: ShapeType::Rectangle {
                width: PAD_HALF_WIDTH * 2.0,
                height: PAD_HALF_HEIGHT * 2.0,
            },
        },
        pad,
        AI {},
    ));

    commands.spawn((
        Sprite {
            width: BALL_RADIUS * 2.0,
            height: BALL_RADIUS * 2.0,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Atlas {
                texture_handle: atlas,
                target_image: Some(ball.clone_as_weak()),
            },
        },
        Transform::new(Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, 1.0),
        Shape2D {
            origin: Vec3::new(0.5, 0.5, 0.0),
            shape_type: ShapeType::Circle {
                radius: BALL_RADIUS,
            },
        },
        Ball {
            vel: initial_ball_movement(),
        },
    ));
}

fn player_pad_control(
    mut pads: Query<(Entity, &mut Pad)>,
    player1: OptionalRes<Player1>,
    input: Res<InputHandler<UserInput>>,
) {
    if let Some(pad) = player1.and_then(|player1| {
        pads.iter_mut().find_map(|(e, pad)| {
            if e == &player1.entity {
                Some(pad)
            } else {
                None
            }
        })
    }) {
        pad.cur_acc = input.axis_value(UserInput::Player1XAxis) * pad.force / pad.mass;
    }
}

fn ai_pad_control(
    mut ai_query: Query<(&AI, &mut Pad, &Transform)>,
    ball_query: Query<(&Ball, &Transform)>,
) {
    if let Some(ball_transform) = ball_query
        .iter()
        .next()
        .map(|(_, ball_transform)| ball_transform.clone())
    {
        for (_, pad, transform) in ai_query.iter_mut() {
            pad.cur_acc = if ball_transform.position.x > transform.position.x {
                1.0
            } else {
                -1.0
            } * pad.force
                / pad.mass;
        }
    }
}

fn pad_movement(
    mut query: Query<(&mut Transform, &mut Pad)>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    for (transform, pad) in query.iter_mut() {
        let drag_acc = -game_settings.drag_constant * pad.velocity / pad.mass;
        pad.velocity +=
            pad.cur_acc * time.delta.as_secs_f32() + drag_acc * time.delta.as_secs_f32();
        transform.position.x += pad.velocity * time.delta.as_secs_f32();
    }

    for (transform, pad) in query.iter_mut() {
        let drag_acc = -game_settings.drag_constant * pad.velocity / pad.mass;
        pad.velocity +=
            pad.cur_acc * time.delta.as_secs_f32() + drag_acc * time.delta.as_secs_f32();
        transform.position.x += pad.velocity * time.delta.as_secs_f32();
    }
}

#[derive(Debug, Default)]
struct BallMovement {
    launched: bool,
    respawn: f32,
}

fn ball_movement(
    mut query: Query<(&mut Transform, &mut Ball)>,
    time: Res<Time>,
    game_events: EventStream<GameEvent>,
    ball_movement: Local<BallMovement>,
) {
    if let Some(GameEvent::Score) = game_events.read().last() {
        ball_movement.launched = false;
    }
    if ball_movement.launched {
        for (transform, ball) in query.iter_mut() {
            transform.position.x += ball.vel.x * time.delta.as_secs_f32();
            transform.position.y += ball.vel.y * time.delta.as_secs_f32();
        }
    } else {
        ball_movement.respawn += time.delta.as_secs_f32();
        if ball_movement.respawn > 5.0 {
            ball_movement.launched = true;
            ball_movement.respawn = 0.0;
        }
    }
}

enum CollisionType {
    PadBorder { pad: Entity, border: Side },
    BallBorder { ball: Entity, border: Side },
    BallPad { pad: Entity, ball: Entity },
}

enum Side {
    Sx(Entity),
    Dx(Entity),
    Bottom(Entity),
    Top(Entity),
}

fn initial_ball_movement() -> Vec2 {
    let angle =
        (fastrand::i32(70..110) as f32 + if fastrand::bool() { 180.0 } else { 0.0 }).to_radians();
    Vec2::new(BALL_VEL * angle.cos(), BALL_VEL * angle.sin())
}

#[allow(clippy::too_many_arguments)]
fn collision_response(
    mut query_pad: Query<(Entity, &mut Transform, &mut Pad)>,
    mut query_ball: Query<(Entity, &mut Transform, &mut Ball)>,
    collision_event: EventStream<Collision>,
    field_border: OptionalRes<FieldBorder>,
    mut game_event: EventPublisher<GameEvent>,
    audio_device: Res<AudioDevice>,
    bounce_effect: OptionalRes<BounceEffect>,
    score_effect: OptionalRes<ScoreEffect>,
) {
    fn get_collision_type(
        collision: &Collision,
        query_pad: &Query<(Entity, &mut Transform, &mut Pad)>,
        query_ball: &Query<(Entity, &mut Transform, &mut Ball)>,
        field_border: &FieldBorder,
    ) -> Option<CollisionType> {
        let get_field_border = |entity: Entity| -> Option<Side> {
            if field_border.sx == entity {
                return Some(Side::Sx(entity));
            } else if field_border.dx == entity {
                return Some(Side::Dx(entity));
            } else if field_border.bottom == entity {
                return Some(Side::Bottom(entity));
            } else if field_border.top == entity {
                return Some(Side::Top(entity));
            }

            None
        };

        if query_pad
            .iter()
            .any(|(entity, _, _)| entity == &collision.entity_a)
        {
            if let Some(border) = get_field_border(collision.entity_b) {
                return Some(CollisionType::PadBorder {
                    pad: collision.entity_a,
                    border,
                });
            } else if query_ball
                .iter()
                .any(|(entity, _, _)| entity == &collision.entity_b)
            {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_a,
                    ball: collision.entity_b,
                });
            }
        } else if query_pad
            .iter()
            .any(|(entity, _, _)| entity == &collision.entity_b)
        {
            if let Some(border) = get_field_border(collision.entity_a) {
                return Some(CollisionType::PadBorder {
                    pad: collision.entity_b,
                    border,
                });
            } else if query_ball
                .iter()
                .any(|(entity, _, _)| entity == &collision.entity_a)
            {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_b,
                    ball: collision.entity_a,
                });
            }
        } else if query_ball
            .iter()
            .any(|(entity, _, _)| entity == &collision.entity_a)
        {
            if let Some(border) = get_field_border(collision.entity_b) {
                return Some(CollisionType::BallBorder {
                    ball: collision.entity_a,
                    border,
                });
            } else if query_pad
                .iter()
                .any(|(entity, _, _)| entity == &collision.entity_b)
            {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_b,
                    ball: collision.entity_a,
                });
            }
        } else if query_ball
            .iter()
            .any(|(entity, _, _)| entity == &collision.entity_b)
        {
            if let Some(border) = get_field_border(collision.entity_a) {
                return Some(CollisionType::BallBorder {
                    ball: collision.entity_b,
                    border,
                });
            } else if query_pad
                .iter()
                .any(|(entity, _, _)| entity == &collision.entity_a)
            {
                return Some(CollisionType::BallPad {
                    pad: collision.entity_a,
                    ball: collision.entity_b,
                });
            }
        }

        None
    }

    if let Some(field_border) = field_border {
        for c in collision_event.read() {
            match get_collision_type(c, &query_pad, &query_ball, &field_border) {
                Some(CollisionType::PadBorder {
                    pad: pad_entity,
                    border: Side::Sx(_),
                }) => {
                    if let Some((pad, transform)) = query_pad.iter_mut().find_map(|(e, t, p)| {
                        if e == &pad_entity {
                            Some((p, t))
                        } else {
                            None
                        }
                    }) {
                        pad.velocity = 0.0;
                        transform.position.x = (-BOARD_WIDTH / 2.) + PAD_HALF_WIDTH;
                    }
                }
                Some(CollisionType::PadBorder {
                    pad: pad_entity,
                    border: Side::Dx(_),
                }) => {
                    if let Some((pad, transform)) = query_pad.iter_mut().find_map(|(e, t, p)| {
                        if e == &pad_entity {
                            Some((p, t))
                        } else {
                            None
                        }
                    }) {
                        pad.velocity = 0.0;
                        transform.position.x = (BOARD_WIDTH / 2.) - PAD_HALF_WIDTH;
                    };
                }
                Some(CollisionType::BallBorder {
                    ball: ball_entity,
                    border: Side::Sx(border_entity),
                })
                | Some(CollisionType::BallBorder {
                    ball: ball_entity,
                    border: Side::Dx(border_entity),
                }) => {
                    if let Some((ball, transform)) = query_ball.iter_mut().find_map(|(e, t, b)| {
                        if e == &ball_entity {
                            Some((b, t))
                        } else {
                            None
                        }
                    }) {
                        ball.vel = Vec2::new(-ball.vel.x, ball.vel.y);
                        transform.position.x = if border_entity == field_border.sx {
                            (-BOARD_WIDTH / 2.) + BALL_RADIUS
                        } else {
                            (BOARD_WIDTH / 2.0) - BALL_RADIUS
                        }
                    }

                    audio_device.play(bounce_effect.as_ref().unwrap().0.clone());
                }
                Some(CollisionType::BallBorder {
                    ball: ball_entity,
                    border: Side::Bottom(_),
                })
                | Some(CollisionType::BallBorder {
                    ball: ball_entity,
                    border: Side::Top(_),
                }) => {
                    if let Some((ball, transform)) = query_ball.iter_mut().find_map(|(e, t, b)| {
                        if e == &ball_entity {
                            Some((b, t))
                        } else {
                            None
                        }
                    }) {
                        transform.position.x = 0.0;
                        transform.position.y = 0.0;

                        ball.vel = initial_ball_movement();
                    }

                    game_event.publish(GameEvent::Score);

                    audio_device.play(score_effect.as_ref().unwrap().0.clone());

                    return;
                }
                Some(CollisionType::BallPad {
                    pad: pad_entity,
                    ball: ball_entity,
                }) => {
                    if let Some(pad_transform) = query_pad.iter().find_map(|(e, t, _)| {
                        if e == &pad_entity {
                            Some(t.clone())
                        } else {
                            None
                        }
                    }) {
                        if let Some((ball_transform, ball)) =
                            query_ball.iter_mut().find_map(|(e, t, b)| {
                                if e == &ball_entity {
                                    Some((t, b))
                                } else {
                                    None
                                }
                            })
                        {
                            ball.vel = Vec2::new(
                                ball.vel.x
                                    + (if ball_transform.position.x < pad_transform.position.x {
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
                                + if ball_transform.position.y > pad_transform.position.y {
                                    PAD_HALF_HEIGHT + BALL_RADIUS
                                } else {
                                    -(PAD_HALF_HEIGHT + BALL_RADIUS)
                                };

                            audio_device.play(bounce_effect.as_ref().unwrap().0.clone());
                        }
                    }
                }
                _ => {}
            };
        }
    }
}
