import { computed, ref } from "vue";

export type Route =
  | { name: "discover" }
  | { name: "favorites" }
  | { name: "search" }
  | { name: "settings" }
  | { name: "playlist"; id: string; kind: string; title: string; coverUrl?: string }
  | { name: "album"; id: string; title: string; coverUrl?: string }
  | { name: "artist"; id: string; title: string };

const route = ref<Route>({ name: "discover" });
/** 返回栈：进入详情页时压入的来源页。 */
const backStack = ref<Route[]>([]);
/** 前进栈：被返回操作让出的页面，前进时按后进先出取回。 */
const forwardStack = ref<Route[]>([]);

export function useNav() {
  /** 新导航会截断前进历史，和浏览器的地址栏行为一致。 */
  function go(next: Route) {
    backStack.value.push(route.value);
    forwardStack.value = [];
    route.value = next;
  }

  /** 切换主栏目（发现/收藏/设置/搜索）也压栈，让返回按钮能逐页回退。 */
  function goRoot(next: Route) {
    go(next);
  }

  /** 硬重置：登录/登出后清空全部历史，避免返回到上一个会话的页面。 */
  function reset(next: Route = { name: "discover" }) {
    backStack.value = [];
    forwardStack.value = [];
    route.value = next;
  }

  function back() {
    const previous = backStack.value.pop();
    if (!previous) return;
    forwardStack.value.push(route.value);
    route.value = previous;
  }

  function forward() {
    const next = forwardStack.value.pop();
    if (!next) return;
    backStack.value.push(route.value);
    route.value = next;
  }

  return {
    route,
    // 响应式：标题栏按钮据此启用/禁用
    canBack: computed(() => backStack.value.length > 0),
    canForward: computed(() => forwardStack.value.length > 0),
    go,
    goRoot,
    reset,
    back,
    forward,
  };
}
