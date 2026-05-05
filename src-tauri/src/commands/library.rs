use std::path::Path;

use tauri::Runtime;
use walkdir::WalkDir;

use crate::constants::AUDIO_EXTENSIONS;

#[tauri::command]
pub async fn scan_library<R: Runtime>(
    _app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    base_dir: &str,
) -> Result<(), String> {
    // let store = app.store(CONFIG_STORE_PATH).map_err(|e| e.to_string())?;

    // if !store.has("library.basePath") {
    //     return Err(String::from("Library is not set"));
    // };

    let base_path = Path::new(base_dir);

    if !base_path.exists() {
        return Err(String::from("指定されたパスは存在しません。"));
    }

    if !base_path.is_dir() {
        return Err(String::from("指定されたパスはディレクトリではありません。"));
    }

    let files = walk_audio_files(base_path);
    for file in files {
        // do something
    }

    Ok(())
}

fn walk_audio_files(base_path: &Path) -> impl Iterator<Item = String> {
    WalkDir::new(base_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
}
