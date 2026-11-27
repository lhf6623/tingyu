import { invoke } from "@tauri-apps/api/core";

export type Artist = { id?: string; name: string };
export type Album = { id: string; title: string };

export type Track = {
  id: string;
  title: string;
  artists: Artist[];
  album?: Album;
  coverUrl?: string;
  durationSeconds?: number;
  available: boolean;
  favorite?: boolean;
};

export type AuthStatus = {
  authenticated: boolean;
  accountLabel?: string;
  musicExpireTime?: string;
};

export type Page<T> = {
  items: T[];
  nextStart: number;
  hasMore: boolean;
};

export type Playback = {
  url: string;
  actualQuality: "standard" | "lossless" | "unknown";
  contentType?: string;
  durationSeconds?: number;
};

export type Playlist = {
  id: string;
  title: string;
  coverUrl?: string;
  count?: number;
  kind: "owned" | "collected" | "chart" | "unknown";
  description?: string;
};

export type PlaylistPage = {
  playlist: Playlist | null;
  items: Track[];
  nextStart: number;
  hasMore: boolean;
};

export type AlbumCard = { id: string; title: string; coverUrl?: string; count?: number };
export type AlbumMeta = {
  id: string;
  title: string;
  coverUrl?: string;
  artists: Artist[];
  count?: number;
};

export type AlbumPage = { album: AlbumMeta; items: Track[]; nextStart: number; hasMore: boolean };

export type ArtistMeta = { id: string; name: string; coverUrl?: string };
export type ArtistPage = { artist: ArtistMeta; items: Track[]; albums: AlbumCard[] };

export type LyricSegment = { timeMs: number | null; text: string };
export type LyricLine = {
  timeMs: number | null;
  text: string;
  translation?: string;
  segments?: LyricSegment[];
};
export type LyricsDoc = {
  synced: boolean;
  lines: LyricLine[];
  source?: "huawei" | "local";
};

export type AppError = { kind: string; message: string; retryable: boolean; code?: string };

export type PersistedState = {
  volume: number;
  mode: string;
  quality: string;
  queueOpen: boolean;
  closeToTray: boolean;
  cursor: number;
  queue: Track[];
};

export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  const appError = error as AppError | undefined;
  if (appError?.message) return appError.message;
  return (error as Error)?.message ?? "未知错误";
}

export const api = {
  authStatus: () => invoke<AuthStatus>("auth_status"),
  login: () => invoke<AuthStatus>("login"),
  logout: () => invoke<void>("logout"),

  favorites: (start = 0, limit = 50) => invoke<Page<Track>>("favorites", { start, limit }),
  recent: () => invoke<Track[]>("recent"),
  playlists: () => invoke<Playlist[]>("playlists"),
  playlistDetail: (playlistId: string, kind: string, start = 0, limit = 50) =>
    invoke<PlaylistPage>("playlist_detail", { playlistId, kind, start, limit }),

  search: (query: string, start = 0, limit = 30) =>
    invoke<Page<Track>>("search", { query, start, limit }),

  album: (albumId: string, start = 0, limit = 50) =>
    invoke<AlbumPage>("album", { albumId, start, limit }),
  artist: (artistId: string) => invoke<ArtistPage>("artist", { artistId }),

  playback: (trackId: string, quality: "standard" | "lossless" = "standard") =>
    invoke<Playback>("playback", { trackId, quality }),
  lyrics: (trackId: string, title: string, artist: string) =>
    invoke<LyricsDoc>("lyrics", { trackId, title, artist }),

  favoriteExist: (trackId: string) => invoke<boolean>("favorite_exist", { trackId }),
  setFavorite: (trackId: string, favorite: boolean) =>
    invoke<void>("set_favorite", { trackId, favorite }),

  setCloseToTray: (enabled: boolean) => invoke<void>("set_close_to_tray", { enabled }),

  loadState: () => invoke<PersistedState>("load_state"),
  saveState: (snapshot: PersistedState) => invoke<void>("save_state", { snapshot }),
  lyricsCacheSize: () => invoke<number>("lyrics_cache_size"),
  clearLyricsCache: () => invoke<number>("clear_lyrics_cache"),
};

export function artistText(track: Pick<Track, "artists">): string {
  return track.artists.map((a) => a.name).join(" / ");
}

export function formatDuration(seconds?: number): string {
  if (!seconds || !Number.isFinite(seconds) || seconds <= 0) return "--:--";
  const total = Math.floor(seconds);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function maskDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return "0:00";
  const total = Math.floor(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}
