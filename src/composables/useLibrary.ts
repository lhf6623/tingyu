import { ref } from "vue";
import type { Track } from "../api";

/**
 * 收藏状态以服务端返回的 `isFavorite` 为初始值，这里只记录本次会话内的改动。
 * 收藏列表 / 搜索结果都带了该字段，所以不需要先拉一遍收藏全集。
 */
const overrides = ref(new Map<string, boolean>());

function isFavorite(track: Track): boolean {
  return overrides.value.get(track.id) ?? track.favorite ?? false;
}

function setFavorite(trackId: string, favorite: boolean) {
  overrides.value = new Map(overrides.value).set(trackId, favorite);
}

export function useLibrary() {
  return { isFavorite, setFavorite };
}
