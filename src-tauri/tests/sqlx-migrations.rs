use std::borrow::Cow;

use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::test]
async fn thumbnail_id_migration_preserves_related_data() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");

    let initial_migrator = Migrator {
        migrations: Cow::Owned(
            MIGRATOR
                .iter()
                .filter(|migration| migration.version == 1)
                .cloned()
                .collect(),
        ),
        ..Migrator::DEFAULT
    };
    initial_migrator
        .run(&pool)
        .await
        .expect("apply initial migration");

    sqlx::query(
        "INSERT INTO thumbnails (id, filepath, original_hash) VALUES (7, '/cover.jpg', 'cover-hash')",
    )
    .execute(&pool)
    .await
    .expect("insert thumbnail");
    sqlx::query(
        "INSERT INTO songs (id, filepath, hash, album_title, thumbnail_id) VALUES (11, '/song.flac', 'song-hash', 'Album', '7')",
    )
    .execute(&pool)
    .await
    .expect("insert song");
    sqlx::query("INSERT INTO song_metadata (id, song_id, genre) VALUES (13, 11, 'Rock')")
        .execute(&pool)
        .await
        .expect("insert song metadata");
    sqlx::query("INSERT INTO albums (id, album_title) VALUES (17, 'Album')")
        .execute(&pool)
        .await
        .expect("insert album");
    sqlx::query("INSERT INTO album_songs (album_id, song_id) VALUES (17, 11)")
        .execute(&pool)
        .await
        .expect("connect album and song");

    MIGRATOR
        .run(&pool)
        .await
        .expect("apply thumbnail ID migration");

    let column_type: String = sqlx::query_scalar(
        "SELECT type FROM pragma_table_info('songs') WHERE name = 'thumbnail_id'",
    )
    .fetch_one(&pool)
    .await
    .expect("read thumbnail_id type");
    let (thumbnail_id, value_type): (i64, String) =
        sqlx::query_as("SELECT thumbnail_id, typeof(thumbnail_id) FROM songs WHERE id = 11")
            .fetch_one(&pool)
            .await
            .expect("read migrated thumbnail ID");
    let metadata_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM song_metadata WHERE id = 13 AND song_id = 11")
            .fetch_one(&pool)
            .await
            .expect("count song metadata");
    let album_song_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM album_songs WHERE album_id = 17 AND song_id = 11")
            .fetch_one(&pool)
            .await
            .expect("count album song");
    let foreign_key_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pragma_foreign_key_check")
            .fetch_one(&pool)
            .await
            .expect("check foreign keys");

    assert_eq!(column_type, "INTEGER");
    assert_eq!(thumbnail_id, 7);
    assert_eq!(value_type, "integer");
    assert_eq!(metadata_count, 1);
    assert_eq!(album_song_count, 1);
    assert_eq!(foreign_key_errors, 0);
}
