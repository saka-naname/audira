use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use blake3::Hasher;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::ItemKey;
use sqlx::{SqliteConnection, SqlitePool};
use walkdir::WalkDir;

use crate::constants::AUDIO_EXTENSIONS;
use crate::models::songs::Songs;
use crate::repository::albums_repository::{AlbumsRepository, InsertAlbumsParams};
use crate::repository::songs_repository::{
    InsertSongMetadataParams, InsertSongsParams, SongsRepository,
};

#[derive(Debug)]
pub enum LibraryScanError {
    BasePathNotFound,
    BasePathIsNotDirectory,
    BeginTransactionFailed,
    WriteTransactionFailed,
    CommitTransactionFailed,
    FindAlbumFailed,
    CreateAlbumFailed,
    ConnectAlbumFailed,
}

pub struct LibraryService {
    pool: SqlitePool,
    songs_repository: SongsRepository,
    albums_repository: AlbumsRepository,
}

impl LibraryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            songs_repository: SongsRepository::new(),
            albums_repository: AlbumsRepository::new(),
        }
    }

    pub async fn scan_library(&self, base_dir: &str) -> Result<(), LibraryScanError> {
        let start = std::time::Instant::now();
        let base_path = Path::new(base_dir);

        Self::validate_base_path(base_path)?;

        let files = Self::walk_audio_files(base_path);
        let files = Self::hash_files(files).filter_map(|e| e.ok());
        let mut count = 0;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| LibraryScanError::BeginTransactionFailed)?;

        for (file, hash) in files {
            println!("[{:?}] File {}: {}", start.elapsed(), count + 1, file);

            let Ok(is_exist) = self.songs_repository.is_exist_by_hash(&mut tx, &hash).await else {
                println!("  Cache Hit. Skipped");
                continue;
            };
            if is_exist {
                println!("  Cache Hit. Skipped");
                continue;
            }

            let Some((song, song_metadata)) = Self::extract_song_params(file, hash) else {
                continue;
            };

            let (song, _) = self
                .songs_repository
                .insert_with_metadata(&mut tx, &song, &song_metadata)
                .await
                .map_err(|_| LibraryScanError::WriteTransactionFailed)?;

            self.create_or_connect_album(&mut tx, song).await?;

            count += 1;
        }

        tx.commit()
            .await
            .map_err(|_| LibraryScanError::CommitTransactionFailed)?;

        dbg!(format!(
            "Elapsed time: {:?} ({} files)",
            start.elapsed(),
            count
        ));

        Ok(())
    }

    fn validate_base_path(base_path: &Path) -> Result<(), LibraryScanError> {
        if !base_path.exists() {
            return Err(LibraryScanError::BasePathNotFound);
        }

        if !base_path.is_dir() {
            return Err(LibraryScanError::BasePathIsNotDirectory);
        }

        Ok(())
    }

    fn walk_audio_files(base_path: &Path) -> impl Iterator<Item = String> {
        WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_string_lossy().to_string())
    }

    fn hash_files(
        files_iter: impl Iterator<Item = String>,
    ) -> impl Iterator<Item = Result<(String, String), String>> {
        files_iter.map(|path| {
            let Ok(file) = File::open(&path) else {
                return Err(format!("Failed to open file: {}", path));
            };
            let mut reader = BufReader::new(file);
            let mut hasher = Hasher::new();

            let mut buffer = [0u8; 65536];
            loop {
                let Ok(n) = reader.read(&mut buffer) else {
                    return Err(format!("Failed to calculate hash: {}", path));
                };
                if n == 0 {
                    break;
                }
                hasher.update_rayon(&buffer[..n]);
            }

            Ok((path, hasher.finalize().to_hex().to_string()))
        })
    }

    fn extract_song_params(
        file: String,
        hash: String,
    ) -> Option<(InsertSongsParams, InsertSongMetadataParams)> {
        let tagged_file = lofty::read_from_path(&file).ok()?;
        let props = tagged_file.properties();
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())?;

        // TODO: picture ハッシュ計算 & リンク

        let song = InsertSongsParams {
            filepath: file,
            hash,
            track_title: tag.get_string(ItemKey::TrackTitle).map(|s| s.to_string()),
            track_artist: tag.get_string(ItemKey::TrackArtist).map(|s| s.to_string()),
            track_lyricist: tag.get_string(ItemKey::Lyricist).map(|s| s.to_string()),
            album_artist: tag.get_string(ItemKey::AlbumArtist).map(|s| s.to_string()),
            album_title: tag.get_string(ItemKey::AlbumTitle).map(|s| s.to_string()),
            disc_number: tag
                .get_string(ItemKey::DiscNumber)
                .and_then(|s| s.parse::<i64>().ok()),
            track_number: tag
                .get_string(ItemKey::TrackNumber)
                .and_then(|s| s.parse::<i64>().ok()),
            track_total: tag
                .get_string(ItemKey::TrackTotal)
                .and_then(|s| s.parse::<i64>().ok()),
            disc_total: tag
                .get_string(ItemKey::DiscTotal)
                .and_then(|s| s.parse::<i64>().ok()),
            album_title_sort_order: tag
                .get_string(ItemKey::AlbumTitleSortOrder)
                .map(|s| s.to_string()),
            album_artist_sort_order: tag
                .get_string(ItemKey::AlbumTitleSortOrder)
                .map(|s| s.to_string()),
            track_title_sort_order: tag
                .get_string(ItemKey::TrackTitleSortOrder)
                .map(|s| s.to_string()),
            track_artist_sort_order: tag
                .get_string(ItemKey::TrackArtistSortOrder)
                .map(|s| s.to_string()),
            thumbnail_id: None,
        };

        let song_metadata = InsertSongMetadataParams {
            recording_date: tag
                .get_string(ItemKey::RecordingDate)
                .map(|s| s.to_string()),
            genre: tag.get_string(ItemKey::Genre).map(|s| s.to_string()),
            composer: tag.get_string(ItemKey::Composer).map(|s| s.to_string()),
            audio_bitrate: props.audio_bitrate().map(|u| u as i64),
            bit_depth: props.bit_depth().map(|u| u as i64),
            channels: props.channels().map(|u| u as i64),
            sample_rate: props.sample_rate().map(|u| u as i64),
            duration_ms: props.duration().as_millis().try_into().ok(),
        };

        Some((song, song_metadata))
    }

    async fn create_or_connect_album(
        &self,
        conn: &mut SqliteConnection,
        song: Songs,
    ) -> Result<(), LibraryScanError> {
        let song_id = &song.id;
        let album_title = song.album_title.as_deref();
        let album_artist = song.album_artist.as_deref();
        let album_title_sort_order = song.album_title_sort_order.as_deref();
        let album_artist_sort_order = song.album_artist_sort_order.as_deref();

        let Some(album_title) = album_title else {
            return Ok(());
        };

        println!("find_album_by_title_and_artist");
        let album = self
            .albums_repository
            .find_album_by_params(
                conn,
                album_title,
                album_artist,
                album_title_sort_order,
                album_artist_sort_order,
            )
            .await
            .map_err(|_| LibraryScanError::FindAlbumFailed)?;

        if let Some(album) = album {
            println!("connect_album_and_song");
            self.albums_repository
                .connect_album_and_song(&mut *conn, &album.id, song_id)
                .await
                .map_err(|_| LibraryScanError::ConnectAlbumFailed)?;
        } else {
            let params = InsertAlbumsParams {
                album_title: String::from(album_title),
                album_artist: album_artist.map(String::from),
                album_title_sort_order: album_title_sort_order.map(String::from),
                album_artist_sort_order: album_artist_sort_order.map(String::from),
            };
            println!("insert_album");

            let album = self
                .albums_repository
                .insert_album(&mut *conn, &params)
                .await
                .map_err(|_| LibraryScanError::CreateAlbumFailed)?;
            println!("connect_album_and_song");

            self.albums_repository
                .connect_album_and_song(&mut *conn, &album.id, song_id)
                .await
                .map_err(|_| LibraryScanError::ConnectAlbumFailed)?;
        }

        Ok(())
    }
}
