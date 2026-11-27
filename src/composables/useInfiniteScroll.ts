import { onBeforeUnmount, ref, watch } from "vue";

/**
 * 列表滚到底自动续页。
 *
 * 哨兵元素进入视口（提前 300px）即触发加载；组件里只需要把 `sentinel` 绑到列表末尾的空元素上，
 * 并在不需要继续加载时用 v-if 把它移除（移除会自动断开观察器）。
 * `canLoad` 由调用方决定，用来挡住正在加载、已到末页、上次失败还没重试这些情况。
 */
export function useInfiniteScroll(canLoad: () => boolean, load: () => void) {
  const sentinel = ref<HTMLElement | null>(null);
  let observer: IntersectionObserver | undefined;

  watch(sentinel, (element) => {
    observer?.disconnect();
    if (!element) return;
    observer = new IntersectionObserver(
      (entries) => {
        if (!entries[0]?.isIntersecting) return;
        if (!canLoad()) return;
        load();
      },
      { rootMargin: "300px" },
    );
    observer.observe(element);
  });

  onBeforeUnmount(() => observer?.disconnect());

  return { sentinel };
}
