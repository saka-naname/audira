import { infiniteQueryOptions, useInfiniteQuery } from "@tanstack/react-query";
import { listAlbums } from "@/features/library/api/albums-api";
import type { AlbumSort, ListAlbumsResponse } from "@/features/library/types";

const ALBUMS_PAGE_SIZE = 50;

export function albumsInfiniteQueryOptions(sort: AlbumSort) {
  return infiniteQueryOptions({
    queryKey: ["albums", "infinite", sort] as const,
    initialPageParam: 0,
    queryFn: ({ pageParam }) =>
      listAlbums({
        offset: Number(pageParam),
        pageSize: ALBUMS_PAGE_SIZE,
        ...sort,
      }),
    getNextPageParam: (lastPage: ListAlbumsResponse) =>
      lastPage.nextOffset ?? undefined,
  });
}

export function useAlbumsInfiniteQuery(sort: AlbumSort) {
  return useInfiniteQuery(albumsInfiniteQueryOptions(sort));
}
