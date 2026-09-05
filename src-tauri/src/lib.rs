mod commands;
mod constants;
pub mod database_path;
mod libs;
mod models;
mod repository;
mod schema;
mod services;
#[cfg(test)]
mod test_helpers;

use std::{str::FromStr, time::Duration};

use diesel::{
    connection::SimpleConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Pool},
    SqliteConnection,
};
use lofty::{
    file::{AudioFile, TaggedFileExt},
    read_from_path,
};
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use tauri::{async_runtime, Emitter, Manager};
use tauri_plugin_store::StoreExt;
use tokio::sync::broadcast;

use crate::{
    libs::db::Database,
    services::{
        albums_service::AlbumsService,
        library_service::LibraryService,
        player_service::{PlayerEvent, PlayerService},
        songs_service::SongsService,
    },
};

#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute("PRAGMA busy_timeout = 5000;")?;
        conn.batch_execute("PRAGMA journal_mode = WAL;")?;
        conn.batch_execute("PRAGMA synchronous = NORMAL")?;
        conn.batch_execute("PRAGMA foreign_keys = ON;")?;

        Ok(())
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PlayerSnapshotDto {
    status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TrackDto {
    id: i64,
    filepath: String,
    track_title: Option<String>,
    track_artist: Option<String>,
    album_title: Option<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn dump_metadata(path: &str) -> Result<(), String> {
    let tagged_file = read_from_path(path).expect("");

    let prop = tagged_file.properties();
    println!("Properties:");
    println!("\tBitrate: {:?}", prop.audio_bitrate());
    println!("\tBitdepth: {:?}", prop.bit_depth());
    println!("\tChannels: {:?}", prop.channels());
    println!("\tDuration: {:?}", prop.duration());
    println!("\tOA Bitrate: {:?}", prop.overall_bitrate());
    println!("\tSample Rate: {:?}", prop.sample_rate());

    if let Some(id3v2) = tagged_file.primary_tag() {
        println!("Primary:");
        for item in id3v2.items() {
            let (k, v) = item.clone().consume();
            println!("\t{:?}: {:?}", k, v);
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Initialize database
            let db_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&db_dir)?;
            let db_path = database_path::from_app_local_data_dir(db_dir)
                .to_string_lossy()
                .to_string();

            let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_secs(30));

            let sqlx_pool = async_runtime::block_on(async {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    // .max_connections(4)
                    .connect_with(opts)
                    .await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })?;

            let diesel_pool = async_runtime::block_on(async {
                let manager = ConnectionManager::<SqliteConnection>::new(db_path);
                let pool = Pool::builder()
                    .connection_timeout(Duration::from_secs(30))
                    .max_size(4)
                    .min_idle(Some(1))
                    .connection_customizer(Box::new(SqliteConnectionCustomizer))
                    .build(manager)
                    .expect("Failed to create Diesel connection pool");
                Ok::<_, diesel::r2d2::PoolError>(pool)
            })?;

            let db = Database {
                sqlx_pool,
                diesel_pool,
            };

            app.manage(LibraryService::new(db.clone()));
            app.manage(SongsService::new(db.clone()));
            app.manage(AlbumsService::new(db.clone()));
            app.manage(PlayerService::new(db.clone()));

            // Initialize config store
            let store = app.store("config.json")?;
            store.close_resource();

            // Initialize event loop
            let app_handle = app.handle().clone();
            let player_service = app.state::<PlayerService>();
            let mut event_rx = player_service.subscribe();

            tauri::async_runtime::spawn(async move {
                loop {
                    match event_rx.recv().await {
                        Ok(PlayerEvent::TrackStarted { song }) => {
                            let _ = app_handle.emit(
                                "player://track",
                                TrackDto {
                                    id: song.id,
                                    filepath: song.filepath,
                                    track_title: song.track_title,
                                    track_artist: song.track_artist,
                                    album_title: song.album_title,
                                },
                            );

                            let _ = app_handle.emit(
                                "player://state",
                                PlayerSnapshotDto {
                                    status: String::from("playing"),
                                },
                            );
                        }
                        Ok(PlayerEvent::TrackEnded) => {
                            let _ = app_handle.emit(
                                "player://state",
                                PlayerSnapshotDto {
                                    status: String::from("idle"),
                                },
                            );

                            let _ = app_handle.emit("player://track-ended", ());
                        }
                        Ok(PlayerEvent::TrackPaused) => {
                            let _ = app_handle.emit(
                                "player://state",
                                PlayerSnapshotDto {
                                    status: String::from("paused"),
                                },
                            );
                        }
                        Ok(PlayerEvent::TrackResumed) => {
                            let _ = app_handle.emit(
                                "player://state",
                                PlayerSnapshotDto {
                                    status: String::from("playing"),
                                },
                            );
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // TODO: プレイヤーの最新の状態を再取得する処理を作成する
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            dump_metadata,
            commands::albums::list_albums,
            commands::library::scan_library,
            commands::player::play_track,
            commands::player::pause,
            commands::player::resume,
            commands::songs::list_songs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
