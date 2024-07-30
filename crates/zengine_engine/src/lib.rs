use std::any::{Any, TypeId};

use schedule::{default_schedule::{Startup, Update}, ScheduleLabelInternal, Schedules};
use zengine_ecs::{
    system::{IntoSystem, SystemParam},
    World,
};

pub mod schedule;

/// A collection of engine logics and configurations.
///
/// A Module configure the [`Engine`]. When the [`Engine`] registers a module,
/// the module's [`Module::init`] function is call.
pub trait Module {
    /// Configures the [`Engine`] to which this module is added.
    fn init(self, engine: &mut Engine);
}

pub trait ScheduleLabel: Copy + Clone + 'static {
    fn internal(&self) -> ScheduleLabelInternal {
        ScheduleLabelInternal(TypeId::of::<Self>(), self.name())
    }

    fn name(&self) -> &'static str;
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
    schedules: Schedules,
    /// The main ECS [`World`] of the [`Engine`].
    /// This stores and provides access to all the data of the application.
    /// The systems of the [`Engine`] will run using this [`World`].
    pub world: World,
    runner: Box<dyn Fn(Engine)>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            schedules: Schedules::default(),
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
    /// Add a system to the [Engine] pipeling
    ///
    /// Using this funtion the system will be added to the default [Update Schedule Label](Update)
    pub fn add_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_schedule(system, Update)
    }

    /// Add a system to the [Engine] pipeling in the [Startup Schedule Label](Startup)
    ///
    /// The system added using this function will run only one time during the engine startup phase
    pub fn add_startup_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_schedule(system, Startup)
    }

    /// Add a system to the [Engine] pipeling in the specified [ScheduleLabel]
    pub fn add_system_into_schedule<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
        schedule: impl ScheduleLabel,
    ) -> &mut Self {
        self.schedules.add_system(schedule, system);

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
        self.schedules.startup(&mut self.world);
    }

    /// Update function of the engine. Should be called only one time for each frame
    pub fn update(&mut self) {
        self.schedules.run(&self.world);

        self.schedules.apply(&mut self.world);
    }

    /// Starts the engine by calling the engine's runner function
    pub fn run(&mut self) {
        self.world.create_event_handler::<EngineEvent>();

        let mut app = std::mem::take(self);
        let runner = std::mem::replace(&mut app.runner, Box::new(default_runner));

        (runner)(app);
    }
}
