use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    repository::albums_repository::{
        AlbumsRepository, AlbumsSortDirection, AlbumsSortKey, ListAlbumsParams,
    },
    Database,
};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlbumSortBy {
    Title,
    Artist,
}

impl From<AlbumSortBy> for AlbumsSortKey {
    fn from(value: AlbumSortBy) -> Self {
        match value {
            AlbumSortBy::Title => AlbumsSortKey::Title,
            AlbumSortBy::Artist => AlbumsSortKey::Artist,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlbumSortOrder {
    Asc,
    Desc,
}

impl From<AlbumSortOrder> for AlbumsSortDirection {
    fn from(value: AlbumSortOrder) -> Self {
        match value {
            AlbumSortOrder::Asc => AlbumsSortDirection::Asc,
            AlbumSortOrder::Desc => AlbumsSortDirection::Desc,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAlbumsRequest {
    pub offset: Option<i64>,
    pub page_size: Option<i64>,
    pub sort_by: AlbumSortBy,
    pub sort_order: AlbumSortOrder,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumListItemDto {
    pub id: i64,
    pub album_title: String,
    pub album_artist: Option<String>,
    pub song_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAlbumsResponse {
    pub albums: Vec<AlbumListItemDto>,
    pub next_offset: Option<i64>,
}

#[derive(Debug)]
pub enum ListAlbumsError {
    ReadFailed,
}

#[derive(Debug, Clone)]
pub struct AlbumsService {
    db: Database,
    albums_repository: AlbumsRepository,
}

impl AlbumsService {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            albums_repository: AlbumsRepository::new(),
        }
    }

    pub async fn list_albums(
        &self,
        request: ListAlbumsRequest,
    ) -> Result<ListAlbumsResponse, ListAlbumsError> {
        let offset = request.offset.unwrap_or(0).max(0);
        let page_size = request
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);

        let rows = self
            .albums_repository
            .list_albums(
                &self.db.sqlx_pool,
                &ListAlbumsParams {
                    offset,
                    page_size: page_size + 1,
                    sort_key: request.sort_by.into(),
                    sort_direction: request.sort_order.into(),
                },
            )
            .await
            .map_err(|_| ListAlbumsError::ReadFailed)?;

        let has_next_page = rows.len() > page_size as usize;
        let albums = rows
            .into_iter()
            .take(page_size as usize)
            .map(|album| AlbumListItemDto {
                id: album.id,
                album_title: album.album_title,
                album_artist: album.album_artist,
                song_count: album.song_count,
                created_at: album.created_at,
                updated_at: album.updated_at,
            })
            .collect::<Vec<_>>();

        Ok(ListAlbumsResponse {
            albums,
            next_offset: has_next_page.then_some(offset + page_size),
        })
    }
}
