import { ref } from "vue";
import { api, errorText, type Page, type Track } from "../api";
import { useCache } from "./useCache";
import { useToast } from "./useToast";

/**
 * 搜索相关的共享状态。
 *
 * 标题栏输入框、下拉候选、结果页要读写同一份数据，靠路由参数传会丢：
 * 同一个路由重复进入不会重建组件，`onMounted` 也不会再跑。
 * 所以关键词、结果、分页、历史全部收敛到这里，组件只负责渲染和触发。
 */
const HISTORY_KEY = "search-history";
const HISTORY_LIMIT = 8;
const SUGGEST_LIMIT = 6;

const toast = useToast();
const pageCacheOwner = useCache();

/**
 * 结果页缓存：同一关键词 + 同一页在 TTL 内直接复用。
 * 上下切页面、删一个字再打回来都是高频重复查询，重复打请求没意义。
 * 缓存键里带上 `useCache().version`，设置页点「刷新」时自动整体失效。
 */
const CACHE_TTL_MS = 5 * 60 * 1000;
const CACHE_MAX = 40;
type CachedPage = { at: number; version: number; page: Page<Track> };
const pageCache = new Map<string, CachedPage>();

function cacheKey(query: string, start: number, limit: number) {
  return `${limit}:${start}:${query}`;
}

function readCache(key: string): Page<Track> | null {
  const hit = pageCache.get(key);
  if (!hit) return null;
  if (hit.version !== pageCacheOwner.version.value || Date.now() - hit.at > CACHE_TTL_MS) {
    pageCache.delete(key);
    return null;
  }
  // 命中后挪到队尾，实现 LRU 淘汰。
  pageCache.delete(key);
  pageCache.set(key, hit);
  return hit.page;
}

function writeCache(key: string, page: Page<Track>) {
  pageCache.set(key, { at: Date.now(), version: pageCacheOwner.version.value, page });
  while (pageCache.size > CACHE_MAX) {
    const oldest = pageCache.keys().next().value;
    if (oldest === undefined) break;
    pageCache.delete(oldest);
  }
}

const keyword = ref("");
const submitted = ref("");
const results = ref<Track[]>([]);
const loading = ref(false);
const nextStart = ref(0);
const hasMore = ref(false);
const suggestions = ref<Track[]>([]);
const suggesting = ref(false);
const history = ref<string[]>(readHistory());

// 候选请求会乱序返回，用序号丢弃过期响应。
let suggestSeq = 0;

function readHistory(): string[] {
  try {
    const raw = JSON.parse(localStorage.getItem(HISTORY_KEY) ?? "[]");
    return Array.isArray(raw)
      ? raw.filter((item): item is string => typeof item === "string").slice(0, HISTORY_LIMIT)
      : [];
  } catch {
    return [];
  }
}

function persistHistory() {
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value));
  } catch {
    // 存不进去不影响搜索。
  }
}

function remember(query: string) {
  const text = query.trim();
  if (!text) return;
  history.value = [text, ...history.value.filter((item) => item !== text)].slice(0, HISTORY_LIMIT);
  persistHistory();
}

function clearHistory() {
  history.value = [];
  persistHistory();
}

async function runSearch(query?: string, reset = true) {
  const text = (query ?? keyword.value).trim();
  if (!text || loading.value) return;
  keyword.value = text;

  const start = reset ? 0 : nextStart.value;
  const key = cacheKey(text, start, 30);
  const cached = readCache(key);
  if (cached) {
    // 命中也走同一套赋值逻辑，保证分页游标、当前关键词一致。
    applyPage(text, cached, reset);
    return;
  }

  loading.value = true;
  try {
    const page = await api.search(text, start, 30);
    writeCache(key, page);
    applyPage(text, page, reset);
  } catch (error) {
    toast.error(`搜索失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
}

function applyPage(text: string, page: Page<Track>, reset: boolean) {
  results.value = reset ? page.items : [...results.value, ...page.items];
  nextStart.value = page.nextStart;
  hasMore.value = page.hasMore;
  submitted.value = text;
  if (reset) remember(text);
}

/** 下拉候选：防抖在调用方做，这里只保证响应顺序。 */
async function suggest(query: string) {
  const text = query.trim();
  const seq = ++suggestSeq;
  if (!text) {
    suggestions.value = [];
    suggesting.value = false;
    return;
  }
  const key = cacheKey(text, 0, SUGGEST_LIMIT);
  const cached = readCache(key);
  if (cached) {
    if (seq === suggestSeq) {
      suggestions.value = cached.items;
      suggesting.value = false;
    }
    return;
  }

  suggesting.value = true;
  try {
    const page = await api.search(text, 0, SUGGEST_LIMIT);
    writeCache(key, page);
    if (seq === suggestSeq) suggestions.value = page.items;
  } catch {
    if (seq === suggestSeq) suggestions.value = [];
  } finally {
    if (seq === suggestSeq) suggesting.value = false;
  }
}

/** 回到空白搜索页：清掉关键词、结果和候选，历史保留。 */
function reset() {
  keyword.value = "";
  submitted.value = "";
  results.value = [];
  nextStart.value = 0;
  hasMore.value = false;
  suggestions.value = [];
  suggestSeq += 1;
}

export function useSearch() {
  return {
    keyword,
    submitted,
    results,
    loading,
    hasMore,
    suggestions,
    suggesting,
    history,
    runSearch,
    suggest,
    remember,
    clearHistory,
    reset,
  };
}
