use sqlx::SqliteConnection;

use crate::models::albums::Albums;

#[derive(Debug, Clone, Default)]
pub struct AlbumsRepository;

#[derive(Debug, Clone, Default)]
pub struct InsertAlbumsParams {
    pub album_title: String,
    pub album_artist: Option<String>,
    pub album_title_sort_order: Option<String>,
    pub album_artist_sort_order: Option<String>,
}

impl AlbumsRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_album_by_params(
        &self,
        conn: &mut SqliteConnection,
        album_title: &str,
        album_artist: Option<&str>,
        album_title_sort_order: Option<&str>,
        album_artist_sort_order: Option<&str>,
    ) -> Result<Option<Albums>, sqlx::Error> {
        sqlx::query_as::<_, Albums>(
            r#"
        SELECT * FROM main.albums WHERE album_title = ?
                                    AND album_artist IS NOT DISTINCT FROM ?
                                    AND album_title_sort_order IS NOT DISTINCT FROM ?
                                    AND album_artist_sort_order IS NOT DISTINCT FROM ?
                                    LIMIT 1
                                    "#,
        )
        .bind(album_title)
        .bind(album_artist)
        .bind(album_title_sort_order)
        .bind(album_artist_sort_order)
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn connect_album_and_song(
        &self,
        conn: &mut SqliteConnection,
        album_id: &i64,
        song_id: &i64,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO album_songs (album_id, song_id) VALUES (?, ?) RETURNING *",
        )
        .bind(album_id)
        .bind(song_id)
        .fetch_one(&mut *conn)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn insert_album(
        &self,
        conn: &mut SqliteConnection,
        params: &InsertAlbumsParams,
    ) -> Result<Albums, sqlx::Error> {
        sqlx::query_as::<_, Albums>(
            "INSERT INTO main.albums (album_title, album_artist, album_title_sort_order, album_artist_sort_order) VALUES (?, ?, ?, ?) RETURNING *",
        )
        .bind(&params.album_title)
        .bind(&params.album_artist)
        .bind(&params.album_title_sort_order)
        .bind(&params.album_artist_sort_order)
        .fetch_one(&mut *conn)
        .await
    }
}
