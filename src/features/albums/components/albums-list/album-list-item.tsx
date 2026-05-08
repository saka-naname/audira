import type { AlbumListItem } from "@/features/albums/types";

type AlbumListItemRowProps = Readonly<{
  album: AlbumListItem;
}>;

export default function AlbumListItemRow({ album }: AlbumListItemRowProps) {
  return (
    <div className="px-3 hover:bg-accent/50">
      <div className="grid h-14 grid-cols-[minmax(44px,44px)_minmax(0,2fr)_minmax(0,1fr)_minmax(88px,88px)] items-center gap-4 border-b px-1 text-sm">
        <div className="min-w-0">
          <div className="aspect-square w-11 rounded-md bg-stone-200"></div>
        </div>
        <div className="min-w-0">
          <p className="truncate font-medium">{album.albumTitle}</p>
        </div>
        <p className="truncate text-muted-foreground">
          {album.albumArtist || "-"}
        </p>
        <p className="truncate text-right text-muted-foreground">
          {album.songCount.toLocaleString()} 曲
        </p>
      </div>
    </div>
  );
}
