import { invoke } from "@tauri-apps/api/core";

export async function playTrack(id: number) {
  return invoke("play_track", { id });
}

export async function stopTrack() {
  return invoke("stop_track");
}

export async function pause() {
  return invoke("pause");
}

export async function resume() {
  return invoke("resume");
}
