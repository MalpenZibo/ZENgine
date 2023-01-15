use glam::vec2;
use zengine::{
    asset::{AssetManager, AssetModule},
    core::Transform,
    ecs::system::{Commands, ResMut},
    graphic::{Background, Camera, CameraMode, Color, GraphicModule},
    log::Level,
    math::{Vec2, Vec3},
    text::TextModule,
    text::{Text, TextAlignment, TextSection, TextStyle},
    window::{WindowConfig, WindowModule},
    Engine,
};

#[cfg(not(target_os = "android"))]
fn main() {
    Engine::init_logger(Level::Info);

    Engine::default()
        .add_module(WindowModule(WindowConfig {
            title: "Simple Text".to_owned(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: false,
        }))
        .add_module(AssetModule::new("assets"))
        .add_module(GraphicModule::default())
        .add_module(TextModule::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut asset_manager: ResMut<AssetManager>) {
    let font = asset_manager.load("fonts/impact.ttf");

    commands.create_resource(Background {
        color: Color::new(35, 31, 32, 255),
    });

    commands.spawn((
        Camera {
            mode: CameraMode::Mode2D(Vec2::new(3.55, 2.0)),
        },
        Transform::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    commands.spawn((
        Text {
            sections: vec![TextSection {
                value: "Test".to_string(),
                style: TextStyle {
                    font: font.clone_as_weak(),
                    font_size: 40.,
                    color: Color::WHITE,
                },
            }],
            alignment: TextAlignment::default(),
            bounds: vec2(200., 300.),
            color: Color::WHITE,
        },
        Transform::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));
}
