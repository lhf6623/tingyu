import { computed, reactive } from "vue";
import { api, artistText, errorText, type LyricsDoc, type PersistedState, type Track } from "../api";
import { useToast } from "./useToast";

export type PlayMode = "order" | "repeat-all" | "repeat-one" | "shuffle";

const toast = useToast();
const audio = new Audio();
audio.preload = "auto";

const lyricCache = new Map<string, LyricsDoc | null>();

const state = reactive({
  queue: [] as Track[],
  cursor: -1,
  playing: false,
  buffering: false,
  position: 0,
  duration: 0,
  volume: 0.8,
  mode: "repeat-all" as PlayMode,
  quality: "standard" as "standard" | "lossless",
  actualQuality: "unknown" as "standard" | "lossless" | "unknown",
  shuffleOrder: [] as number[],
  lyrics: null as LyricsDoc | null,
  lyricsLoading: false,
  queueOpen: false,
  closeToTray: true,
});

audio.volume = state.volume;

/** hydrate 之前不改写磁盘，否则默认值会覆盖上一次真正的现场。 */
let hydrated = false;
let saveTimer: number | undefined;

function snapshot(): PersistedState {
  return {
    volume: state.volume,
    mode: state.mode,
    quality: state.quality,
    queueOpen: state.queueOpen,
    closeToTray: state.closeToTray,
    cursor: state.cursor,
    queue: state.queue,
  };
}

function scheduleSave() {
  if (!hydrated) return;
  if (saveTimer) window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    void api.saveState(snapshot()).catch(() => undefined);
  }, 400);
}

/** 恢复上次的队列、当前曲和播放偏好。不自动播放，避免一开软件就出声。 */
async function hydrate() {
  if (hydrated) return;
  try {
    const saved = await api.loadState();
    // 用户在读取返回前就点了播放，就别用旧现场覆盖他刚建的新队列。
    if (state.queue.length === 0 && state.cursor === -1) {
      state.volume = saved.volume;
      state.mode = saved.mode as PlayMode;
      state.quality = saved.quality as "standard" | "lossless";
      state.queueOpen = saved.queueOpen;
      state.closeToTray = saved.closeToTray !== false;
      state.queue = saved.queue;
      state.cursor = saved.cursor >= 0 && saved.cursor < saved.queue.length ? saved.cursor : -1;
      audio.volume = state.volume;
      // 随机模式依赖洗牌表，不重建的话恢复后按"下一首"会没有反应。
      rebuildShuffle(state.cursor >= 0 ? state.cursor : undefined);
      const track = state.cursor >= 0 ? saved.queue[state.cursor] : null;
      if (track) state.duration = track.durationSeconds ?? 0;
    }
  } catch {
    // 配置缺失或损坏就按默认值启动。
  }
  hydrated = true;
}

const current = computed<Track | null>(() =>
  state.cursor >= 0 && state.cursor < state.queue.length ? state.queue[state.cursor] : null,
);

const progress = computed(() => (state.duration > 0 ? state.position / state.duration : 0));

function rebuildShuffle(keep?: number) {
  const order = state.queue.map((_, index) => index);
  for (let i = order.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [order[i], order[j]] = [order[j], order[i]];
  }
  if (keep !== undefined) {
    const at = order.indexOf(keep);
    if (at > 0) [order[0], order[at]] = [order[at], order[0]];
  }
  state.shuffleOrder = order;
}

async function loadLyrics(track: Track) {
  const cached = lyricCache.get(track.id);
  if (cached !== undefined) {
    state.lyrics = cached;
    return;
  }
  state.lyricsLoading = true;
  state.lyrics = null;
  try {
    const doc = await api.lyrics(track.id, track.title, artistText(track));
    const value = doc.lines.length ? doc : null;
    lyricCache.set(track.id, value);
    if (current.value?.id === track.id) state.lyrics = value;
  } catch {
    // 歌词拿不到不影响播放，也不弹错。
    lyricCache.set(track.id, null);
  } finally {
    state.lyricsLoading = false;
  }
}

async function loadAndPlay(index: number) {
  const track = state.queue[index];
  if (!track) return;
  state.cursor = index;
  state.position = 0;
  state.duration = track.durationSeconds ?? 0;
  state.buffering = true;
  scheduleSave();
  void loadLyrics(track);
  try {
    const result = await api.playback(track.id, state.quality);
    // 请求期间用户可能已经切歌，过期的响应直接丢掉。
    if (state.cursor !== index) return;
    state.actualQuality = result.actualQuality;
    audio.src = result.url;
    if (result.durationSeconds) state.duration = result.durationSeconds;
    await audio.play();
  } catch (error) {
    state.buffering = false;
    toast.error(`播放失败：${errorText(error)}`);
  }
}

function playList(tracks: Track[], startIndex = 0) {
  if (!tracks.length) return;
  state.queue = [...tracks];
  state.cursor = Math.min(Math.max(startIndex, 0), tracks.length - 1);
  rebuildShuffle(state.cursor);
  void loadAndPlay(state.cursor);
}

function playAt(index: number) {
  if (index < 0 || index >= state.queue.length) return;
  if (index === state.cursor) {
    void toggle();
    return;
  }
  void loadAndPlay(index);
}

/** 队列里点当前行不该把歌暂停，只跳转其它曲目。 */
function selectAt(index: number) {
  if (index < 0 || index >= state.queue.length || index === state.cursor) return;
  void loadAndPlay(index);
}

function stopPlayback() {
  audio.pause();
  audio.removeAttribute("src");
  state.playing = false;
  state.position = 0;
  state.duration = 0;
}

