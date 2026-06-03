use crate::services::player_service::{PlayerService, PlayerServiceError};

#[tauri::command]
pub async fn play_track(service: tauri::State<'_, PlayerService>, id: i64) -> Result<(), String> {
    service
        .play_track(&id)
        .await
        .map_err(service_error_message)?;

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
