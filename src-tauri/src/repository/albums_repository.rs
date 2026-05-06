use sqlx::SqliteConnection;

use crate::models::albums::Albums;

#[derive(Debug, Clone, Default)]
pub struct InsertAlbumsParams {
    pub album_title: String,
    pub album_artist: Option<String>,
}

pub async fn find_album_by_title_and_artist(
    conn: &mut SqliteConnection,
    album_title: &str,
    album_artist: Option<&str>,
) -> Result<Option<Albums>, sqlx::Error> {
    let result = sqlx::query_as::<_, Albums>(
        "SELECT * FROM main.albums WHERE album_title = ? AND album_artist IS NOT DISTINCT FROM ? LIMIT 1",
    )
    .bind(album_title)
    .bind(album_artist)
    .fetch_optional(&mut *conn)
    .await;

    result
}

pub async fn connect_album_and_song(
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
    conn: &mut SqliteConnection,
    params: &InsertAlbumsParams,
) -> Result<Albums, sqlx::Error> {
    let album = sqlx::query_as::<_, Albums>(
        "INSERT INTO main.albums (album_title, album_artist) VALUES (?, ?) RETURNING *",
    )
    .bind(&params.album_title)
    .bind(&params.album_artist)
    .fetch_one(&mut *conn)
    .await;

    if let Err(e) = album {
        panic!("{e}");
    }

    album
}
