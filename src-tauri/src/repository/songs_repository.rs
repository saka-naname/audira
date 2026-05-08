use sqlx::SqliteConnection;

use crate::models::{song_metadata::SongMetadata, songs::Songs};

#[derive(Debug, Clone, Default)]
pub struct SongsRepository;

#[derive(Debug, Clone, Default)]
pub struct InsertSongsParams {
    pub filepath: String,
    pub hash: String,
    pub track_title: Option<String>,
    pub track_artist: Option<String>,
    pub track_lyricist: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub disc_number: Option<i64>,
    pub track_number: Option<i64>,
    pub track_total: Option<i64>,
    pub disc_total: Option<i64>,
    pub album_title_sort_order: Option<String>,
    pub album_artist_sort_order: Option<String>,
    pub track_title_sort_order: Option<String>,
    pub track_artist_sort_order: Option<String>,
    pub thumbnail_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InsertSongMetadataParams {
    pub recording_date: Option<String>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub audio_bitrate: Option<i64>,
    pub bit_depth: Option<i64>,
    pub channels: Option<i64>,
    pub sample_rate: Option<i64>,
    pub duration_ms: Option<i64>,
}

impl SongsRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn insert_with_metadata(
        &self,
        conn: &mut SqliteConnection,
        params: &InsertSongsParams,
        metadata: &InsertSongMetadataParams,
    ) -> Result<(Songs, SongMetadata), sqlx::Error> {
        let song = sqlx::query_as::<_, Songs>(
            r#"
            INSERT INTO main.songs (filepath, hash, track_title, track_artist, track_lyricist, album_artist, album_title, disc_number, track_number, track_total, disc_total, album_title_sort_order, album_artist_sort_order, track_title_sort_order, track_artist_sort_order, thumbnail_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#,
        )
        .bind(&params.filepath)
        .bind(&params.hash)
        .bind(&params.track_title)
        .bind(&params.track_artist)
        .bind(&params.track_lyricist)
        .bind(&params.album_artist)
        .bind(&params.album_title)
        .bind(&params.disc_number)
        .bind(&params.track_number)
        .bind(&params.track_total)
        .bind(&params.disc_total)
        .bind(&params.album_title_sort_order)
        .bind(&params.album_artist_sort_order)
        .bind(&params.track_title_sort_order)
        .bind(&params.track_artist_sort_order)
        .bind(&params.thumbnail_id)
        .fetch_one(&mut *conn)
        .await?;

        let song_metadata = sqlx::query_as::<_, SongMetadata>(
            r#"
            INSERT INTO main.song_metadata (song_id, recording_date, genre, composer, audio_bitrate, bit_depth, channels, sample_rate, duration_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
        "#,
        )
        .bind(&song.id)
        .bind(&metadata.recording_date)
        .bind(&metadata.genre)
        .bind(&metadata.composer)
        .bind(&metadata.audio_bitrate)
        .bind(&metadata.bit_depth)
        .bind(&metadata.channels)
        .bind(&metadata.sample_rate)
        .bind(&metadata.duration_ms)
        .fetch_one(&mut *conn)
        .await?;

        Ok((song, song_metadata))
    }

    pub async fn is_exist_by_hash(
        &self,
        conn: &mut SqliteConnection,
        hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("SELECT * FROM main.songs WHERE hash = ? LIMIT 1")
            .bind(hash)
            .fetch_optional(&mut *conn)
            .await;

        match result {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(err) => Err(err),
        }
    }
}
