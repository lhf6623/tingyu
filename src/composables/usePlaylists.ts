import { ref } from "vue";
import { api, errorText, type Playlist } from "../api";
import { useCache } from "./useCache";
import { useToast } from "./useToast";

/**
 * 侧栏歌单列表的共享状态。
 *
 * 侧栏不在 KeepAlive 里——进歌词页时会整个卸载（沉浸模式收起左栏），
 * 退出时重新挂载。数据放在组件内就等于每次进出都重拉一遍，
 * 所以挪到模块级单例，和 usePlayer / useNav 同一套 store 模式。
 */
const items = ref<Playlist[]>([]);
const loading = ref(false);
/** 上次成功拉取时的 cache 版本；与当前版本不一致即视为过期。 */
const loadedVersion = ref(-1);
let inflight: Promise<void> | null = null;

export function usePlaylists() {
  const cache = useCache();
  const toast = useToast();

  function isFresh() {
    return loadedVersion.value === cache.version.value;
  }

  /**
   * 拉取歌单。`force` 用于「设置 → 刷新」这类显式动作。
   * 并发调用共用同一个请求，避免切页面时打两遍。
   */
  function load(force = false): Promise<void> {
    if (!force && isFresh()) return Promise.resolve();
    if (inflight) return inflight;

    loading.value = true;
    const version = cache.version.value;
    inflight = (async () => {
      try {
        items.value = await api.playlists();
        loadedVersion.value = version;
      } catch (error) {
        toast.error(`加载歌单失败：${errorText(error)}`);
      } finally {
        loading.value = false;
        inflight = null;
      }
    })();
    return inflight;
  }

  /** 组件挂载时调用：已有数据就直接用，没有才发请求。 */
  async function ensure() {
    if (isFresh()) return;
    await load();
  }

  /** 退出登录时清空，避免下一个账号看到上一个账号的歌单。 */
  function reset() {
    items.value = [];
    loadedVersion.value = -1;
    inflight = null;
  }

  return { items, loading, load, ensure, reset, isFresh };
}
