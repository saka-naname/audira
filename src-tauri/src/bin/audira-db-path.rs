use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let local_data_dir = dirs::data_local_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve the local data directory",
        )
    })?;
    let app_local_data_dir = local_data_dir.join(&context.config().identifier);
    let database_path = audira_lib::database_path::from_app_local_data_dir(app_local_data_dir);

    println!("{}", database_path.display());

    Ok(())
}
