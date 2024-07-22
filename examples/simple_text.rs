use zengine::{
    asset::{AssetManager, AssetModule},
    core::Transform,
    ecs::system::{Commands, ResMut},
    graphic::{Background, Camera, CameraMode, Color, GraphicModule},
    math::{vec2, Vec2, Vec3},
    text::{Text, TextModule, TextSection, TextStyle},
    window::{WindowConfig, WindowModule},
    Engine,
};
use zengine_asset::Handle;
use zengine_text::Font;

#[cfg(not(target_os = "android"))]
fn main() {
    Engine::default()
        .add_module(WindowModule(WindowConfig {
            title: "Simple Text".to_owned(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: false,
        }))
        .add_module(AssetModule::new("assets"))
        .add_module(GraphicModule)
        .add_module(TextModule)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut asset_manager: ResMut<AssetManager>) {
    let font: Handle<Font> = asset_manager.load("fonts/impact.ttf");

    commands.create_resource(Background {
        color: Color::new(35, 31, 32, 255),
    });

    let camera_width = 10.0;

    commands.spawn((
        Camera {
            mode: CameraMode::Mode2D(Vec2::new(camera_width, camera_width / 1.777)),
        },
        Transform::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));

    commands.spawn((
        Text::builder()
            .sections(vec![TextSection::builder()
                .value("Hello!!! ".to_string())
                .build()])
            .style(
                TextStyle::builder()
                    .font(font.clone_as_weak())
                    .font_size(80.)
                    .build(),
            )
            .bounds(vec2(4., 1.))
            .build(),
        Transform::new(Vec3::new(-1.3, 0., 0.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));
}
