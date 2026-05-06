use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use blake3::Hasher;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::ItemKey;
use sqlx::{SqliteConnection, SqlitePool};
use tauri::Runtime;
use walkdir::WalkDir;

use crate::constants::AUDIO_EXTENSIONS;
use crate::models::songs::Songs;
use crate::repository::albums_repository::{
    connect_album_and_song, find_album_by_title_and_artist, insert_album, InsertAlbumsParams,
};
use crate::repository::songs_repository::{self, is_exist_by_hash};

#[tauri::command]
pub async fn scan_library<R: Runtime>(
    _app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    pool: tauri::State<'_, SqlitePool>,
    base_dir: &str,
) -> Result<(), String> {
    let start = std::time::Instant::now();

    let base_path = Path::new(base_dir);

    if !base_path.exists() {
        return Err(String::from("指定されたパスは存在しません。"));
    }

    if !base_path.is_dir() {
        return Err(String::from("指定されたパスはディレクトリではありません。"));
    }

    let files = walk_audio_files(base_path);
    let files = hash_files(files).filter_map(|e| e.ok());
    let mut count = 0;

    let Ok(mut tx) = pool.begin().await else {
        return Err(String::from("トランザクションの生成に失敗しました"));
    };

    for (file, hash) in files {
        println!("[{:?}] File {}: {}", start.elapsed(), count + 1, file);

        let Ok(is_exist) = is_exist_by_hash(&mut *tx, &hash).await else {
            println!("  Cache Hit. Skipped");
            continue;
        };
        if is_exist {
            println!("  Cache Hit. Skipped");
            continue;
        }

        let tagged_file = match lofty::read_from_path(&file) {
            Ok(tagged_file) => tagged_file,
            Err(_) => continue,
        };

        let props = tagged_file.properties();
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag());
        let Some(tag) = tag else { continue };

        // TODO: picture ハッシュ計算 & リンク

        let song = songs_repository::InsertSongsParams {
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

        let song_metadata = songs_repository::InsertSongMetadataParams {
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

        let Ok((song, _)) =
            songs_repository::insert_with_metadata(&mut *tx, &song, &song_metadata).await
        else {
            tx.rollback().await.expect("Transaction rollback error.");
            return Err(String::from(
                "トランザクションの書き込み中に問題が発生しました",
            ));
        };

        // アルバムを関連付け（なければ作成）
        if let Err(e) = create_or_connect_album(&mut *tx, song).await {
            return Err(e);
        }

        count += 1;
    }

    if let Err(_) = tx.commit().await {
        return Err(String::from("トランザクションの実行中に問題が発生しました"));
    };

    dbg!(format!(
        "Elapsed time: {:?} ({} files)",
        start.elapsed(),
        count
    ));

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
    let tasks = files_iter.map(|path| {
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
    });

    tasks
}

async fn create_or_connect_album(conn: &mut SqliteConnection, song: Songs) -> Result<(), String> {
    let song_id = &song.id;
    let album_title = song.album_title.as_deref();
    let album_artist = song.album_artist.as_deref();

    // Album must have album_title
    let Some(album_title) = album_title else {
        return Ok(());
    };

    println!("find_album_by_title_and_artist");
    let album = find_album_by_title_and_artist(conn, &album_title, album_artist.as_deref()).await;

    let Ok(album) = album else {
        return Err(String::from("アルバムの検索に失敗しました"));
    };

    if let Some(album) = album {
        // Connect with existing album
        println!("connect_album_and_song");

        if let Err(_) = connect_album_and_song(&mut *conn, &album.id, song_id).await {
            return Err(String::from("アルバムへの登録に失敗しました"));
        }
    } else {
        // Create new album
        let params = InsertAlbumsParams {
            album_title: String::from(album_title),
            album_artist: match album_artist {
                Some(aa) => Some(String::from(aa)),
                None => None,
            },
        };
        println!("insert_album");

        let Ok(album) = insert_album(&mut *conn, &params).await else {
            return Err(String::from("アルバムの作成に失敗しました"));
        };
        println!("connect_album_and_song");

        if let Err(_) = connect_album_and_song(&mut *conn, &album.id, song_id).await {
            return Err(String::from("アルバムへの登録に失敗しました"));
        }
    }

    Ok(())
}
