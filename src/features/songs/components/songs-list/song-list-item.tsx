import type { SongListItem } from "@/features/songs/types";

type SongListItemRowProps = Readonly<{
  song: SongListItem;
}>;

export default function SongListItemRow({ song }: SongListItemRowProps) {
  return (
    <div className="hover:bg-accent/50 px-3">
          <div className="grid h-14 grid-cols-[minmax(44px,44px)_minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)] items-center gap-4 border-b text-sm px-1">
      <div className="min-w-0">
        <div className="w-11 aspect-square bg-stone-200 rounded-md"></div>
      </div>
      <div className="min-w-0">
        <p className="truncate font-medium">
          {song.trackTitle ?? song.filepath}
        </p>
      </div>
      <p className="truncate text-muted-foreground">
        {song.trackArtist || "-"}
      </p>
      <p className="truncate text-muted-foreground">
        {song.albumTitle || "-"}
      </p>
    </div>
    </div>

  );
}
