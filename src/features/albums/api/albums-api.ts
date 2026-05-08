import { invoke } from "@tauri-apps/api/core";
import type {
  ListAlbumsRequest,
  ListAlbumsResponse,
} from "@/features/albums/types";

export async function listAlbums(
  request: ListAlbumsRequest,
): Promise<ListAlbumsResponse> {
  return invoke<ListAlbumsResponse>("list_albums", { request });
}
