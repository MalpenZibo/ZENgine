use log::debug;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::fmt::Debug;
use std::io::Cursor;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use std::{collections::VecDeque, sync::atomic::AtomicU64};
use zengine_asset::{AssetExtension, AssetLoader, Assets, Handle, HandleId};
use zengine_ecs::system::{OptionalRes, OptionalResMut, Res, UnsendableRes};
use zengine_engine::{Module, StageLabel};
use zengine_macro::{Asset, Resource, UnsendableResource};

#[derive(Default)]
pub struct AudioModule;

impl Module for AudioModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine
            .add_asset::<Audio>()
            .add_asset::<AudioInstance>()
            .add_system_into_stage(audio_system, StageLabel::PostUpdate)
            .add_asset_loader(AudioLoader);
    }
}

#[derive(Debug)]
pub struct AudioLoader;
impl AssetLoader for AudioLoader {
    fn extension(&self) -> &[&str] {
        &["ogg", "wav", "flac"]
    }

    fn load(&self, data: Vec<u8>, context: &mut zengine_asset::LoaderContext) {
        context.set_asset(Audio { data });
    }
}

#[derive(Debug)]
pub struct AudioSettings {
    pub volume: f32,
    pub speed: f32,
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
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn in_loop(mut self) -> Self {
        self.in_loop = true;
        self
    }
}

#[derive(Asset, Debug)]
pub struct Audio {
    data: Vec<u8>,
}

#[derive(Asset)]
pub struct AudioInstance(Option<Sink>);
impl Debug for AudioInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioInstance")
    }
}

impl Drop for AudioInstance {
    fn drop(&mut self) {
        self.0.take().unwrap().detach();
    }
}

impl AudioInstance {
    pub fn volume(&self) -> f32 {
        self.0.as_ref().unwrap().volume()
    }

    pub fn set_volume(&self, volume: f32) {
        self.0.as_ref().unwrap().set_volume(volume);
    }

    pub fn speed(&self) -> f32 {
        self.0.as_ref().unwrap().speed()
    }

    pub fn set_speed(&self, speed: f32) {
        self.0.as_ref().unwrap().set_speed(speed);
    }

    pub fn play(&self) {
        self.0.as_ref().unwrap().play();
    }

    pub fn pause(&self) {
        self.0.as_ref().unwrap().pause();
    }

    pub fn is_paused(&self) -> bool {
        self.0.as_ref().unwrap().is_paused()
    }

    pub fn stop(&self) {
        self.0.as_ref().unwrap().stop();
    }
}

#[derive(Resource, Default, Debug)]
pub struct AudioDevice {
    instance_counter: AtomicU64,
    queue: RwLock<VecDeque<(HandleId, Handle<Audio>, AudioSettings)>>,
}

impl AudioDevice {
    pub fn play(&self, audio: Handle<Audio>) -> Handle<AudioInstance> {
        self.play_with_settings(audio, AudioSettings::default())
    }

    pub fn play_with_settings(
        &self,
        audio: Handle<Audio>,
        settings: AudioSettings,
    ) -> Handle<AudioInstance> {
        let next_id = self.instance_counter.fetch_add(1, Ordering::Relaxed);
        let handle_id = HandleId::new_manual::<AudioInstance>(next_id);

        debug!("created an Audio Instance handle {:?}", handle_id);

        self.queue
            .write()
            .unwrap()
            .push_back((handle_id, audio, settings));

        Handle::weak(handle_id)
    }
}

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

pub fn audio_system(
    audio_output: UnsendableRes<AudioOutput>,
    audio_device: Res<AudioDevice>,
    audio: OptionalRes<Assets<Audio>>,
    audio_instances: OptionalResMut<Assets<AudioInstance>>,
) {
    if let (Some(audio), Some(mut audio_instances)) = (audio, audio_instances) {
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
                let _ = audio_instances.set(Handle::weak(instance_id), audio_instance);
            } else {
                queue.push_back((instance_id, audio_handle, settings));
            }
            i += 1;
        }
    }
}
