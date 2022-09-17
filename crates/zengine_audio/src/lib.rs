use log::debug;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::any::TypeId;
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

#[derive(Asset, Debug)]
pub struct Audio {
    data: Vec<u8>,
}

#[derive(Asset)]
pub struct AudioInstance(Sink);
impl Debug for AudioInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioInstance")
    }
}

#[derive(Resource, Default, Debug)]
pub struct AudioDevice {
    instance_counter: AtomicU64,
    queue: RwLock<VecDeque<(HandleId, Handle<Audio>)>>,
}

impl AudioDevice {
    pub fn play(&self, audio: Handle<Audio>) -> Handle<AudioInstance> {
        let next_id = self.instance_counter.fetch_add(1, Ordering::Relaxed);
        let type_id = TypeId::of::<AudioInstance>();
        let handle_id = HandleId::new_manual(type_id, next_id);

        debug!("created an Audio Instance handle {:?}", handle_id);

        self.queue.write().unwrap().push_back((handle_id, audio));

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
            let (instance_id, audio_handle) = queue.pop_front().unwrap();
            if let Some(audio) = audio.get(&audio_handle.id) {
                let sink = Sink::try_new(&audio_output.stream_handle).unwrap();
                let decoder = rodio::Decoder::new(Cursor::new(audio.data.clone())).unwrap();
                sink.append(decoder);

                let audio_instance = AudioInstance(sink);
                let _ = audio_instances.set(&instance_id, audio_instance);
            } else {
                queue.push_back((instance_id, audio_handle));
            }
            i += 1;
        }
    }
}
