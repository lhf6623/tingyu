<script setup lang="ts">
import { computed } from "vue";

/**
 * 手写图标集（24 网格 / 2.0 描边 / 圆头圆角）。
 *
 * 统一规格：描边 2、端点与拐角全部 round，矩形用 2–2.5 的圆角。
 * 实心图形（播放、暂停、上下首）同时给同色描边，靠 linejoin:round 把尖角磨圆，
 * 视觉重量才和描边图标一致。
 */
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 });

const ROUND =
  'stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"';

const PATHS: Record<string, string> = {
  play: `<path d="M9.6 6.8v10.4L18.2 12z" fill="currentColor" ${ROUND}/>`,
  pause:
    `<rect x="7.7" y="6.2" width="2.8" height="11.6" rx="1.4" fill="currentColor" ${ROUND}/>` +
    `<rect x="13.5" y="6.2" width="2.8" height="11.6" rx="1.4" fill="currentColor" ${ROUND}/>`,
  prev:
    `<path d="M18 6.9v10.2L10.6 12z" fill="currentColor" ${ROUND}/>` +
    `<rect x="5.4" y="6.2" width="2.2" height="11.6" rx="1.1" fill="currentColor" ${ROUND}/>`,
  next:
    `<path d="M6 6.9v10.2L13.4 12z" fill="currentColor" ${ROUND}/>` +
    `<rect x="16.4" y="6.2" width="2.2" height="11.6" rx="1.1" fill="currentColor" ${ROUND}/>`,
  heart:
    '<path d="M12 19.4l-1-1C6.6 14.8 4.3 12.6 4.3 10A3.9 3.9 0 0 1 8.2 6.1c1.5 0 2.9.8 3.8 2 1-1.2 2.4-2 3.9-2A3.9 3.9 0 0 1 19.7 10c0 2.6-2.3 4.8-6.7 8.4z"/>',
  "heart-filled":
    '<path d="M12 19.4l-1-1C6.6 14.8 4.3 12.6 4.3 10A3.9 3.9 0 0 1 8.2 6.1c1.5 0 2.9.8 3.8 2 1-1.2 2.4-2 3.9-2A3.9 3.9 0 0 1 19.7 10c0 2.6-2.3 4.8-6.7 8.4z" fill="currentColor"/>',
  search: '<circle cx="10.9" cy="10.9" r="6.1"/><path d="M15.4 15.4L19.8 19.8"/>',
  list: '<path d="M4.2 7h15.6M4.2 12h15.6M4.2 17h10"/>',
  "list-plus":
    '<path d="M4.2 7h13M4.2 12h13M4.2 17h8"/><path d="M18.6 12.6v6M15.6 15.6h6"/>',
  note: '<circle cx="7.2" cy="17.6" r="2.7"/><path d="M9.9 17.6V6.8L18.8 5v10"/><circle cx="16.1" cy="15" r="2.7"/>',
  back: '<path d="M14.4 5.8L8.2 12l6.2 6.2"/>',
  forward: '<path d="M9.6 5.8L15.8 12l-6.2 6.2"/>',
  /* 循环改成单环缺口 + 一个箭头：原来的双环双箭头在 16px 下会糊成一团。 */
  repeat: '<path d="M19.8 12a7.8 7.8 0 1 1-2.4-5.6"/><path d="M19.9 4.4v4.3h-4.3"/>',
  "repeat-one":
    '<path d="M19.8 12a7.8 7.8 0 1 1-2.4-5.6"/><path d="M19.9 4.4v4.3h-4.3"/><path d="M10.9 10.6l1.5-1v5"/>',
  shuffle:
    '<path d="M4.2 6.8h3.2l8.4 10.4h3.6"/><path d="M4.2 17.2h3.2l2.6-3.2"/><path d="M16.2 3.8l2.8 3-2.8 3"/><path d="M16.2 14.2l2.8 3-2.8 3"/>',
  order:
    '<path d="M4.2 7h9.6M4.2 12h9.6M4.2 17h6"/><path d="M18 5.4v13.2"/><path d="M15.2 15.8L18 18.6l2.8-2.8"/>',
  volume:
    '<path d="M4.9 9.6h3l4.4-3.6v12l-4.4-3.6H4.9z"/><path d="M15.6 9.8a3.3 3.3 0 0 1 0 4.4"/><path d="M18 7.6a6.4 6.4 0 0 1 0 8.8"/>',
  mute: '<path d="M4.9 9.6h3l4.4-3.6v12l-4.4-3.6H4.9z"/><path d="M15.4 10.2l4.2 4.2M19.6 10.2l-4.2 4.2"/>',
  close: '<path d="M6.6 6.6l10.8 10.8M17.4 6.6L6.6 17.4"/>',
  "win-min": '<path d="M6.6 12h10.8"/>',
  "win-max": '<rect x="7.2" y="7.2" width="9.6" height="9.6" rx="2.4"/>',
  "win-restore":
    '<rect x="9.2" y="9.2" width="7.6" height="7.6" rx="2"/><path d="M7.2 14.6V8.8a1.6 1.6 0 0 1 1.6-1.6h5.4"/>',
  /* 歌词：三段线和「列表」分不出来，改成带两行文字的气泡。 */
  lyrics:
    '<path d="M4.6 7.6A2.4 2.4 0 0 1 7 5.2h10a2.4 2.4 0 0 1 2.4 2.4v6.2a2.4 2.4 0 0 1-2.4 2.4h-5.8l-3.4 2.8v-2.8H7a2.4 2.4 0 0 1-2.4-2.4z"/><path d="M8.6 9.4h6.8M8.6 12.4h4.4"/>',
  folder:
    '<path d="M4.2 7.8A1.8 1.8 0 0 1 6 6h3.2l2 2.3h7A1.8 1.8 0 0 1 20 10.1v7.3a1.8 1.8 0 0 1-1.8 1.8H6a1.8 1.8 0 0 1-1.8-1.8z"/>',
  clock: '<circle cx="12" cy="12" r="8"/><path d="M12 7.6V12l3 1.8"/>',
  /* 设置用「滑块」而不是齿轮：齿轮在小尺寸下齿牙会粘连，滑块全是圆头。 */
  gear:
    '<path d="M3.8 8.4h5.6M14.4 8.4h5.8"/><circle cx="11.9" cy="8.4" r="2.3" fill="currentColor" stroke="none"/>' +
    '<path d="M3.8 15.6h8.2M17.6 15.6h2.6"/><circle cx="14.9" cy="15.6" r="2.3" fill="currentColor" stroke="none"/>',
  /* 发现用罗盘：描边菱形针 + 中心点，实心针会读成「禁止」。 */
  /* 发现用四角星：罗盘在小尺寸下会和「禁止」混淆，星形没有歧义。 */
  spark:
    '<path d="M12 4.4l1.9 5.7 5.7 1.9-5.7 1.9L12 19.6l-1.9-5.7L4.4 12l5.7-1.9z"/>',
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
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="body"
  />
</template>
