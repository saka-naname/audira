use chrono::NaiveDateTime;
use sqlx::SqliteConnection;

use crate::models::albums::Albums;

#[derive(Debug, Clone, Default)]
pub struct AlbumsRepository;

#[derive(Debug, Clone, Copy)]
pub enum AlbumsSortKey {
    Title,
    Artist,
}

#[derive(Debug, Clone, Copy)]
pub enum AlbumsSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct ListAlbumsParams {
    pub offset: i64,
    pub page_size: i64,
    pub sort_key: AlbumsSortKey,
    pub sort_direction: AlbumsSortDirection,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ListAlbumRow {
    pub id: i64,
    pub album_artist: Option<String>,
    pub album_title: String,
    pub song_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

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

    pub async fn list_albums(
        &self,
        pool: &sqlx::SqlitePool,
        params: &ListAlbumsParams,
    ) -> Result<Vec<ListAlbumRow>, sqlx::Error> {
        let order_expression = match params.sort_key {
            AlbumsSortKey::Title => {
                "COALESCE(NULLIF(TRIM(a.album_title_sort_order), ''), a.album_title)"
            }
            AlbumsSortKey::Artist => {
                "COALESCE(NULLIF(TRIM(a.album_artist_sort_order), ''), NULLIF(TRIM(a.album_artist), ''), a.album_title)"
            }
        };
        let order_direction = match params.sort_direction {
            AlbumsSortDirection::Asc => "ASC",
            AlbumsSortDirection::Desc => "DESC",
        };
        let query = format!(
            r#"
            SELECT
                a.id,
                a.album_artist,
                a.album_title,
                COUNT(album_songs.song_id) AS song_count,
                a.created_at,
                a.updated_at
            FROM main.albums AS a
            INNER JOIN main.album_songs AS album_songs
                ON album_songs.album_id = a.id
            GROUP BY
                a.id,
                a.album_artist,
                a.album_title,
                a.album_title_sort_order,
                a.album_artist_sort_order,
                a.created_at,
                a.updated_at
            HAVING COUNT(album_songs.song_id) >= 2
            ORDER BY
                LOWER({order_expression}) {order_direction},
                a.id {order_direction}
            LIMIT ? OFFSET ?
            "#,
        );

        sqlx::query_as::<_, ListAlbumRow>(&query)
            .bind(params.page_size)
            .bind(params.offset)
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    use super::{AlbumsRepository, AlbumsSortDirection, AlbumsSortKey, ListAlbumsParams};

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");

        sqlx::query(
            r#"
            CREATE TABLE albums (
                id INTEGER NOT NULL PRIMARY KEY,
                album_artist TEXT,
                album_title TEXT NOT NULL,
                album_title_sort_order TEXT,
                album_artist_sort_order TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("create albums");

        sqlx::query(
            r#"
            CREATE TABLE songs (
                id INTEGER NOT NULL PRIMARY KEY,
                filepath TEXT NOT NULL,
                hash TEXT NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("create songs");

        sqlx::query(
            r#"
            CREATE TABLE album_songs (
                album_id INTEGER NOT NULL REFERENCES albums (id) ON DELETE CASCADE,
                song_id INTEGER NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (album_id, song_id)
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("create album_songs");

        pool
    }

    async fn insert_album(pool: &SqlitePool, id: i64, title: &str, artist: Option<&str>) {
        sqlx::query("INSERT INTO albums (id, album_title, album_artist) VALUES (?, ?, ?)")
            .bind(id)
            .bind(title)
            .bind(artist)
            .execute(pool)
            .await
            .expect("insert album");
    }

    async fn connect_songs(pool: &SqlitePool, album_id: i64, song_ids: &[i64]) {
        for song_id in song_ids {
            sqlx::query("INSERT INTO songs (id, filepath, hash) VALUES (?, ?, ?)")
                .bind(song_id)
                .bind(format!("/music/{song_id}.flac"))
                .bind(format!("hash-{song_id}"))
                .execute(pool)
                .await
                .expect("insert song");

            sqlx::query("INSERT INTO album_songs (album_id, song_id) VALUES (?, ?)")
                .bind(album_id)
                .bind(song_id)
                .execute(pool)
                .await
                .expect("connect album song");
        }
    }

    #[tokio::test]
    async fn list_albums_returns_only_albums_with_two_or_more_songs() {
        let pool = setup_pool().await;
        insert_album(&pool, 1, "One Song", Some("Artist A")).await;
        insert_album(&pool, 2, "Two Songs", Some("Artist B")).await;
        connect_songs(&pool, 1, &[1]).await;
        connect_songs(&pool, 2, &[2, 3]).await;

        let albums = AlbumsRepository::new()
            .list_albums(
                &pool,
                &ListAlbumsParams {
                    offset: 0,
                    page_size: 10,
                    sort_key: AlbumsSortKey::Title,
                    sort_direction: AlbumsSortDirection::Asc,
                },
            )
            .await
            .expect("list albums");

        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].album_title, "Two Songs");
        assert_eq!(albums[0].song_count, 2);
    }

    #[tokio::test]
    async fn list_albums_sorts_by_title_and_artist() {
        let pool = setup_pool().await;
        insert_album(&pool, 1, "Bravo", Some("Alice")).await;
        insert_album(&pool, 2, "Alpha", Some("Charlie")).await;
        connect_songs(&pool, 1, &[1, 2]).await;
        connect_songs(&pool, 2, &[3, 4]).await;

        let repository = AlbumsRepository::new();
        let by_title = repository
            .list_albums(
                &pool,
                &ListAlbumsParams {
                    offset: 0,
                    page_size: 10,
                    sort_key: AlbumsSortKey::Title,
                    sort_direction: AlbumsSortDirection::Asc,
                },
            )
            .await
            .expect("list by title");

        assert_eq!(
            by_title
                .iter()
                .map(|album| album.album_title.as_str())
                .collect::<Vec<_>>(),
            vec!["Alpha", "Bravo"]
        );

        let by_artist = repository
            .list_albums(
                &pool,
                &ListAlbumsParams {
                    offset: 0,
                    page_size: 10,
                    sort_key: AlbumsSortKey::Artist,
                    sort_direction: AlbumsSortDirection::Desc,
                },
            )
            .await
            .expect("list by artist");

        assert_eq!(
            by_artist
                .iter()
                .map(|album| album.album_artist.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("Charlie"), Some("Alice")]
        );
    }
}
