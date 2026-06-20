import { Button } from "@base-ui/react";
import {
  IconPlayerPauseFilled,
  IconPlayerPlayFilled,
} from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useCallback } from "react";
import { pause, resume } from "../../api/player-api";
import { playerSnapshotAtom } from "../../state/playerAtoms";

export default function ControllerOverlay() {
  const { status } = useAtomValue(playerSnapshotAtom);

  const handleTogglePause = useCallback(async () => {
    switch (status) {
      case "idle": {
        return;
      }
      case "playing": {
        void pause();
        break;
      }
      case "paused": {
        void resume();
        break;
      }
    }
  }, [status]);

  return (
    <div className="group/display absolute inset-0 bg-black/0 hover:bg-black/25 has-focus-visible:bg-black/25">
      {status === "idle" ? null : (
        <div className="flex h-full w-full items-center justify-center opacity-0 group-hover/display:opacity-100 has-focus-visible:opacity-100">
          <Button
            onClick={handleTogglePause}
            className="size-12 opacity-50 hover:opacity-100 focus-visible:opacity-100"
          >
            {status === "playing" ? (
              <IconPlayerPauseFilled className="size-12 text-black" />
            ) : (
              <IconPlayerPlayFilled className="size-12 text-black" />
            )}
          </Button>
        </div>
      )}
    </div>
  );
}