/**
 * 移除队列中的一首。删的是正在播的那首就顺位放下一首（末尾则回上一首），
 * 删的是前面的曲目要把游标一起前移，否则 cursor 会指向错的人。
 */
function removeAt(index: number) {
  if (index < 0 || index >= state.queue.length) return;
  const wasCurrent = index === state.cursor;
  state.queue.splice(index, 1);
  if (wasCurrent) {
    if (!state.queue.length) {
      state.cursor = -1;
      stopPlayback();
    } else {
      rebuildShuffle();
      void loadAndPlay(Math.min(index, state.queue.length - 1));
    }
  } else if (index < state.cursor) {
    state.cursor -= 1;
    rebuildShuffle(state.cursor);
  }
  scheduleSave();
}

function clearQueue() {
  state.queue = [];
  state.cursor = -1;
  state.shuffleOrder = [];
  stopPlayback();
  scheduleSave();
}

/** 把一首歌插到当前曲后面，不动正在播的这首。 */
function playNext(track: Track) {
  if (!state.queue.length) {
    playList([track], 0);
    return;
  }
  const already = state.queue.findIndex((item) => item.id === track.id);
  if (already >= 0) {
    // 已在队列里就把它挪到当前曲后面，避免重复入队。
    state.queue.splice(already, 1);
    if (already <= state.cursor) state.cursor -= 1;
  }
  const insertAt = Math.min(state.cursor + 1, state.queue.length);
  state.queue.splice(insertAt, 0, track);
  rebuildShuffle(state.cursor);
  scheduleSave();
}

function appendToQueue(track: Track) {
  if (state.queue.some((item) => item.id === track.id)) return;
  state.queue.push(track);
  if (state.cursor < 0) {
    void loadAndPlay(state.queue.length - 1);
  } else {
    rebuildShuffle(state.cursor);
  }
  scheduleSave();
}

function advance(auto: boolean): number | null {
  const total = state.queue.length;
  if (!total) return null;
  if (state.mode === "repeat-one" && auto) return state.cursor;
  if (state.mode === "shuffle") {
    const at = state.shuffleOrder.indexOf(state.cursor);
    if (at + 1 < state.shuffleOrder.length) return state.shuffleOrder[at + 1];
    if (auto) {
      rebuildShuffle();
      return state.shuffleOrder[0] ?? null;
    }
    return state.shuffleOrder[0] ?? null;
  }
  const next = state.cursor + 1;
  if (next < total) return next;
  return auto && state.mode === "order" ? null : 0;
}

function next() {
  const index = advance(false);
  if (index !== null) void loadAndPlay(index);
}

function previous() {
  if (state.position > 3) {
    audio.currentTime = 0;
    return;
  }
  const total = state.queue.length;
  if (!total) return;
  const index = state.cursor <= 0 ? total - 1 : state.cursor - 1;
  void loadAndPlay(index);
}

async function toggle() {
  if (!current.value) return;
  if (!audio.src) {
    void loadAndPlay(state.cursor);
    return;
  }
  if (audio.paused) {
    try {
      await audio.play();
    } catch (error) {
      toast.error(`播放失败：${errorText(error)}`);
    }
  } else {
    audio.pause();
  }
}

function seek(ratio: number) {
  if (!Number.isFinite(audio.duration) || audio.duration <= 0) return;
  audio.currentTime = Math.min(Math.max(ratio, 0), 1) * audio.duration;
  state.position = audio.currentTime;
}

function setVolume(value: number) {
  const clamped = Math.min(Math.max(value, 0), 1);
  state.volume = clamped;
  audio.volume = clamped;
  scheduleSave();
}

function setQuality(quality: "standard" | "lossless") {
  state.quality = quality;
  scheduleSave();
  const index = state.cursor;
  if (index >= 0 && !audio.paused) void loadAndPlay(index);
}

function cycleMode() {
  const order: PlayMode[] = ["repeat-all", "order", "repeat-one", "shuffle"];
  const at = order.indexOf(state.mode);
  state.mode = order[(at + 1) % order.length];
  if (state.mode === "shuffle") rebuildShuffle(state.cursor >= 0 ? state.cursor : undefined);
  scheduleSave();
}

function setCloseToTray(enabled: boolean) {
  state.closeToTray = enabled;
  void api.setCloseToTray(enabled).catch(() => undefined);
  scheduleSave();
}

function toggleQueue() {
  state.queueOpen = !state.queueOpen;
  scheduleSave();
}

audio.addEventListener("timeupdate", () => {
  state.position = audio.currentTime;
});
audio.addEventListener("loadedmetadata", () => {
  if (Number.isFinite(audio.duration)) state.duration = audio.duration;
});
audio.addEventListener("play", () => {
  state.playing = true;
  state.buffering = false;
});
audio.addEventListener("pause", () => {
  state.playing = false;
});
audio.addEventListener("waiting", () => {
  state.buffering = true;
});
audio.addEventListener("playing", () => {
  state.buffering = false;
});
audio.addEventListener("ended", () => {
  if (state.mode === "repeat-one") {
    audio.currentTime = 0;
    void audio.play();
    return;
  }
  const index = advance(true);
  if (index === null) {
    state.playing = false;
    return;
  }
  void loadAndPlay(index);
});
audio.addEventListener("error", () => {
  if (!audio.src) return;
  state.buffering = false;
  toast.error("音频加载失败，可能是网络或资源地址已过期");
});

export function usePlayer() {
  return {
    state,
    current,
    progress,
    hydrate,
    playList,
    playAt,
    selectAt,
    playNext,
    appendToQueue,
    removeAt,
    clearQueue,
    toggle,
    next,
    previous,
    seek,
    setVolume,
    setQuality,
    cycleMode,
    toggleQueue,
    setCloseToTray,
  };
}
