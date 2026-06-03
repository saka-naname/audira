use serde::Serialize;
use tauri::Emitter;

use crate::services::player_service::{PlayerService, PlayerServiceError};

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

#[tauri::command]
pub async fn play_track(
    app: tauri::AppHandle,
    service: tauri::State<'_, PlayerService>,
    id: i64,
) -> Result<(), String> {
    let (song, _) = service
        .play_track(&id)
        .await
        .map_err(service_error_message)?;

    app.emit(
        "player://track",
        TrackDto {
            id: song.id,
            filepath: song.filepath,
            track_title: song.track_title,
            track_artist: song.track_artist,
            album_title: song.album_title,
        },
    )
    .map_err(|_| String::from("楽曲情報の取得に失敗しました"))?;

    app.emit(
        "player://state",
        PlayerSnapshotDto {
            status: String::from("playing"),
        },
    )
    .map_err(|_| String::from("プレイヤー状態の取得に失敗しました"))?;

    Ok(())
}

fn service_error_message(error: PlayerServiceError) -> String {
    match error {
        PlayerServiceError::RuntimeInitializeError => {
            String::from("プレイヤーの初期化に失敗しました")
        }
        PlayerServiceError::DataNotFoundError => String::from("曲が見つかりませんでした"),
        PlayerServiceError::FileOpenError => String::from("ファイルを開けませんでした"),
        PlayerServiceError::DecodeError => String::from("ファイルのデコードに失敗しました"),
    }
}
