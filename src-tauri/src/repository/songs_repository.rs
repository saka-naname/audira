use chrono::NaiveDateTime;
use sqlx::SqliteConnection;

use crate::models::{song_metadata::SongMetadata, songs::Songs};

#[derive(Debug, Clone, Default)]
pub struct SongsRepository;

#[derive(Debug, Clone, Copy)]
pub enum SongsSortKey {
    Title,
    Artist,
}

#[derive(Debug, Clone, Copy)]
pub enum SongsSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct ListSongsParams {
    pub offset: i64,
    pub page_size: i64,
    pub sort_key: SongsSortKey,
    pub sort_direction: SongsSortDirection,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ListSongRow {
    pub id: i64,
    pub filepath: String,
    pub track_title: Option<String>,
    pub track_artist: Option<String>,
    pub track_lyricist: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub disc_number: Option<i64>,
    pub track_number: Option<i64>,
    pub track_total: Option<i64>,
    pub disc_total: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

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
    pub thumbnail_id: Option<i64>,
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

    pub async fn select_with_metadata(
        &self,
        pool: &sqlx::SqlitePool,
        id: &i64,
    ) -> Result<(Songs, SongMetadata), sqlx::Error> {
        let song = sqlx::query_as::<_, Songs>("SELECT * FROM main.songs WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        let song_metadata = sqlx::query_as::<_, SongMetadata>(
            "SELECT * FROM main.song_metadata WHERE song_id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok((song, song_metadata))
    }

    pub async fn list_songs(
        &self,
        pool: &sqlx::SqlitePool,
        params: &ListSongsParams,
    ) -> Result<Vec<ListSongRow>, sqlx::Error> {
        let order_expression = match params.sort_key {
            SongsSortKey::Title => {
                "COALESCE(NULLIF(TRIM(s.track_title_sort_order), ''), NULLIF(TRIM(s.track_title), ''), s.filepath)"
            }
            SongsSortKey::Artist => {
                "COALESCE(NULLIF(TRIM(s.track_artist_sort_order), ''), NULLIF(TRIM(s.track_artist), ''), s.filepath)"
            }
        };
        let order_direction = match params.sort_direction {
            SongsSortDirection::Asc => "ASC",
            SongsSortDirection::Desc => "DESC",
        };
        let query = format!(
            r#"
            SELECT
                s.id,
                s.filepath,
                s.track_title,
                s.track_artist,
                s.track_lyricist,
                s.album_artist,
                s.album_title,
                s.disc_number,
                s.track_number,
                s.track_total,
                s.disc_total,
                s.created_at,
                s.updated_at
            FROM main.songs AS s
            ORDER BY
                LOWER({order_expression}) {order_direction},
                s.id {order_direction}
            LIMIT ? OFFSET ?
            "#,
        );

        sqlx::query_as::<_, ListSongRow>(&query)
            .bind(params.page_size)
            .bind(params.offset)
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::SongsRepository;
    use crate::test_helpers::songs::{insert_song_with_metadata, setup_songs_pool};

    #[tokio::test]
    async fn select_with_metadata_returns_existing_song_and_metadata() {
        let pool = setup_songs_pool().await;
        insert_song_with_metadata(&pool, 1).await;

        let (song, metadata) = SongsRepository::new()
            .select_with_metadata(&pool, &1)
            .await
            .expect("select song with metadata");

        assert_eq!(song.id, 1);
        assert_eq!(song.filepath, "/music/song-1.flac");
        assert_eq!(song.hash, "hash-1");
        assert_eq!(song.track_title.as_deref(), Some("Track Title"));
        assert_eq!(song.track_artist.as_deref(), Some("Track Artist"));
        assert_eq!(song.album_artist.as_deref(), Some("Album Artist"));
        assert_eq!(song.album_title.as_deref(), Some("Album Title"));
        assert_eq!(song.disc_number, Some(1));
        assert_eq!(song.track_number, Some(2));
        assert_eq!(metadata.song_id, 1);
        assert_eq!(metadata.recording_date.as_deref(), Some("2026"));
        assert_eq!(metadata.genre.as_deref(), Some("Rock"));
        assert_eq!(metadata.composer.as_deref(), Some("Composer"));
        assert_eq!(metadata.audio_bitrate, Some(320000));
        assert_eq!(metadata.bit_depth, Some(16));
        assert_eq!(metadata.channels, Some(2));
        assert_eq!(metadata.sample_rate, Some(44100));
        assert_eq!(metadata.duration_ms, Some(180000));
    }

    #[tokio::test]
    async fn select_with_metadata_returns_sqlx_error_for_missing_id() {
        let pool = setup_songs_pool().await;

        let error = SongsRepository::new()
            .select_with_metadata(&pool, &999)
            .await
            .expect_err("missing song should return sqlx error");

        assert!(matches!(error, sqlx::Error::RowNotFound));
    }
}
