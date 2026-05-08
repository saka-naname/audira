use sqlx::SqlitePool;
use tauri::Runtime;

use crate::services::library_service::{self, LibraryScanError};

#[tauri::command]
pub async fn scan_library<R: Runtime>(
    _app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    pool: tauri::State<'_, SqlitePool>,
    base_dir: &str,
) -> Result<(), String> {
    library_service::scan_library(&pool, base_dir)
        .await
        .map_err(scan_error_message)
}

fn scan_error_message(error: LibraryScanError) -> String {
    match error {
        LibraryScanError::BasePathNotFound => String::from("指定されたパスは存在しません。"),
        LibraryScanError::BasePathIsNotDirectory => {
            String::from("指定されたパスはディレクトリではありません。")
        }
        LibraryScanError::BeginTransactionFailed => {
            String::from("トランザクションの生成に失敗しました")
        }
        LibraryScanError::WriteTransactionFailed => {
            String::from("トランザクションの書き込み中に問題が発生しました")
        }
        LibraryScanError::CommitTransactionFailed => {
            String::from("トランザクションの実行中に問題が発生しました")
        }
        LibraryScanError::FindAlbumFailed => String::from("アルバムの検索に失敗しました"),
        LibraryScanError::CreateAlbumFailed => String::from("アルバムの作成に失敗しました"),
        LibraryScanError::ConnectAlbumFailed => String::from("アルバムへの登録に失敗しました"),
    }
}
