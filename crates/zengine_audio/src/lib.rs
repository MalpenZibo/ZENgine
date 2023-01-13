use log::debug;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::Cursor;
use std::sync::RwLock;
use zengine_asset::Asset;
use zengine_asset::{AssetExtension, AssetLoader, Assets, Handle, HandleId};
use zengine_ecs::system::{Local, Res, ResMut, UnsendableRes};
use zengine_engine::{Module, Stage};
use zengine_macro::{Asset, Resource, UnsendableResource};

/// Adds audio support to the engine
///
/// Use the [`AudioDevice`] resource to play audio.
#[derive(Default)]
pub struct AudioModule;

impl Module for AudioModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine
            .add_asset::<Audio>()
            .add_asset::<AudioInstance>()
            .add_asset_loader(AudioLoader)
            .add_system_into_stage(audio_system, Stage::PostUpdate)
            .add_system_into_stage(update_instances, Stage::PostRender);

        #[cfg(target_os = "android")]
        engine.add_system_into_stage(handle_resume_suspended, Stage::PreUpdate);
    }
}

#[derive(Debug)]
struct AudioLoader;
impl AssetLoader for AudioLoader {
    fn extension(&self) -> &[&str] {
        &["ogg", "wav", "flac"]
    }

    fn load(&self, data: Vec<u8>, context: &mut zengine_asset::LoaderContext) {
        context.set_asset(Audio { data });
    }
}

/// Initial audio playback settings
#[derive(Debug)]
pub struct AudioSettings {
    /// Volume to play at
    pub volume: f32,
    /// Speed to play at
    pub speed: f32,
    /// Play in loop
    pub in_loop: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            in_loop: false,
        }
    }
}

impl AudioSettings {
    /// Set the speed initial of playback
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set the volume initial of playback
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Will play the associated audio in loop
    pub fn in_loop(mut self) -> Self {
        self.in_loop = true;
        self
    }
}

/// An asset that rappresents an audio source
///
/// It could be a song or an audio effect
#[derive(Asset, Debug)]
pub struct Audio {
    data: Vec<u8>,
}

/// An asset that represents an instance of an [Audio] queued for playback
#[derive(Asset)]
pub struct AudioInstance(Option<Sink>);
impl Debug for AudioInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioInstance")
    }
}

impl AudioInstance {
    /// Get the current audio instance volume
    ///
    /// The value 1.0 is the “normal” volume. Any value other than 1.0
    /// will change the volume of the sound
    pub fn volume(&self) -> f32 {
        self.0.as_ref().unwrap().volume()
    }

    /// Set the audio instance volume
    ///
    /// The value 1.0 is the “normal” volume. Any value other than 1.0
    /// will change the volume of the sound
    pub fn set_volume(&self, volume: f32) {
        self.0.as_ref().unwrap().set_volume(volume);
    }

    /// Get the current audio instance speed
    ///
    /// The value 1.0 is the “normal” speed. Any value other than 1.0
    /// will change the speed of the sound
    pub fn speed(&self) -> f32 {
        self.0.as_ref().unwrap().speed()
    }

    /// Set the audio instance speed
    ///
    /// The value 1.0 is the “normal” speed. Any value other than 1.0
    /// will change the speed of the sound
    pub fn set_speed(&self, speed: f32) {
        self.0.as_ref().unwrap().set_speed(speed);
    }

    /// Play the audio instance
    pub fn play(&self) {
        self.0.as_ref().unwrap().play();
    }

    /// Pause the audio isntance
    pub fn pause(&self) {
        self.0.as_ref().unwrap().pause();
    }

    /// Returns true if the audio instance is paused
    pub fn is_paused(&self) -> bool {
        self.0.as_ref().unwrap().is_paused()
    }

    /// Returns true if the audio instance is stopped
    pub fn stop(&self) {
        self.0.as_ref().unwrap().stop();
    }

    /// Returns true if the audio instance has finished
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().unwrap().empty()
    }
}

/// A [Resource](zengine_ecs::Resource) that rappresent an audio device
///
/// Use this resource to play audio
///
/// # Example
/// ```
/// use zengine_asset::AssetManager;
/// use zengine_ecs::system::{Res, ResMut};
/// use zengine_audio::AudioDevice;
///
/// fn play_audio_system(mut asset_manager: ResMut<AssetManager>, audio_device: Res<AudioDevice>) {
///     audio_device.play(asset_manager.load("test_sound.ogg"));
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct AudioDevice {
    queue: RwLock<VecDeque<(HandleId, Handle<Audio>, AudioSettings)>>,
    instances: Vec<Handle<AudioInstance>>,
    suspended: Vec<Handle<AudioInstance>>,
}

