import { IconMusic } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { Card, CardContent } from "@/components/ui/card";
import { playerCurrentTrackAtom } from "../state/playerAtoms";

export default function SidebarPlayer() {
  const currentTrack = useAtomValue(playerCurrentTrackAtom);

  return (
    <Card className="mx-auto w-fit gap-3">
      <CardContent className="flex flex-col justify-center gap-3">
        <div className="mx-auto flex aspect-square w-48 items-center justify-center rounded-lg bg-stone-400">
          <IconMusic size={120} className="text-stone-500" />
        </div>

        <div className="w-48 space-y-1 px-2">
          <p className="truncate">{currentTrack?.trackTitle ?? "-"}</p>
          <p className="truncate text-muted-foreground text-xs">
            {currentTrack?.trackArtist ?? "-"}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
