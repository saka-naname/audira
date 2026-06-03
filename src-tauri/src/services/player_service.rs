use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::{
    models::{song_metadata::SongMetadata, songs::Songs},
    repository::songs_repository::SongsRepository,
};

#[derive(Debug)]
pub enum PlayerServiceError {
    RuntimeInitializeError,
    DataNotFoundError,
    FileOpenError,
    DecodeError,
}

#[derive(Clone)]
pub enum PlayerEvent {
    TrackEnded,
}

pub struct PlayerService {
    state: Mutex<Option<PlayerRuntime>>,
    db_pool: SqlitePool,
    songs_repository: SongsRepository,
    event_tx: broadcast::Sender<PlayerEvent>,
    playback_generation: AtomicU64,
    current_generation: Arc<AtomicU64>,
}

struct PlayerRuntime {
    sink: MixerDeviceSink,
    player: Player,
}

impl PlayerService {
    pub fn new(db_pool: SqlitePool) -> Self {
        let (event_tx, _) = broadcast::channel(32);

        Self {
            state: Mutex::new(None),
            db_pool,
            songs_repository: SongsRepository::new(),
            event_tx,
            playback_generation: AtomicU64::new(0),
            current_generation: Arc::new(AtomicU64::new(0)),
        }
    }

    /// イベントのレシーバーを登録する
    pub fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
        self.event_tx.subscribe()
    }

    /// プレイヤー状態を管理するランタイムのロックを獲得する。
    /// 初期化されていなければ初期化を行う。
    fn ensure_runtime(&self) -> Result<MutexGuard<'_, Option<PlayerRuntime>>, PlayerServiceError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| PlayerServiceError::RuntimeInitializeError)?;

        if state.is_none() {
            let sink = DeviceSinkBuilder::open_default_sink()
                .map_err(|_| PlayerServiceError::RuntimeInitializeError)?;

            let player = Player::connect_new(sink.mixer());

            *state = Some(PlayerRuntime { sink, player });
        }

        Ok(state)
    }

    /// ID 指定でトラックを再生する。
    pub async fn play_track(&self, id: &i64) -> Result<(Songs, SongMetadata), PlayerServiceError> {
        let (song, song_metadata) = self
            .songs_repository
            .select_with_metadata(&self.db_pool, id)
            .await
            .map_err(|_| PlayerServiceError::DataNotFoundError)?;

        let path = &song.filepath;

        let file =
            File::open(PathBuf::from(path)).map_err(|_| PlayerServiceError::FileOpenError)?;

        let source =
            Decoder::new(BufReader::new(file)).map_err(|_| PlayerServiceError::DecodeError)?;

        let mut state = self.ensure_runtime()?;
        let runtime = state
            .as_mut()
            .ok_or(PlayerServiceError::RuntimeInitializeError)?;

        let generation = self.playback_generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.current_generation.store(generation, Ordering::SeqCst);

        let current_generation = self.current_generation.clone();
        let event_tx = self.event_tx.clone();

        runtime.player.stop();
        runtime.player.append(source);
        runtime
            .player
            .append(rodio::source::EmptyCallback::new(Box::new(move || {
                if current_generation.load(Ordering::SeqCst) == generation {
                    let _ = event_tx.send(PlayerEvent::TrackEnded);
                }
            })));

        Ok((song, song_metadata))
    }
}