impl AudioDevice {
    /// Play a sound from an [Handle] to an [Audio] asset
    ///
    /// Returns a weak handle to an [AudioInstance].
    /// Changing it to a strong handle allows to control
    /// the playback through the [AudioInstance] asset.
    ///
    /// NB: the AudioDevice maintains a strong reference
    /// to each AudioInstance that [is not empty](AudioInstance::is_empty).
    /// The strong reference are dropped when the AudioInstance is empty
    pub fn play(&self, audio: Handle<Audio>) -> Handle<AudioInstance> {
        self.play_with_settings(audio, AudioSettings::default())
    }

    /// Play a sound from an [Handle] to an [Audio] asset with an
    /// [AudioSettings]
    pub fn play_with_settings(
        &self,
        audio: Handle<Audio>,
        settings: AudioSettings,
    ) -> Handle<AudioInstance> {
        let next_id = AudioInstance::next_counter();
        let handle_id = HandleId::new_from_u64::<AudioInstance>(next_id);

        debug!("created an Audio Instance handle {:?}", handle_id);

        self.queue
            .write()
            .unwrap()
            .push_back((handle_id, audio, settings));

        Handle::weak(handle_id)
    }

    /// Suspend the audio device pausing every [AudioInstance] that is currently playing
    pub fn suspend(&mut self, audio_instances: &Assets<AudioInstance>) {
        for i in self.instances.iter() {
            let instance = audio_instances.get(i).unwrap();
            if !instance.is_paused() {
                instance.pause();
                self.suspended.push(i.clone_as_weak());
            }
        }
    }

    /// Resume the audio device starting every [AudioInstance] that were playing before the suspend
    pub fn resume(&mut self, audio_instances: &Assets<AudioInstance>) {
        for i in self.suspended.drain(..) {
            let instance = audio_instances.get(&i).unwrap();
            instance.play();
        }
    }
}

/// Used internally to play audio on the platform
#[derive(UnsendableResource)]
pub struct AudioOutput {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioManager")
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        let stream_handle = OutputStream::try_default().expect("Audio device not found");
        Self {
            _stream: stream_handle.0,
            stream_handle: stream_handle.1,
        }
    }
}

fn audio_system(
    audio_output: UnsendableRes<AudioOutput>,
    mut audio_device: ResMut<AudioDevice>,
    audio: Option<Res<Assets<Audio>>>,
    audio_instances: Option<ResMut<Assets<AudioInstance>>>,
    to_add: Local<Vec<Handle<AudioInstance>>>,
) {
    if let (Some(audio), Some(mut audio_instances)) = (audio, audio_instances) {
        {
            let mut queue = audio_device.queue.write().unwrap();
            let len = queue.len();
            let mut i = 0;

            while i < len {
                let (instance_id, audio_handle, settings) = queue.pop_front().unwrap();
                if let Some(audio) = audio.get(&audio_handle) {
                    let sink = Sink::try_new(&audio_output.stream_handle).unwrap();

                    if settings.in_loop {
                        sink.append(
                            rodio::Decoder::new(Cursor::new(audio.data.clone()))
                                .unwrap()
                                .repeat_infinite(),
                        );
                    } else {
                        sink.append(rodio::Decoder::new(Cursor::new(audio.data.clone())).unwrap())
                    };

                    sink.set_speed(settings.speed);
                    sink.set_volume(settings.volume);

                    let audio_instance = AudioInstance(Some(sink));
                    let handle = audio_instances.set(Handle::weak(instance_id), audio_instance);
                    to_add.push(handle);
                } else {
                    queue.push_back((instance_id, audio_handle, settings));
                }
                i += 1;
            }
        }

        for i in to_add.drain(..) {
            audio_device.instances.push(i);
        }
    }
}

#[cfg(target_os = "android")]
fn handle_resume_suspended(
    engine_event: zengine_ecs::system::EventStream<zengine_engine::EngineEvent>,
    mut audio_device: zengine_ecs::system::ResMut<AudioDevice>,
    audio_instances: Option<Res<Assets<AudioInstance>>>,
) {
    let last_event = engine_event.read().last();
    if last_event == Some(&zengine_engine::EngineEvent::Suspended) {
        if let Some(audio_instances) = audio_instances.as_ref() {
            audio_device.suspend(audio_instances);
        }
    }

    if last_event == Some(&zengine_engine::EngineEvent::Resumed) {
        if let Some(audio_instances) = audio_instances.as_ref() {
            audio_device.resume(audio_instances);
        }
    }
}

fn update_instances(
    mut audio_device: ResMut<AudioDevice>,
    audio_instances: Option<Res<Assets<AudioInstance>>>,
    to_remove: Local<Vec<usize>>,
) {
    if let Some(audio_instances) = audio_instances.as_ref() {
        for (index, handle) in audio_device.instances.iter().enumerate() {
            let instance = audio_instances.get(handle).unwrap();
            if instance.is_empty() {
                to_remove.push(index);
            }
        }

        for i in to_remove.drain(..).rev() {
            audio_device.instances.swap_remove(i);
        }
    }
}
