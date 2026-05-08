import {
  IconArrowDown,
  IconArrowUp,
  IconMusic,
  IconSortAscendingLetters,
  IconUser,
} from "@tabler/icons-react";
import { useEffect, useMemo, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useSongsInfiniteQuery } from "@/features/songs/hooks/use-songs-infinite-query";
import type {
  ListSongsResponse,
  SongListItem,
  SongSort,
} from "@/features/songs/types";
import { cn } from "@/lib/utils";

const sortLabels = {
  title: "曲名",
  artist: "アーティスト",
} as const;

const skeletonRows = Array.from(
  { length: 12 },
  (_, index) => `song-skeleton-${index}`,
);

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
  const loadMoreRef = useRef<HTMLDivElement | null>(null);
  const query = useSongsInfiniteQuery(sort);
  const songs = useMemo<SongListItem[]>(
    () =>
      query.data?.pages.flatMap((page: ListSongsResponse) => page.songs) ?? [],
    [query.data],
  );

  useEffect(() => {
    const target = loadMoreRef.current;
    if (!target) {
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (
          entry?.isIntersecting &&
          query.hasNextPage &&
          !query.isFetchingNextPage
        ) {
          query.fetchNextPage();
        }
      },
      { rootMargin: "240px" },
    );

    observer.observe(target);
    return () => observer.disconnect();
  }, [query.fetchNextPage, query.hasNextPage, query.isFetchingNextPage]);

  return (
    <section className="flex h-full min-h-0 flex-col bg-background">
      <header className="border-b bg-background/95 px-8 py-6 backdrop-blur supports-[backdrop-filter]:bg-background/80">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="font-medium text-muted-foreground text-sm">
              ライブラリ
            </p>
            <h1 className="font-semibold text-3xl tracking-tight">曲</h1>
            <p className="mt-2 text-muted-foreground text-sm">
              SQLite
              データベースに保存された全楽曲を無限スクロールで表示します。
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <span className="flex items-center gap-1.5 text-muted-foreground text-sm">
              <IconSortAscendingLetters className="size-4" />
              ソート
            </span>
            <Button
              type="button"
              variant={sort.sortBy === "title" ? "default" : "outline"}
              onClick={() => onSortByChange("title")}
            >
              <IconMusic />
              曲名
            </Button>
            <Button
              type="button"
              variant={sort.sortBy === "artist" ? "default" : "outline"}
              onClick={() => onSortByChange("artist")}
            >
              <IconUser />
              アーティスト
            </Button>
            <Button type="button" variant="outline" onClick={onSortOrderToggle}>
              {sort.sortOrder === "asc" ? <IconArrowUp /> : <IconArrowDown />}
              {sort.sortOrder === "asc" ? "昇順" : "降順"}
            </Button>
          </div>
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-8 py-6">
        <div className="mb-3 flex items-center justify-between text-muted-foreground text-sm">
          <span>
            {sortLabels[sort.sortBy]}・
            {sort.sortOrder === "asc" ? "昇順" : "降順"}
          </span>
          <span>{songs.length.toLocaleString()} 曲を表示中</span>
        </div>

        <div className="overflow-hidden rounded-xl border bg-card">
          <div className="grid grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)_minmax(0,1fr)] gap-4 border-b bg-muted/40 px-4 py-3 font-medium text-muted-foreground text-xs uppercase tracking-wide">
            <span>曲名</span>
            <span>アーティスト</span>
            <span>アルバム</span>
          </div>

          {query.isLoading ? <SongsListSkeleton /> : null}

          {query.isError ? (
            <div className="px-4 py-10 text-center text-destructive text-sm">
              楽曲一覧を取得できませんでした。
            </div>
          ) : null}

          {!query.isLoading && !query.isError && songs.length === 0 ? (
            <div className="px-4 py-16 text-center">
              <IconMusic className="mx-auto mb-3 size-10 text-muted-foreground" />
              <p className="font-medium">楽曲がまだ登録されていません</p>
              <p className="mt-1 text-muted-foreground text-sm">
                ライブラリをスキャンすると、ここに曲が表示されます。
              </p>
            </div>
          ) : null}

          {songs.map((song) => (
            <SongRow key={song.id} song={song} />
          ))}
        </div>

        <div
          ref={loadMoreRef}
          className="flex h-20 items-center justify-center"
        >
          {query.isFetchingNextPage ? (
            <span className="text-muted-foreground text-sm">
              さらに読み込み中...
            </span>
          ) : null}
        </div>
      </div>
    </section>
  );
}

function SongRow({ song }: Readonly<{ song: SongListItem }>) {
  return (
    <div className="grid grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)_minmax(0,1fr)] gap-4 border-b px-4 py-3 last:border-b-0 hover:bg-muted/40">
      <div className="min-w-0">
        <p className="truncate font-medium">
          {song.trackTitle || filenameFromPath(song.filepath)}
        </p>
        <p className="truncate text-muted-foreground text-xs">
          {song.filepath}
        </p>
      </div>
      <CellText>{song.trackArtist || "不明なアーティスト"}</CellText>
      <CellText>{song.albumTitle || "不明なアルバム"}</CellText>
    </div>
  );
}

function CellText({ children }: Readonly<{ children: string }>) {
  return <span className="self-center truncate text-sm">{children}</span>;
}

function SongsListSkeleton() {
  return skeletonRows.map((rowId, index) => (
    <div
      className={cn(
        "grid grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)_minmax(0,1fr)] gap-4 border-b px-4 py-3",
        index === 11 && "border-b-0",
      )}
      key={rowId}
    >
      <div className="space-y-2">
        <Skeleton className="h-4 w-2/3" />
        <Skeleton className="h-3 w-full" />
      </div>
      <Skeleton className="h-4 self-center" />
      <Skeleton className="h-4 self-center" />
    </div>
  ));
}

function filenameFromPath(filepath: string) {
  return filepath.split(/[\\/]/).pop() || filepath;
}
