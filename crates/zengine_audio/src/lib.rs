use rodio::{OutputStream, OutputStreamHandle, Sink};
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Cursor;
use std::{any::Any, collections::VecDeque};
use zengine_asset::audio_loader::{self, AudioAssetHandler};
use zengine_ecs::system::UnsendableResMut;
use zengine_macro::UnsendableResource;

pub trait AudioType: Any + Eq + PartialEq + Hash + Clone + Debug + Send + Sync {}
impl AudioType for String {}

#[derive(Debug)]
pub struct Audio {
    audio_asset: AudioAssetHandler,
    data: Option<Vec<u8>>,
}

#[derive(UnsendableResource)]
pub struct AudioManager<AT: AudioType> {
    sink_id: usize,
    audio_asset: FxHashMap<AT, Audio>,
    queue: VecDeque<(usize, AT)>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: FxHashMap<usize, Sink>,
}

impl<AT: AudioType> Debug for AudioManager<AT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AudioManager {{ audio_asset: {:?}, queue: {:?} }}",
            self.audio_asset, self.queue
        )
    }
}

impl<AT: AudioType> Default for AudioManager<AT> {
    fn default() -> Self {
        let stream_handle = OutputStream::try_default().expect("Audio device not found");
        Self {
            sink_id: 0,
            audio_asset: FxHashMap::default(),
            queue: VecDeque::default(),
            _stream: stream_handle.0,
            stream_handle: stream_handle.1,
            sink: FxHashMap::default(),
        }
    }
}

impl<AT: AudioType> AudioManager<AT> {
    pub fn load(&mut self, audio_type: AT, file_path: &str) {
        let audio_asset = audio_loader::load(file_path);
        self.audio_asset.insert(
            audio_type,
            Audio {
                audio_asset,
                data: None,
            },
        );
    }

    pub fn play(&mut self, audio_type: AT) -> usize {
        let id = self.sink_id;
        self.sink_id += 1;
        self.queue.push_back((id, audio_type));

        id
    }

    fn play_queue(&mut self) {
        let len = self.queue.len();
        let mut i = 0;

        while i < len {
            let (id, audio_type) = self.queue.pop_front().unwrap();
            if let Some(audio) = self.audio_asset.get_mut(&audio_type).and_then(|a| {
                if a.data.is_some() {
                    a.data.as_ref()
                } else {
                    let audio_asset = a.audio_asset.try_recv();
                    if let Ok(audio_asset) = audio_asset {
                        a.data = Some(audio_asset.data);
                        a.data.as_ref()
                    } else {
                        None
                    }
                }
            }) {
                let sink = Sink::try_new(&self.stream_handle).unwrap();
                let decoder = rodio::Decoder::new(Cursor::new(audio.clone())).unwrap();
                sink.append(decoder);

                self.sink.insert(id, sink);
            } else {
                self.queue.push_back((id, audio_type));
            }
            i += 1;
        }
    }
}

pub fn audio_system<AT: AudioType>(mut audio_manager: UnsendableResMut<AudioManager<AT>>) {
    audio_manager.play_queue()
}
