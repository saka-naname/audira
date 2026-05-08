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
