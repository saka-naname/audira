use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    libs::db::Database,
    repository::songs_repository::{
        ListSongsParams, SongsRepository, SongsSortDirection, SongsSortKey,
    },
};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SongSortBy {
    Title,
    Artist,
}

impl From<SongSortBy> for SongsSortKey {
    fn from(value: SongSortBy) -> Self {
        match value {
            SongSortBy::Title => SongsSortKey::Title,
            SongSortBy::Artist => SongsSortKey::Artist,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SongSortOrder {
    Asc,
    Desc,
}

impl From<SongSortOrder> for SongsSortDirection {
    fn from(value: SongSortOrder) -> Self {
        match value {
            SongSortOrder::Asc => SongsSortDirection::Asc,
            SongSortOrder::Desc => SongsSortDirection::Desc,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSongsRequest {
    pub offset: Option<i64>,
    pub page_size: Option<i64>,
    pub sort_by: SongSortBy,
    pub sort_order: SongSortOrder,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongListItemDto {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSongsResponse {
    pub songs: Vec<SongListItemDto>,
    pub next_offset: Option<i64>,
}

#[derive(Debug)]
pub enum ListSongsError {
    ReadFailed,
}

#[derive(Debug, Clone)]
pub struct SongsService {
    db: Database,
    songs_repository: SongsRepository,
}

impl SongsService {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            songs_repository: SongsRepository::new(),
        }
    }

    pub async fn list_songs(
        &self,
        request: ListSongsRequest,
    ) -> Result<ListSongsResponse, ListSongsError> {
        let offset = request.offset.unwrap_or(0).max(0);
        let page_size = request
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);

        let rows = self
            .songs_repository
            .list_songs(
                &self.db.sqlx_pool,
                &ListSongsParams {
                    offset,
                    page_size: page_size + 1,
                    sort_key: request.sort_by.into(),
                    sort_direction: request.sort_order.into(),
                },
            )
            .await
            .map_err(|_| ListSongsError::ReadFailed)?;

        let has_next_page = rows.len() > page_size as usize;
        let songs = rows
            .into_iter()
            .take(page_size as usize)
            .map(|song| SongListItemDto {
                id: song.id,
                filepath: song.filepath,
                track_title: song.track_title,
                track_artist: song.track_artist,
                track_lyricist: song.track_lyricist,
                album_artist: song.album_artist,
                album_title: song.album_title,
                disc_number: song.disc_number,
                track_number: song.track_number,
                track_total: song.track_total,
                disc_total: song.disc_total,
                created_at: song.created_at,
                updated_at: song.updated_at,
            })
            .collect::<Vec<_>>();

        Ok(ListSongsResponse {
            next_offset: has_next_page.then_some(offset + page_size),
            songs,
        })
    }
}
