-- This migration intentionally mirrors the sqlx initial migration. Diesel tracks
-- migration history separately, so every CREATE statement must also be safe when
-- the sqlx migration has already created the object.

CREATE TABLE IF NOT EXISTS thumbnails (
    id            INTEGER  NOT NULL PRIMARY KEY,
    filepath      TEXT     NOT NULL,
    original_hash TEXT     NOT NULL,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS songs (
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
    thumbnail_id            TEXT     REFERENCES thumbnails (id) ON DELETE SET NULL,
    created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS song_metadata (
    id             INTEGER  NOT NULL PRIMARY KEY,
    song_id        INTEGER  NOT NULL UNIQUE REFERENCES songs (id) ON DELETE CASCADE,
    recording_date TEXT,
    genre          TEXT,
    composer       TEXT,
    audio_bitrate  INTEGER,
    bit_depth      INTEGER,
    channels       INTEGER,
    sample_rate    INTEGER,
    duration_ms    INTEGER,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS albums (
    id                      INTEGER  NOT NULL PRIMARY KEY,
    album_artist            TEXT,
    album_title             TEXT     NOT NULL,
    album_title_sort_order  TEXT,
    album_artist_sort_order TEXT,
    created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS album_songs (
    album_id   INTEGER  NOT NULL REFERENCES albums (id) ON DELETE CASCADE,
    song_id    INTEGER  NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (album_id, song_id)
);

CREATE INDEX IF NOT EXISTS idx_songs_title ON songs (track_title);
CREATE INDEX IF NOT EXISTS idx_songs_album_track ON songs (
    album_title, disc_number, track_number
);

CREATE TRIGGER IF NOT EXISTS trigger_thumbnails_update_at AFTER UPDATE ON thumbnails
    BEGIN
        UPDATE thumbnails SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;

CREATE TRIGGER IF NOT EXISTS trigger_songs_update_at AFTER UPDATE ON songs
    BEGIN
        UPDATE songs SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;

CREATE TRIGGER IF NOT EXISTS trigger_song_metadata_update_at AFTER UPDATE ON song_metadata
    BEGIN
        UPDATE song_metadata SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;

CREATE TRIGGER IF NOT EXISTS trigger_albums_update_at AFTER UPDATE ON albums
    BEGIN
        UPDATE albums SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;

CREATE TRIGGER IF NOT EXISTS trigger_album_songs_update_at AFTER UPDATE ON album_songs
    BEGIN
        UPDATE album_songs SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
    END;
