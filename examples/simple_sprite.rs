use zengine::{
    asset::{AssetManager, AssetModule, Assets},
    core::Transform,
    ecs::system::{Commands, ResMut},
    graphic::{
        Background, Camera, CameraMode, Color, GraphicModule, Sprite, SpriteTexture, Texture,
        TextureAssets,
    },
    math::{Vec2, Vec3},
    window::{WindowConfig, WindowModule},
    Engine,
};

#[cfg(not(target_os = "android"))]
fn main() {
    Engine::default()
        .add_module(WindowModule(WindowConfig {
            title: "Simple Sprite".to_owned(),
            width: 1280,
            height: 720,
            fullscreen: false,
            vsync: false,
        }))
        .add_module(AssetModule::new("assets"))
        .add_module(GraphicModule::default())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut asset_manager: ResMut<AssetManager>,
    textures: Option<ResMut<Assets<Texture>>>,
) {
    let mut textures = textures.unwrap();
    let texture = asset_manager.load("branding/logo_extended.png");

    let texture = textures.create_texture(&texture);

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
        Sprite {
            width: 1.77,
            height: 1.,
            origin: Vec3::new(0.5, 0.5, 0.0),
            color: Color::WHITE,
            texture: SpriteTexture::Simple(texture),
        },
        Transform::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), 1.0),
    ));
}
