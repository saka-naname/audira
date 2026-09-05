use std::time::Duration;

use diesel::{
    connection::SimpleConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sql_query, QueryableByName, SqliteConnection,
};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tempfile::TempDir;
use tokio::{sync::oneshot, time::timeout};

use crate::{
    schema::{album_songs, albums, song_metadata, songs, thumbnails},
    SqliteConnectionCustomizer,
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

type DieselPool = Pool<ConnectionManager<SqliteConnection>>;

struct TestDatabase {
    _temp_dir: TempDir,
    sqlx_pool: SqlitePool,
    diesel_pool: DieselPool,
}

#[derive(Debug, QueryableByName)]
struct TextPragma {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

#[derive(Debug, QueryableByName)]
struct IntegerPragma {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    value: i64,
}

async fn setup_database() -> TestDatabase {
    let temp_dir = tempfile::tempdir().expect("create temporary database directory");
    let db_path = temp_dir.path().join("sqlx-diesel-compatibility.db");
    let sqlx_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(2));
    let sqlx_pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(sqlx_options)
        .await
        .expect("connect SQLx pool to temporary database");

    MIGRATOR
        .run(&sqlx_pool)
        .await
        .expect("create schema with SQLx migrations");

    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_string_lossy());
    let diesel_pool = Pool::builder()
        .max_size(2)
        .min_idle(Some(1))
        .connection_timeout(Duration::from_secs(2))
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .expect("connect Diesel pool to SQLx-created database");

    TestDatabase {
        _temp_dir: temp_dir,
        sqlx_pool,
        diesel_pool,
    }
}

