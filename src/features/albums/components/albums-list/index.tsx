import {
  IconArrowsSort,
  IconSortAscending,
  IconSortDescending,
} from "@tabler/icons-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useMemo, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useAlbumsInfiniteQuery } from "@/features/albums/hooks/use-albums-infinite-query";
import type {
  AlbumListItem,
  AlbumSort,
  ListAlbumsResponse,
} from "@/features/albums/types";
import { cn } from "@/lib/utils";
import AlbumListItemRow from "./album-list-item";

const SKELETON_ROW_KEYS = Array.from(
  { length: 12 },
  (_, index) => `album-skeleton-${index}`,
);

type AlbumsListProps = Readonly<{
  sort: AlbumSort;
  onSortByChange: (sortBy: AlbumSort["sortBy"]) => void;
  onSortOrderToggle: () => void;
}>;

export default function AlbumsList({
  sort,
  onSortByChange,
  onSortOrderToggle,
}: AlbumsListProps) {
  const query = useAlbumsInfiniteQuery(sort);
  const albums = useMemo<AlbumListItem[]>(
    () =>
      query.data?.pages.flatMap((page: ListAlbumsResponse) => page.albums) ??
      [],
    [query.data],
  );

  const parentRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: query.hasNextPage ? albums.length + 1 : albums.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 64,
    overscan: 10,
  });
  const virtualItems = rowVirtualizer.getVirtualItems();

  useEffect(() => {
    const lastItem = virtualItems[virtualItems.length - 1];

    if (
      lastItem &&
      lastItem.index >= albums.length - 1 &&
      query.hasNextPage &&
      !query.isFetchingNextPage
    ) {
      query.fetchNextPage();
    }
  }, [
    albums.length,
    query.fetchNextPage,
    query.hasNextPage,
    query.isFetchingNextPage,
    virtualItems,
  ]);

  return (
    <section className="flex h-svh min-h-0 flex-col bg-background">
      <header className="flex h-24 shrink-0 items-center justify-between gap-4 border-b bg-background px-6">
        <div className="min-w-0">
          <h1 className="truncate font-semibold text-2xl tracking-normal">
            アルバム
          </h1>
          <p className="mt-1 text-muted-foreground text-sm">
            {albums.length.toLocaleString()} 枚を表示中
          </p>
        </div>

        <div className="flex shrink-0 items-center gap-2">
          <div className="flex items-center rounded-lg border bg-background p-1">
            <Button
              aria-pressed={sort.sortBy === "title"}
              className={cn(
                "rounded-md",
                sort.sortBy === "title" && "bg-muted text-foreground",
              )}
              onClick={() => onSortByChange("title")}
              size="sm"
              variant="ghost"
            >
              アルバム名
            </Button>
            <Button
              aria-pressed={sort.sortBy === "artist"}
              className={cn(
                "rounded-md",
                sort.sortBy === "artist" && "bg-muted text-foreground",
              )}
              onClick={() => onSortByChange("artist")}
              size="sm"
              variant="ghost"
            >
              アーティスト
            </Button>
          </div>

          <Button
            aria-label={
              sort.sortOrder === "asc" ? "昇順で並び替え" : "降順で並び替え"
            }
            onClick={onSortOrderToggle}
            size="icon"
            variant="outline"
          >
            {sort.sortOrder === "asc" ? (
              <IconSortAscending />
            ) : (
              <IconSortDescending />
            )}
          </Button>
        </div>
      </header>

      <div className="grid h-10 shrink-0 grid-cols-[minmax(44px,44px)_minmax(0,2fr)_minmax(0,1fr)_minmax(88px,88px)] items-center border-b bg-muted/30 px-4 font-medium text-muted-foreground text-xs">
        <span />
        <span className="ml-4">アルバム名</span>
        <span>アーティスト</span>
        <span className="text-right">曲数</span>
      </div>

      <div ref={parentRef} className="min-h-0 flex-1 overflow-auto">
        {query.isLoading ? (
          <div className="space-y-2 px-6 py-4">
            {SKELETON_ROW_KEYS.map((key) => (
              <Skeleton className="h-14 rounded-md" key={key} />
            ))}
          </div>
        ) : query.isError ? (
          <div className="flex h-full items-center justify-center px-6 text-muted-foreground text-sm">
            アルバムの読み込みに失敗しました。
          </div>
        ) : albums.length === 0 ? (
          <div className="flex h-full items-center justify-center px-6 text-muted-foreground text-sm">
            表示できるアルバムがありません。
          </div>
        ) : (
          <div
            className="relative w-full"
            style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
          >
            {virtualItems.map((virtualItem) => {
              const album = albums[virtualItem.index];

              return (
                <div
                  className="absolute top-0 left-0 w-full"
                  data-index={virtualItem.index}
                  key={virtualItem.key}
                  ref={rowVirtualizer.measureElement}
                  style={{
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  {album ? (
                    <AlbumListItemRow album={album} />
                  ) : (
                    <div className="flex h-14 items-center justify-center px-4 text-muted-foreground text-sm">
                      <IconArrowsSort className="mr-2 size-4 animate-pulse" />
                      読み込み中
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </section>
  );
}
