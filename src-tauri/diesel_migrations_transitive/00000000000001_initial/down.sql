DROP TRIGGER IF EXISTS trigger_album_songs_update_at;
DROP TRIGGER IF EXISTS trigger_albums_update_at;
DROP TRIGGER IF EXISTS trigger_song_metadata_update_at;
DROP TRIGGER IF EXISTS trigger_songs_update_at;
DROP TRIGGER IF EXISTS trigger_thumbnails_update_at;

DROP INDEX IF EXISTS idx_songs_album_track;
DROP INDEX IF EXISTS idx_songs_title;

DROP TABLE IF EXISTS album_songs;
DROP TABLE IF EXISTS albums;
DROP TABLE IF EXISTS song_metadata;
DROP TABLE IF EXISTS songs;
DROP TABLE IF EXISTS thumbnails;
