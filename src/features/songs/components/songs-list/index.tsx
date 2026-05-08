import { useMemo, useRef } from "react";
import { useSongsInfiniteQuery } from "@/features/songs/hooks/use-songs-infinite-query";
import type {
  ListSongsResponse,
  SongListItem,
  SongSort,
} from "@/features/songs/types";
import { useVirtualizer } from "@tanstack/react-virtual"

type SongsListProps = Readonly<{
  sort: SongSort;
  onSortByChange: (sortBy: SongSort["sortBy"]) => void;
  onSortOrderToggle: () => void;
}>;

export default function SongsList({
  sort,
  onSortByChange,
  onSortOrderToggle,
}: SongsListProps) {
  const query = useSongsInfiniteQuery(sort);
  const songs = useMemo<SongListItem[]>(
    () =>
      query.data?.pages.flatMap((page: ListSongsResponse) => page.songs) ?? [],
    [query.data],
  );

  const parentRef = useRef<HTMLDivElement>(null)
  const rowVirtualizer = useVirtualizer({
    count: query.hasNextPage ? songs.length + 1 : songs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80,
    overscan: 10,
  })

  return (
    <section>
      
    </section>
  );
}
