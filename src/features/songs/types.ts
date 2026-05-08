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
