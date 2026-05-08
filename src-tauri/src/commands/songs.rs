use crate::services::songs_service::{
    ListSongsError, ListSongsRequest, ListSongsResponse, SongsService,
};

#[tauri::command]
pub async fn list_songs(
    service: tauri::State<'_, SongsService>,
    request: ListSongsRequest,
) -> Result<ListSongsResponse, String> {
    service
        .list_songs(request)
        .await
        .map_err(list_songs_error_message)
}

fn list_songs_error_message(error: ListSongsError) -> String {
    match error {
        ListSongsError::ReadFailed => String::from("楽曲一覧の取得に失敗しました。"),
    }
}
