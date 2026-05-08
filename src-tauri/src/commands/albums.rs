use crate::services::albums_service::{
    AlbumsService, ListAlbumsError, ListAlbumsRequest, ListAlbumsResponse,
};

#[tauri::command]
pub async fn list_albums(
    service: tauri::State<'_, AlbumsService>,
    request: ListAlbumsRequest,
) -> Result<ListAlbumsResponse, String> {
    service
        .list_albums(request)
        .await
        .map_err(list_albums_error_message)
}

fn list_albums_error_message(error: ListAlbumsError) -> String {
    match error {
        ListAlbumsError::ReadFailed => String::from("アルバム一覧の取得に失敗しました。"),
    }
}
