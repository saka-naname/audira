export type SongSortBy = "title" | "artist";

export type SongSortOrder = "asc" | "desc";

export type SongSort = {
  sortBy: SongSortBy;
  sortOrder: SongSortOrder;
};

export type SongListItem = {
  id: number;
  filepath: string;
  trackTitle: string | null;
  trackArtist: string | null;
  trackLyricist: string | null;
  albumArtist: string | null;
  albumTitle: string | null;
  discNumber: number | null;
  trackNumber: number | null;
  trackTotal: number | null;
  discTotal: number | null;
  createdAt: string;
  updatedAt: string;
};

export type ListSongsRequest = SongSort & {
  offset?: number;
  pageSize: number;
};

export type ListSongsResponse = {
  songs: SongListItem[];
  nextOffset: number | null;
};

export type AlbumSortBy = "title" | "artist";

export type AlbumSortOrder = "asc" | "desc";

export type AlbumSort = {
  sortBy: AlbumSortBy;
  sortOrder: AlbumSortOrder;
};

export type AlbumListItem = {
  id: number;
  albumTitle: string;
  albumArtist: string | null;
  songCount: number;
  createdAt: string;
  updatedAt: string;
};

export type ListAlbumsRequest = AlbumSort & {
  offset?: number;
  pageSize: number;
};

export type ListAlbumsResponse = {
  albums: AlbumListItem[];
  nextOffset: number | null;
};
