import { listen } from "@tauri-apps/api/event";
import { useSetAtom } from "jotai";
import { useEffect } from "react";
import {
  playerCurrentTrackAtom,
  playerSnapshotAtom,
} from "../state/playerAtoms";
import type { PlayerSnapshot, Track } from "../types";

export function PlayerEventBridge() {
  const setSnapshot = useSetAtom(playerSnapshotAtom);
  const setCurrentTrack = useSetAtom(playerCurrentTrackAtom);

  useEffect(() => {
    const unlistenPromises = [
      listen<PlayerSnapshot>("player://state", (event) => {
        setSnapshot(event.payload);
      }),
      listen<Track>("player://track", (event) => {
        setCurrentTrack(event.payload);
      }),
    ];

    return () => {
      void Promise.all(unlistenPromises).then((unlisteners) => {
        for (const unlisten of unlisteners) {
          unlisten();
        }
      });
    };
  }, [setSnapshot, setCurrentTrack]);

  return null;
}
