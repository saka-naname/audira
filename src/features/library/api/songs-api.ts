import { invoke } from "@tauri-apps/api/core";
import type {
  ListSongsRequest,
  ListSongsResponse,
} from "@/features/library/types";

export async function listSongs(
  request: ListSongsRequest,
): Promise<ListSongsResponse> {
  return invoke<ListSongsResponse>("list_songs", { request });
}
