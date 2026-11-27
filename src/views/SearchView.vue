<script setup lang="ts">
import { computed } from "vue";
import Icon from "../components/Icon.vue";
import TrackList from "../components/TrackList.vue";
import { type Track } from "../api";
import { usePlayer } from "../composables/usePlayer";
import { useInfiniteScroll } from "../composables/useInfiniteScroll";
import { useSearch } from "../composables/useSearch";

const player = usePlayer();
const { current } = player;
const search = useSearch();
const { keyword, submitted, results, loading, hasMore } = search;

/** 有过搜索就只显示结果，输入框只在空白页出现。 */
const compact = computed(() => !!submitted.value || loading.value);

function submit() {
  void search.runSearch(keyword.value, true);
}

function loadMore() {
  void search.runSearch(submitted.value, false);
}

const { sentinel } = useInfiniteScroll(
  () => hasMore.value && !loading.value,
  loadMore,
);

function play(track: Track, index: number) {
  player.playList(results.value, index);
}
</script>

<template>
  <section class="page">
    <div v-if="!compact" class="hero">
      <h1>搜索</h1>
      <label class="field">
        <Icon name="search" :size="21" />
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索歌曲、艺人"
          spellcheck="false"
          @keyup.enter="submit"
        />
        <button v-if="keyword" class="clear" title="清空" @click="keyword = ''">
          <Icon name="close" :size="15" />
        </button>
      </label>
      <p class="tip">输入歌名或艺人，回车开始搜索</p>
    </div>

    <template v-else>
      <header class="result-head">
        <h2 class="ellipsis">“{{ submitted || keyword }}”</h2>
        <span v-if="results.length" class="count">已加载 {{ results.length }} 首</span>
      </header>

      <TrackList
        :tracks="results"
        :current-id="current?.id"
        :loading="loading && !results.length"
        empty-text="没有找到相关歌曲"
        @play="play"
      />

      <div v-if="hasMore || loading" ref="sentinel" class="sentinel">
        <span v-if="loading">加载中…</span>
      </div>
    </template>
  </section>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding-bottom: 24px;
  min-height: 0;
}

.hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 12vh 0 34px;
}

.hero h1 {
  font-size: 26px;
  font-weight: 500;
  letter-spacing: 0.5px;
}

.field {
  display: flex;
  align-items: center;
  gap: 12px;
  width: min(620px, 100%);
  height: 52px;
  padding: 0 18px;
  border-radius: var(--r-xl);
  color: var(--faint);
  /* 大搜索框靠「浅灰下沉 → 聚焦时浮成白卡 + 投影」表达状态，没有描边 */
  background: var(--surface);
  box-shadow: 0 6px 22px rgba(0, 0, 0, 0.05);
  transition: background 0.15s ease, box-shadow 0.15s ease;
}

.field:focus-within {
  background: var(--bg);
  box-shadow: var(--pop-shadow);
}

.field input {
  flex: 1;
  min-width: 0;
  font-size: 16px;
  color: var(--text);
}

.field input::placeholder {
  color: var(--faint);
}

.clear {
  display: flex;
  color: var(--faint);
}

.clear:hover {
  color: var(--text);
}

.tip {
  font-size: 12px;
  color: var(--faint);
}

.result-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 8px;
  min-width: 0;
}

.result-head h2 {
  font-size: 18px;
  font-weight: 500;
  min-width: 0;
}

.count {
  flex: none;
  font-size: 11px;
  color: var(--faint);
}

.sentinel {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 48px;
  font-size: 12px;
  color: var(--faint);
}
</style>
