-- SQLite does not support changing a column type in place, so rebuild songs.
-- Keep copies of child rows because dropping songs applies their ON DELETE CASCADE.
-- This migration is also safe to run after the equivalent sqlx migration: in that
-- case it performs the same rebuild and records the Diesel migration version.
CREATE TEMPORARY TABLE IF NOT EXISTS song_metadata_backup AS
SELECT * FROM song_metadata;

CREATE TEMPORARY TABLE IF NOT EXISTS album_songs_backup AS
SELECT * FROM album_songs;

CREATE TABLE IF NOT EXISTS songs_new (
    id                      INTEGER  NOT NULL PRIMARY KEY,
    filepath                TEXT     NOT NULL,
    hash                    TEXT     NOT NULL,
    track_title             TEXT,
    track_artist            TEXT,
    track_lyricist          TEXT,
    album_artist            TEXT,
    album_title             TEXT,
    disc_number             INTEGER,
    track_number            INTEGER,
    track_total             INTEGER,
    disc_total              INTEGER,
    album_title_sort_order  TEXT,
    album_artist_sort_order TEXT,
    track_title_sort_order  TEXT,
    track_artist_sort_order TEXT,
    thumbnail_id            INTEGER  REFERENCES thumbnails (id) ON DELETE SET NULL,
    created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO songs_new (
    id,
    filepath,
    hash,
    track_title,
    track_artist,
    track_lyricist,
    album_artist,
    album_title,
    disc_number,
    track_number,
    track_total,
    disc_total,
    album_title_sort_order,
    album_artist_sort_order,
    track_title_sort_order,
    track_artist_sort_order,
    thumbnail_id,
    created_at,
    updated_at
)
SELECT
    id,
    filepath,
    hash,
    track_title,
    track_artist,
    track_lyricist,
    album_artist,
    album_title,
    disc_number,
    track_number,
    track_total,
    disc_total,
    album_title_sort_order,
    album_artist_sort_order,
    track_title_sort_order,
    track_artist_sort_order,
    CAST(thumbnail_id AS INTEGER),
    created_at,
    updated_at
FROM songs;

DROP TABLE songs;
ALTER TABLE songs_new RENAME TO songs;

DELETE FROM song_metadata;
INSERT INTO song_metadata
SELECT * FROM song_metadata_backup;

DELETE FROM album_songs;
INSERT INTO album_songs
SELECT * FROM album_songs_backup;

DROP TABLE song_metadata_backup;
DROP TABLE album_songs_backup;

CREATE INDEX IF NOT EXISTS idx_songs_title ON songs (track_title);
CREATE INDEX IF NOT EXISTS idx_songs_album_track ON songs (
    album_title, disc_number, track_number
);

CREATE TRIGGER IF NOT EXISTS trigger_songs_update_at AFTER UPDATE ON songs
    BEGIN
        UPDATE songs SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;
