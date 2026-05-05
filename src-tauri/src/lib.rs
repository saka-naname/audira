mod commands;
mod constants;
mod schema;

use lofty::{
    file::{AudioFile, TaggedFileExt},
    read_from_path,
};
use tauri::{async_runtime, Manager};
use tauri_plugin_store::StoreExt;

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
            let db_path = app.path().app_local_data_dir()?;
            let db_path = db_path.join("libdata.db").to_string_lossy().to_string();
            let pool = async_runtime::block_on(async {
                let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })?;
            app.manage(pool);

            // Initialize config store
            let store = app.store("config.json")?;
            store.close_resource();
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            dump_metadata,
            commands::library::scan_library
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
