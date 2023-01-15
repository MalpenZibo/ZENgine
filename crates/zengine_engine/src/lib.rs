use std::{any::Any, collections::HashMap};

use zengine_ecs::{
    system::{IntoSystem, System, SystemParam},
    World,
};

pub use log;

/// A collection of engine logics and configurations.
///
/// A Module configure the [`Engine`]. When the [`Engine`] registers a module,
/// the module's [`Module::init`] function is call.
pub trait Module {
    /// Configures the [`Engine`] to which this module is added.
    fn init(self, engine: &mut Engine);
}

/// The possible stages in the engine pipeline
#[derive(Hash, Eq, PartialEq)]
pub enum Stage {
    /// Statup stage, runs only one time when the engine start
    Startup,
    /// Run just before the main update stage
    PreUpdate,
    /// Main stage
    Update,
    /// Run after the update stage
    PostUpdate,
    /// Run before the render stage
    PreRender,
    /// Render stage, draws the new state
    Render,
    /// Run after the render stage
    PostRender,
}

#[derive(Default)]
struct SystemsStage {
    systems: Vec<Box<dyn System>>,
}

impl SystemsStage {
    pub fn init(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.init(world);
        }
    }

    pub fn run(&mut self, world: &World) {
        for s in self.systems.iter_mut() {
            s.run(world);
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.apply(world);
        }
    }

    pub fn run_and_apply(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.run(world);
            s.apply(world);
        }
    }
}

/// List of all engine events
#[derive(Debug, PartialEq, Eq)]
pub enum EngineEvent {
    /// Fired when the engine is closing
    Quit,
    /// Only in Android - Fired when the Activity goes in background
    Suspended,
    /// Only in Android - Fired when the Activity goes in foreground
    Resumed,
}

/// A container of engine logic and data.
///
/// Bundles together the necessary elements to create an engine instance.
/// It also stores a pointer to a [runner function](Self::set_runner).
///
/// The runner is responsible for managing the engine's event loop
/// and call the engine update function to drive application logic.
///
/// # Examples
///
/// Here is a simple "Hello World" ZENgine app:
///
/// ```no_run
/// use zengine_engine::Engine;
///
/// fn main() {
///     Engine::default().add_system(hello_world_system).run();
/// }
///
/// fn hello_world_system() {
///     println!("hello world");
/// }
/// ```
pub struct Engine {
    stages: HashMap<Stage, SystemsStage>,
    stage_order: Vec<Stage>,
    running_stages: Vec<SystemsStage>,
    /// The main ECS [`World`] of the [`Engine`].
    /// This stores and provides access to all the data of the application.
    /// The systems of the [`Engine`] will run using this [`World`].
    pub world: World,
    runner: Box<dyn Fn(Engine)>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            stages: HashMap::from([
                (Stage::Startup, SystemsStage::default()),
                (Stage::PreUpdate, SystemsStage::default()),
                (Stage::Update, SystemsStage::default()),
                (Stage::PostUpdate, SystemsStage::default()),
                (Stage::PreRender, SystemsStage::default()),
                (Stage::Render, SystemsStage::default()),
                (Stage::PostRender, SystemsStage::default()),
            ]),
            stage_order: vec![
                Stage::Startup,
                Stage::PreUpdate,
                Stage::Update,
                Stage::PostUpdate,
                Stage::PreRender,
                Stage::Render,
                Stage::PostRender,
            ],
            running_stages: Vec::default(),
            world: World::default(),
            runner: Box::new(default_runner),
        }
    }
}

fn default_runner(mut engine: Engine) {
    engine.startup();

    loop {
        engine.update();

        if engine
            .world
            .get_event_handler::<EngineEvent>()
            .and_then(|event| event.read_last().map(|e| e == &EngineEvent::Quit))
            .unwrap_or(false)
        {
            break;
        }
    }
}

impl Engine {
    /// Initialize the logging utilities setting minimum log level
    pub fn init_logger(level: log::Level) {
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(level).expect("Couldn't initialize logger");
        }

        #[cfg(target_os = "android")]
        {
            android_logger::init_once(android_logger::Config::default().with_min_level(level));
            log_panics::init();
        }

        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            use simplelog::{Config, SimpleLogger, TermLogger, TerminalMode};
            let level_filter = level.to_level_filter();
            if TermLogger::init(level_filter, Config::default(), TerminalMode::Mixed).is_err() {
                SimpleLogger::init(level_filter, Config::default())
                    .expect("Couldn't initialize logger")
            }

            log_panics::init();
        }
    }

    /// Add a system to the [Engine] pipeling
    ///
    /// Using this funtion the system will be added to the default [Update Stage](Stage::Update)
    pub fn add_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_stage(system, Stage::Update)
    }

    /// Add a system to the [Engine] pipeling in the [Startup Stage](Stage::Startup)
    ///
    /// The system added using this function will run only one time during the engine startup phase
    pub fn add_startup_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_stage(system, Stage::Startup)
    }

    /// Add a system to the [Engine] pipeling in the specified [Stage]
    pub fn add_system_into_stage<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
        stage: Stage,
    ) -> &mut Self {
        if let Some(stage) = self.stages.get_mut(&stage) {
            stage.systems.push(Box::new(system.into_system()));
        }

        self
    }

    /// Add a [Module] to the engine
    pub fn add_module(&mut self, module: impl Module) -> &mut Self {
        module.init(self);

        self
    }

    /// Set the engine runner funtion
    ///
    /// This function is responsable of running the main event loop of the engine.
    ///
    /// By default the engine use this runner implementation
    ///
    /// ```
    /// use zengine_engine::{Engine, EngineEvent};
    ///
    /// fn default_runner(mut engine: Engine) {
    ///     engine.startup();
    ///
    ///     loop {
    ///         engine.update();
    ///
    ///         if engine
    ///             .world
    ///             .get_event_handler::<EngineEvent>()
    ///             .and_then(|event| event.read_last().map(|e| e == &EngineEvent::Quit))
    ///             .unwrap_or(false)
    ///         {
    ///             break;
    ///         }
    ///     }
    /// }
    ///```
    pub fn set_runner<F: Fn(Engine) + 'static>(&mut self, runner: F) -> &mut Self {
        self.runner = Box::new(runner);
        self
    }

    /// Startup function of the engine. Should be called only one time before the update function
    pub fn startup(&mut self) {
        let mut stages: Vec<SystemsStage> = self
            .stage_order
            .iter()
            .map(|stage| self.stages.remove(stage).unwrap())
            .collect();

        for stage in stages.iter_mut() {
            stage.init(&mut self.world);
        }

        let mut startup_stage = stages.remove(0);
        startup_stage.run_and_apply(&mut self.world);

        self.running_stages = stages;
    }

    /// Update function of the engine. Should be called only one time for each frame
    pub fn update(&mut self) {
        for stage in self.running_stages.iter_mut() {
            stage.run(&self.world);
        }

        for stage in self.running_stages.iter_mut() {
            stage.apply(&mut self.world);
        }
    }

    /// Starts the engine by calling the engine's runner function
    pub fn run(&mut self) {
        self.world.create_event_handler::<EngineEvent>();

        let mut app = std::mem::take(self);
        let runner = std::mem::replace(&mut app.runner, Box::new(default_runner));

        (runner)(app);
    }
}
