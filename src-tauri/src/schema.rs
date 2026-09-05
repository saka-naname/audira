// @generated automatically by Diesel CLI.

diesel::table! {
    album_songs (album_id, song_id) {
        album_id -> BigInt,
        song_id -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    albums (id) {
        id -> BigInt,
        album_artist -> Nullable<Text>,
        album_title -> Text,
        album_title_sort_order -> Nullable<Text>,
        album_artist_sort_order -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    song_metadata (id) {
        id -> BigInt,
        song_id -> BigInt,
        recording_date -> Nullable<Text>,
        genre -> Nullable<Text>,
        composer -> Nullable<Text>,
        audio_bitrate -> Nullable<Integer>,
        bit_depth -> Nullable<Integer>,
        channels -> Nullable<Integer>,
        sample_rate -> Nullable<Integer>,
        duration_ms -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    songs (id) {
        id -> BigInt,
        filepath -> Text,
        hash -> Text,
        track_title -> Nullable<Text>,
        track_artist -> Nullable<Text>,
        track_lyricist -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        album_title -> Nullable<Text>,
        disc_number -> Nullable<Integer>,
        track_number -> Nullable<Integer>,
        track_total -> Nullable<Integer>,
        disc_total -> Nullable<Integer>,
        album_title_sort_order -> Nullable<Text>,
        album_artist_sort_order -> Nullable<Text>,
        track_title_sort_order -> Nullable<Text>,
        track_artist_sort_order -> Nullable<Text>,
        thumbnail_id -> Nullable<BigInt>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    thumbnails (id) {
        id -> BigInt,
        filepath -> Text,
        original_hash -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(album_songs -> albums (album_id));
diesel::joinable!(album_songs -> songs (song_id));
diesel::joinable!(song_metadata -> songs (song_id));
diesel::joinable!(songs -> thumbnails (thumbnail_id));

diesel::allow_tables_to_appear_in_same_query!(
    album_songs,
    albums,
    song_metadata,
    songs,
    thumbnails,
);
