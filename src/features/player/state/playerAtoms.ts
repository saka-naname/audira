import { atom } from "jotai";
import type { PlayerSnapshot, Track } from "../types";

export const playerSnapshotAtom = atom<PlayerSnapshot>({
  status: "idle",
});

export const playerCurrentTrackAtom = atom<Track | null>(null);
