export type SongSortBy = "title" | "artist";

export type SongSortOrder = "asc" | "desc";

export type SongSort = {
  sortBy: SongSortBy;
  sortOrder: SongSortOrder;
};

export type SongListItem = {
  id: number;
  title: string | null;
  artist: string | null;
  albumTitle: string | null;
  filepath: string;
  durationMs: number | null;
};

export type ListSongsRequest = SongSort & {
  offset?: number;
  pageSize: number;
};

export type ListSongsResponse = {
  songs: SongListItem[];
  nextOffset: number | null;
};
