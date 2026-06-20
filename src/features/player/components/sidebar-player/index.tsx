import { useAtomValue } from "jotai";
import { Card, CardContent } from "@/components/ui/card";
import { playerCurrentTrackAtom } from "../../state/playerAtoms";
import DisplayController from "./display-controller";

export default function SidebarPlayer() {
  const currentTrack = useAtomValue(playerCurrentTrackAtom);

  return (
    <Card className="mx-auto w-fit gap-3">
      <CardContent className="flex flex-col justify-center gap-3">
        <DisplayController />

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
