<script setup lang="ts">
import { computed } from "vue";

/**
 * 手写图标集（24 网格 / 1.75 描边 / 圆角端点）。
 *
 * 之前的 1.6 描边画到 13–15px 时实际只有 0.9px，发虚、显小气。
 * 现在统一 1.75，并把实心图形（播放、暂停、上一首/下一首）单独按视觉重量调过：
 * 三角形的视觉面积比同尺寸的描边图形小，所以放大一档才和别的图标等重。
 */
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 });

const PATHS: Record<string, string> = {
  play: '<path d="M9 5.9v12.2L18.8 12z" fill="currentColor" stroke="none"/>',
  pause:
    '<rect x="7.6" y="5.5" width="3.2" height="13" rx="1.1" fill="currentColor" stroke="none"/><rect x="13.2" y="5.5" width="3.2" height="13" rx="1.1" fill="currentColor" stroke="none"/>',
  prev:
    '<path d="M18.4 6.7v10.6L10 12z" fill="currentColor" stroke="none"/><rect x="5.2" y="5.8" width="2.2" height="12.4" rx="1.1" fill="currentColor" stroke="none"/>',
  next:
    '<path d="M5.6 6.7v10.6L14 12z" fill="currentColor" stroke="none"/><rect x="16.6" y="5.8" width="2.2" height="12.4" rx="1.1" fill="currentColor" stroke="none"/>',
  heart:
    '<path d="M12 19.7l-1.1-1C6.4 14.9 4 12.7 4 10a4 4 0 0 1 4-3.9c1.5 0 2.9.8 4 2 1.1-1.2 2.5-2 4-2A4 4 0 0 1 20 10c0 2.7-2.4 4.9-6.9 8.7z"/>',
  "heart-filled":
    '<path d="M12 19.7l-1.1-1C6.4 14.9 4 12.7 4 10a4 4 0 0 1 4-3.9c1.5 0 2.9.8 4 2 1.1-1.2 2.5-2 4-2A4 4 0 0 1 20 10c0 2.7-2.4 4.9-6.9 8.7z" fill="currentColor"/>',
  search: '<circle cx="11" cy="11" r="6.3"/><path d="M15.7 15.7L20 20"/>',
  list: '<path d="M4 6.8h15.5M4 12h15.5M4 17.2h9.5"/>',
  "list-plus":
    '<path d="M4 6.8h13M4 12h13M4 17.2h8"/><path d="M18.4 12.6v6.2M15.3 15.7h6.2"/>',
  note: '<circle cx="7" cy="17.8" r="2.7"/><path d="M9.7 17.8V6.6L19 4.8v10.4"/><circle cx="16.3" cy="15.2" r="2.7"/>',
  back: '<path d="M14.6 5.4L8 12l6.6 6.6"/>',
  repeat:
    '<path d="M4.2 11.4V9.9a3.7 3.7 0 0 1 3.7-3.7h9.6"/><path d="M15.6 3.4l2.9 2.8-2.9 2.8"/><path d="M19.8 12.6v1.5a3.7 3.7 0 0 1-3.7 3.7H6.5"/><path d="M8.4 20.6l-2.9-2.8 2.9-2.8"/>',
  "repeat-one":
    '<path d="M4.2 11.4V9.9a3.7 3.7 0 0 1 3.7-3.7h9.6"/><path d="M15.6 3.4l2.9 2.8-2.9 2.8"/><path d="M19.8 12.6v1.5a3.7 3.7 0 0 1-3.7 3.7H6.5"/><path d="M8.4 20.6l-2.9-2.8 2.9-2.8"/><path d="M11.5 10.4l1.3-.9v4.9"/>',
  shuffle:
    '<path d="M4 6.6h3.4l8.4 10.8h3.6"/><path d="M4 17.4h3.4l2.6-3.4"/><path d="M16.4 3.4l3 3.2-3 3.2"/><path d="M16.4 14l3 3.2-3 3.2"/>',
  order:
    '<path d="M4 6.9h10M4 12h10M4 17.1h6.5"/><path d="M18 5.2v13.6"/><path d="M15.2 15.8L18 18.6l2.8-2.8"/>',
  volume:
    '<path d="M4.6 9.4h3.1l4.6-3.7v12.6l-4.6-3.7H4.6z"/><path d="M15.6 9.6a3.3 3.3 0 0 1 0 4.8"/><path d="M18 7.4a6.6 6.6 0 0 1 0 9.2"/>',
  mute: '<path d="M4.6 9.4h3.1l4.6-3.7v12.6l-4.6-3.7H4.6z"/><path d="M15.4 10l4.4 4.4M19.8 10l-4.4 4.4"/>',
  close: '<path d="M6.3 6.3l11.4 11.4M17.7 6.3L6.3 17.7"/>',
  "win-min": '<path d="M6.5 12h11"/>',
  "win-max": '<rect x="7" y="7" width="10" height="10" rx="1"/>',
  "win-restore":
    '<rect x="8.6" y="8.6" width="8.6" height="8.6" rx="1"/><path d="M6.8 15.2V8.2a1.4 1.4 0 0 1 1.4-1.4h7"/>',
  lyrics: '<path d="M4 7h16M4 12h10.5M4 17h13"/>',
  folder:
    '<path d="M4 7.6A1.6 1.6 0 0 1 5.6 6h3.3l2 2.3h7.5A1.6 1.6 0 0 1 20 9.9v7.5A1.6 1.6 0 0 1 18.4 19H5.6A1.6 1.6 0 0 1 4 17.4z"/>',
  clock: '<circle cx="12" cy="12" r="8.2"/><path d="M12 7.4V12l3.1 1.9"/>',
  /* 设置用「滑块」而不是齿轮：齿轮在小尺寸下容易糊成一团。 */
  gear:
    '<path d="M3.6 8.4h5.8M14.2 8.4h6.2"/><circle cx="11.8" cy="8.4" r="2.4"/><path d="M3.6 15.6h8.6M17 15.6h3.4"/><circle cx="14.6" cy="15.6" r="2.4"/>',
  /* 发现用罗盘，比四角星更有指向性。 */
  spark:
    '<circle cx="12" cy="12" r="8.2"/><path d="M15.5 8.5l-1.9 4.7-4.7 1.9 1.9-4.7z" fill="currentColor" stroke="none"/>',
};

const body = computed(() => PATHS[props.name] ?? "");
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.75"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="body"
  />
</template>
