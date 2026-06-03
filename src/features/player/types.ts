export type PlaybackStatus = "idle" | "playing" | "paused";

export type Track = {
  id: number;
  filepath: string;
  trackTitle: string | null;
  trackArtist: string | null;
  albumTitle: string | null;
};

export type PlayerSnapshot = {
  status: PlaybackStatus;
};
