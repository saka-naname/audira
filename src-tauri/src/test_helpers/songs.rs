use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub(crate) async fn setup_songs_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");

    sqlx::query(
        r#"
        CREATE TABLE songs (
            id INTEGER NOT NULL PRIMARY KEY,
            filepath TEXT NOT NULL,
            hash TEXT NOT NULL,
            track_title TEXT,
            track_artist TEXT,
            track_lyricist TEXT,
            album_artist TEXT,
            album_title TEXT,
            disc_number INTEGER,
            track_number INTEGER,
            track_total INTEGER,
            disc_total INTEGER,
            album_title_sort_order TEXT,
            album_artist_sort_order TEXT,
            track_title_sort_order TEXT,
            track_artist_sort_order TEXT,
            thumbnail_id TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("create songs");

    sqlx::query(
        r#"
        CREATE TABLE song_metadata (
            id INTEGER NOT NULL PRIMARY KEY,
            song_id INTEGER NOT NULL UNIQUE REFERENCES songs (id) ON DELETE CASCADE,
            recording_date TEXT,
            genre TEXT,
            composer TEXT,
            audio_bitrate INTEGER,
            bit_depth INTEGER,
            channels INTEGER,
            sample_rate INTEGER,
            duration_ms INTEGER,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("create song_metadata");

    pool
}

pub(crate) async fn insert_song_with_metadata(pool: &SqlitePool, id: i64) {
    sqlx::query(
        r#"
        INSERT INTO songs (
            id,
            filepath,
            hash,
            track_title,
            track_artist,
            album_artist,
            album_title,
            disc_number,
            track_number
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(format!("/music/song-{id}.flac"))
    .bind(format!("hash-{id}"))
    .bind("Track Title")
    .bind("Track Artist")
    .bind("Album Artist")
    .bind("Album Title")
    .bind(1_i64)
    .bind(2_i64)
    .execute(pool)
    .await
    .expect("insert song");

    sqlx::query(
        r#"
        INSERT INTO song_metadata (
            song_id,
            recording_date,
            genre,
            composer,
            audio_bitrate,
            bit_depth,
            channels,
            sample_rate,
            duration_ms
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind("2026")
    .bind("Rock")
    .bind("Composer")
    .bind(320000_i64)
    .bind(16_i64)
    .bind(2_i64)
    .bind(44100_i64)
    .bind(180000_i64)
    .execute(pool)
    .await
    .expect("insert song_metadata");
}
