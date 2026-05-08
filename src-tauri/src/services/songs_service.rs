use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::repository::songs_repository::{
    ListSongsParams, SongsRepository, SongsSortDirection, SongsSortKey,
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
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_title: Option<String>,
    pub filepath: String,
    pub duration_ms: Option<i64>,
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
    db_pool: SqlitePool,
    songs_repository: SongsRepository,
}

impl SongsService {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool,
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
                &self.db_pool,
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
                title: song.track_title,
                artist: song.track_artist,
                album_title: song.album_title,
                filepath: song.filepath,
                duration_ms: song.duration_ms,
            })
            .collect::<Vec<_>>();

        Ok(ListSongsResponse {
            next_offset: has_next_page.then_some(offset + page_size),
            songs,
        })
    }
}