#[tokio::test]
async fn diesel_can_read_and_write_the_schema_created_by_sqlx() {
    let db = setup_database().await;

    let mut transaction = db.sqlx_pool.begin().await.expect("begin SQLx transaction");
    sqlx::query(
        "INSERT INTO thumbnails (id, filepath, original_hash) VALUES (7, '/sqlx-cover.jpg', 'sqlx-cover-hash')",
    )
    .execute(&mut *transaction)
    .await
    .expect("insert thumbnail with SQLx");
    sqlx::query(
        "INSERT INTO songs (id, filepath, hash, track_title, thumbnail_id) VALUES (11, '/sqlx-song.flac', 'sqlx-song-hash', 'SQLx song', 7)",
    )
    .execute(&mut *transaction)
    .await
    .expect("insert song with SQLx");
    sqlx::query("INSERT INTO song_metadata (id, song_id, genre) VALUES (13, 11, 'Rock')")
        .execute(&mut *transaction)
        .await
        .expect("insert song metadata with SQLx");
    sqlx::query(
        "INSERT INTO albums (id, album_artist, album_title) VALUES (17, 'Artist', 'Album')",
    )
    .execute(&mut *transaction)
    .await
    .expect("insert album with SQLx");
    sqlx::query("INSERT INTO album_songs (album_id, song_id) VALUES (17, 11)")
        .execute(&mut *transaction)
        .await
        .expect("connect album and song with SQLx");
    transaction.commit().await.expect("commit SQLx inserts");

    let mut diesel_connection = db
        .diesel_pool
        .get()
        .expect("get Diesel connection for compatibility test");
    let sqlx_song = songs::table
        .find(11_i64)
        .select((songs::filepath, songs::track_title, songs::thumbnail_id))
        .first::<(String, Option<String>, Option<i64>)>(&mut diesel_connection)
        .expect("read SQLx song with Diesel");
    let sqlx_thumbnail = thumbnails::table
        .find(7_i64)
        .select(thumbnails::filepath)
        .first::<String>(&mut diesel_connection)
        .expect("read SQLx thumbnail with Diesel");
    let sqlx_metadata = song_metadata::table
        .filter(song_metadata::song_id.eq(11_i64))
        .select(song_metadata::genre)
        .first::<Option<String>>(&mut diesel_connection)
        .expect("read SQLx song metadata with Diesel");
    let sqlx_album = albums::table
        .find(17_i64)
        .select(albums::album_title)
        .first::<String>(&mut diesel_connection)
        .expect("read SQLx album with Diesel");
    let sqlx_album_song = album_songs::table
        .find((17_i64, 11_i64))
        .select((album_songs::album_id, album_songs::song_id))
        .first::<(i64, i64)>(&mut diesel_connection)
        .expect("read SQLx album relation with Diesel");

    assert_eq!(
        sqlx_song,
        (
            "/sqlx-song.flac".to_string(),
            Some("SQLx song".to_string()),
            Some(7),
        )
    );
    assert_eq!(sqlx_thumbnail, "/sqlx-cover.jpg");
    assert_eq!(sqlx_metadata.as_deref(), Some("Rock"));
    assert_eq!(sqlx_album, "Album");
    assert_eq!(sqlx_album_song, (17, 11));

    diesel_connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            diesel::insert_into(thumbnails::table)
                .values((
                    thumbnails::id.eq(27_i64),
                    thumbnails::filepath.eq("/diesel-cover.jpg"),
                    thumbnails::original_hash.eq("diesel-cover-hash"),
                ))
                .execute(connection)?;
            diesel::insert_into(songs::table)
                .values((
                    songs::id.eq(31_i64),
                    songs::filepath.eq("/diesel-song.flac"),
                    songs::hash.eq("diesel-song-hash"),
                    songs::track_title.eq(Some("Diesel song")),
                    songs::thumbnail_id.eq(Some(27_i64)),
                ))
                .execute(connection)?;
            diesel::insert_into(song_metadata::table)
                .values((
                    song_metadata::id.eq(33_i64),
                    song_metadata::song_id.eq(31_i64),
                    song_metadata::genre.eq(Some("Jazz")),
                ))
                .execute(connection)?;
            diesel::insert_into(albums::table)
                .values((
                    albums::id.eq(37_i64),
                    albums::album_artist.eq(Some("Diesel artist")),
                    albums::album_title.eq("Diesel album"),
                ))
                .execute(connection)?;
            diesel::insert_into(album_songs::table)
                .values((
                    album_songs::album_id.eq(37_i64),
                    album_songs::song_id.eq(31_i64),
                ))
                .execute(connection)?;

            Ok(())
        })
        .expect("write all tables with Diesel");

    let diesel_song: (String, Option<String>, Option<i64>) =
        sqlx::query_as("SELECT filepath, track_title, thumbnail_id FROM songs WHERE id = 31")
            .fetch_one(&db.sqlx_pool)
            .await
            .expect("read Diesel song with SQLx");
    let diesel_related_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM song_metadata AS metadata INNER JOIN album_songs AS relation ON relation.song_id = metadata.song_id INNER JOIN albums AS album ON album.id = relation.album_id INNER JOIN thumbnails AS thumbnail ON thumbnail.id = (SELECT thumbnail_id FROM songs WHERE id = metadata.song_id) WHERE metadata.song_id = 31 AND metadata.genre = 'Jazz' AND album.album_title = 'Diesel album' AND thumbnail.filepath = '/diesel-cover.jpg'",
    )
    .fetch_one(&db.sqlx_pool)
    .await
    .expect("read Diesel relations with SQLx");

    assert_eq!(
        diesel_song,
        (
            "/diesel-song.flac".to_string(),
            Some("Diesel song".to_string()),
            Some(27),
        )
    );
    assert_eq!(diesel_related_rows, 1);

    diesel::update(songs::table.find(31_i64))
        .set(songs::track_title.eq(Some("Updated by Diesel")))
        .execute(&mut diesel_connection)
        .expect("update song with Diesel");
    let updated_title: Option<String> =
        sqlx::query_scalar("SELECT track_title FROM songs WHERE id = 31")
            .fetch_one(&db.sqlx_pool)
            .await
            .expect("read Diesel update with SQLx");
    assert_eq!(updated_title.as_deref(), Some("Updated by Diesel"));

    let invalid_foreign_key = diesel::insert_into(song_metadata::table)
        .values((
            song_metadata::id.eq(34_i64),
            song_metadata::song_id.eq(999_i64),
        ))
        .execute(&mut diesel_connection)
        .expect_err("reject invalid foreign key through Diesel");
    assert!(matches!(
        invalid_foreign_key,
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _
        )
    ));

    diesel::delete(songs::table.find(31_i64))
        .execute(&mut diesel_connection)
        .expect("delete song with Diesel");
    let remaining_child_rows: i64 = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM song_metadata WHERE song_id = 31) + (SELECT COUNT(*) FROM album_songs WHERE song_id = 31)",
    )
    .fetch_one(&db.sqlx_pool)
    .await
    .expect("check cascades with SQLx");
    assert_eq!(remaining_child_rows, 0);
}

