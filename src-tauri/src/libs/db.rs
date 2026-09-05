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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use diesel::{
        connection::SimpleConnection,
        r2d2::{ConnectionManager, Pool},
        sql_query, Connection, QueryableByName, RunQueryDsl, SqliteConnection,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    use super::{Database, DatabaseExecuteError};

    #[derive(Debug, QueryableByName)]
    struct RowCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    fn setup_database(connection_timeout: Duration) -> Database {
        // TODO: #38 で sqlx の初期化処理を外す
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let diesel_pool = Pool::builder()
            .max_size(1)
            .connection_timeout(connection_timeout)
            .build(manager)
            .expect("create in-memory Diesel connection pool");
        let sqlx_pool = SqlitePoolOptions::new()
            .connect_lazy("sqlite::memory:")
            .expect("create lazy in-memory SQLx connection pool");

        Database {
            sqlx_pool,
            diesel_pool,
        }
    }

    async fn create_test_table(db: &Database) {
        db.execute(|conn| {
            conn.batch_execute(
                "CREATE TABLE test_items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            )
            .expect("create test table");

            Ok(())
        })
        .await
        .expect("execute table creation");
    }

    async fn count_test_items(db: &Database) -> i64 {
        db.execute(|conn| {
            let row = sql_query("SELECT COUNT(*) AS count FROM test_items")
                .get_result::<RowCount>(conn)
                .expect("count test items");

            Ok(row.count)
        })
        .await
        .expect("execute item count")
    }

    #[tokio::test]
    async fn execute_runs_single_database_operation() {
        let db = setup_database(Duration::from_secs(1));
        create_test_table(&db).await;

        let affected_rows = db
            .execute(|conn| {
                let affected_rows =
                    sql_query("INSERT INTO test_items (name) VALUES ('single operation')")
                        .execute(conn)
                        .expect("insert test item");

                Ok(affected_rows)
            })
            .await
            .expect("execute insert");

        assert_eq!(affected_rows, 1);
        assert_eq!(count_test_items(&db).await, 1);
    }

    #[tokio::test]
    async fn execute_commits_transaction() {
        let db = setup_database(Duration::from_secs(1));
        create_test_table(&db).await;

        db.execute(|conn| {
            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                sql_query("INSERT INTO test_items (name) VALUES ('first')").execute(conn)?;
                sql_query("INSERT INTO test_items (name) VALUES ('second')").execute(conn)?;

                Ok(())
            })
            .expect("commit transaction");

            Ok(())
        })
        .await
        .expect("execute transaction");

        assert_eq!(count_test_items(&db).await, 2);
    }

    #[tokio::test]
    async fn execute_rolls_back_failed_transaction() {
        let db = setup_database(Duration::from_secs(1));
        create_test_table(&db).await;

        let transaction_result = db
            .execute(|conn| {
                let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                    sql_query("INSERT INTO test_items (id, name) VALUES (1, 'first')")
                        .execute(conn)?;
                    sql_query("INSERT INTO test_items (id, name) VALUES (1, 'duplicate')")
                        .execute(conn)?;

                    Ok(())
                });

                Ok(transaction_result)
            })
            .await
            .expect("execute transaction");

        assert!(matches!(
            transaction_result,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            ))
        ));
        assert_eq!(count_test_items(&db).await, 0);
    }

    #[tokio::test]
    async fn execute_returns_get_pooled_connection_failed() {
        let db = setup_database(Duration::from_millis(10));
        let _held_connection = db
            .diesel_pool
            .get()
            .expect("hold the only pooled connection");

        let error = db
            .execute(|_| Ok(()))
            .await
            .expect_err("pool exhaustion should fail");

        assert!(matches!(
            error,
            DatabaseExecuteError::GetPooledConnectionFailed(_)
        ));
    }

    #[tokio::test]
    async fn execute_returns_task_join_failed_when_task_panics() {
        let db = setup_database(Duration::from_secs(1));

        let error = db
            .execute(|_| -> Result<(), DatabaseExecuteError> {
                panic!("simulate blocking task failure");
            })
            .await
            .expect_err("panicked blocking task should fail to join");

        assert!(matches!(error, DatabaseExecuteError::TaskJoinFailed(_)));
    }
}
