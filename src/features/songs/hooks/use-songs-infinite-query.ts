import { infiniteQueryOptions, useInfiniteQuery } from "@tanstack/react-query";
import { listSongs } from "@/features/songs/api/songs-api";
import type { ListSongsResponse, SongSort } from "@/features/songs/types";

const SONGS_PAGE_SIZE = 50;

export function songsInfiniteQueryOptions(sort: SongSort) {
  return infiniteQueryOptions({
    queryKey: ["songs", "infinite", sort] as const,
    initialPageParam: 0,
    queryFn: ({ pageParam }) =>
      listSongs({
        offset: Number(pageParam),
        pageSize: SONGS_PAGE_SIZE,
        ...sort,
      }),
    getNextPageParam: (lastPage: ListSongsResponse) =>
      lastPage.nextOffset ?? undefined,
  });
}

export function useSongsInfiniteQuery(sort: SongSort) {
  return useInfiniteQuery(songsInfiniteQueryOptions(sort));
}
