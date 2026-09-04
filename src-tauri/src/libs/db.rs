use diesel::{
    r2d2::{ConnectionManager, Pool},
    SqliteConnection,
};
use tauri::async_runtime::spawn_blocking;

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug, Clone)]
pub struct Database {
    pub sqlx_pool: sqlx::SqlitePool,
    pub diesel_pool: SqlitePool,
}

#[derive(Debug)]
pub enum DatabaseExecuteError {
    TaskJoinFailed(tauri::Error),
    GetPooledConnectionFailed(r2d2::Error),
}

impl Database {
    pub async fn execute<F, U>(&self, f: F) -> Result<U, DatabaseExecuteError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<U, DatabaseExecuteError> + Send + 'static,
        U: Send + 'static,
    {
        let pool = self.diesel_pool.clone();

        spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(DatabaseExecuteError::GetPooledConnectionFailed)?;

            f(&mut conn)
        })
        .await
        .map_err(DatabaseExecuteError::TaskJoinFailed)?
    }
}