#[tokio::test]
async fn sqlx_and_diesel_connections_can_share_wal_and_serialize_writes() {
    let db = setup_database().await;
    let mut sqlx_connection = db
        .sqlx_pool
        .acquire()
        .await
        .expect("hold SQLx connection open");
    let mut diesel_connection = db.diesel_pool.get().expect("hold Diesel connection open");

    let sqlx_journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&mut *sqlx_connection)
        .await
        .expect("read SQLx journal mode");
    let sqlx_foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *sqlx_connection)
        .await
        .expect("read SQLx foreign key setting");
    let diesel_journal_mode = sql_query("SELECT journal_mode AS value FROM pragma_journal_mode")
        .get_result::<TextPragma>(&mut diesel_connection)
        .expect("read Diesel journal mode");
    let diesel_foreign_keys = sql_query("SELECT foreign_keys AS value FROM pragma_foreign_keys")
        .get_result::<IntegerPragma>(&mut diesel_connection)
        .expect("read Diesel foreign key setting");

    assert_eq!(sqlx_journal_mode, "wal");
    assert_eq!(sqlx_foreign_keys, 1);
    assert_eq!(diesel_journal_mode.value, "wal");
    assert_eq!(diesel_foreign_keys.value, 1);

    sqlx::query(
        "INSERT INTO songs (id, filepath, hash) VALUES (101, '/committed.flac', 'committed')",
    )
    .execute(&mut *sqlx_connection)
    .await
    .expect("insert committed SQLx row");
    let visible_to_diesel = songs::table
        .find(101_i64)
        .select(songs::filepath)
        .first::<String>(&mut diesel_connection)
        .expect("read committed SQLx row with Diesel");
    assert_eq!(visible_to_diesel, "/committed.flac");

    diesel_connection
        .batch_execute(
            "BEGIN IMMEDIATE; INSERT INTO songs (id, filepath, hash) VALUES (102, '/uncommitted.flac', 'uncommitted');",
        )
        .expect("start Diesel write transaction");
    let visible_during_diesel_write: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM songs WHERE id IN (101, 102)")
            .fetch_one(&mut *sqlx_connection)
            .await
            .expect("read with SQLx during Diesel write transaction");
    assert_eq!(visible_during_diesel_write, 1);
    diesel_connection
        .batch_execute("COMMIT")
        .expect("commit Diesel write transaction");

    let visible_after_diesel_commit: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM songs WHERE id IN (101, 102)")
            .fetch_one(&mut *sqlx_connection)
            .await
            .expect("read Diesel commit with SQLx");
    assert_eq!(visible_after_diesel_commit, 2);

    diesel_connection
        .batch_execute(
            "BEGIN IMMEDIATE; INSERT INTO songs (id, filepath, hash) VALUES (103, '/diesel-writer.flac', 'diesel-writer');",
        )
        .expect("hold Diesel writer lock");

    let sqlx_pool = db.sqlx_pool.clone();
    let (started_sender, started_receiver) = oneshot::channel();
    let mut competing_writer = tokio::spawn(async move {
        let _ = started_sender.send(());
        sqlx::query(
            "INSERT INTO songs (id, filepath, hash) VALUES (104, '/sqlx-writer.flac', 'sqlx-writer')",
        )
        .execute(&sqlx_pool)
        .await
    });
    started_receiver.await.expect("start competing SQLx writer");

    assert!(
        timeout(Duration::from_millis(100), &mut competing_writer)
            .await
            .is_err(),
        "SQLx writer should wait while Diesel holds the writer lock"
    );

    diesel_connection
        .batch_execute("COMMIT")
        .expect("release Diesel writer lock");
    timeout(Duration::from_secs(2), competing_writer)
        .await
        .expect("SQLx writer should finish within busy timeout")
        .expect("join competing SQLx writer")
        .expect("serialize SQLx write after Diesel commit");

    let serialized_write_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM songs WHERE id IN (103, 104)")
            .fetch_one(&mut *sqlx_connection)
            .await
            .expect("read serialized writes");
    assert_eq!(serialized_write_count, 2);
}
