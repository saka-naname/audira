-- thumbnails
CREATE TABLE IF NOT EXISTS thumbnails (
    id   TEXT NOT NULL PRIMARY KEY,
    filepath TEXT NOT NULL,
    original_hash TEXT NOT NULL
);

-- songs
CREATE TABLE IF NOT EXISTS songs (
    id                      TEXT NOT NULL PRIMARY KEY,
    filepath                TEXT NOT NULL,
    hash                    TEXT NOT NULL,
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
    updated_at              TEXT,
    thumbnail_id            TEXT REFERENCES thumbnails (id) ON DELETE SET NULL
);

-- song_metadata (1:1 with songs)
CREATE TABLE IF NOT EXISTS song_metadata (
    id             TEXT NOT NULL PRIMARY KEY,
    song_id        TEXT NOT NULL UNIQUE REFERENCES songs (id) ON DELETE CASCADE,
    recording_date TEXT,
    genre          TEXT,
    composer       TEXT,
    audio_bitrate  INTEGER,
    bit_depth      INTEGER,
    channels       INTEGER,
    sample_rate    INTEGER,
    duration_ms    INTEGER
);

-- albums
CREATE TABLE IF NOT EXISTS albums (
    id           TEXT NOT NULL PRIMARY KEY,
    album_artist TEXT NOT NULL,
    album_title  TEXT NOT NULL
);

-- album_songs (albums <-> songs の中間テーブル)
CREATE TABLE IF NOT EXISTS album_songs (
    album_id TEXT NOT NULL REFERENCES albums (id) ON DELETE CASCADE,
    song_id  TEXT NOT NULL REFERENCES songs  (id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, song_id)
);

-- indexes
CREATE INDEX idx_songs_title ON songs(track_title);
CREATE INDEX idx_songs_album_track ON songs(album_title, disc_number, track_number);
