import { Button } from "@base-ui/react";
import { IconPlayerPlayFilled } from "@tabler/icons-react";
import { useCallback } from "react";
import type { SongListItem } from "@/features/library/types";
import { playTrack, stopTrack } from "../../api/songs-api";

type SongListItemRowProps = Readonly<{
  song: SongListItem;
}>;

export default function SongListItemRow({ song }: SongListItemRowProps) {
  const handlePlay = useCallback(async () => {
    await stopTrack();
    void playTrack(song.id);
  }, [song]);

  return (
    <div className="px-3 hover:bg-accent/50">
      <div className="grid h-14 grid-cols-[minmax(44px,44px)_minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)] items-center gap-4 border-b px-1 text-sm">
        <div className="min-w-0">
          <Button
            render={<div />}
            nativeButton={false}
            className="group relative aspect-square w-11 rounded-md bg-stone-200"
            onClick={handlePlay}
          >
            <IconPlayerPlayFilled className="absolute inset-0 m-auto hidden text-stone-500/70 group-hover:block" />
          </Button>
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
