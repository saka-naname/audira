mod commands;
mod constants;
mod models;
mod repository;
mod services;
#[cfg(test)]
mod test_helpers;

use std::{str::FromStr, time::Duration};

use lofty::{
    file::{AudioFile, TaggedFileExt},
    read_from_path,
};
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use tauri::{async_runtime, Emitter, Manager};
use tauri_plugin_store::StoreExt;
use tokio::sync::broadcast;

use crate::services::{
    albums_service::AlbumsService,
    library_service::LibraryService,
    player_service::{PlayerEvent, PlayerService},
    songs_service::SongsService,
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PlayerSnapshotDto {
    status: String,
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
            let db_path = db_dir.join("libdata.db").to_string_lossy().to_string();

            let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_secs(30));

            let pool = async_runtime::block_on(async {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    // .max_connections(4)
                    .connect_with(opts)
                    .await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })?;
            app.manage(LibraryService::new(pool.clone()));
            app.manage(SongsService::new(pool.clone()));
            app.manage(AlbumsService::new(pool.clone()));
            app.manage(PlayerService::new(pool));

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
                        Ok(PlayerEvent::TrackEnded) => {
                            let _ = app_handle.emit(
                                "player://state",
                                PlayerSnapshotDto {
                                    status: String::from("idle"),
                                },
                            );

                            let _ = app_handle.emit("player://track-ended", ());
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
            commands::songs::list_songs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
