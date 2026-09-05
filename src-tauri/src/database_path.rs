use std::path::{Path, PathBuf};

const DATABASE_FILENAME: &str = "libdata.db";

pub fn from_app_local_data_dir(app_local_data_dir: impl AsRef<Path>) -> PathBuf {
    app_local_data_dir.as_ref().join(DATABASE_FILENAME)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::from_app_local_data_dir;

    #[test]
    fn appends_database_filename() {
        assert_eq!(
            from_app_local_data_dir("app-data"),
            PathBuf::from("app-data/libdata.db")
        );
    }
}
