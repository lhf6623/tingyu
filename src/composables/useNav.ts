import { ref } from "vue";

export type Route =
  | { name: "discover" }
  | { name: "favorites" }
  | { name: "search" }
  | { name: "settings" }
  | { name: "playlist"; id: string; kind: string; title: string; coverUrl?: string }
  | { name: "album"; id: string; title: string; coverUrl?: string }
  | { name: "artist"; id: string; title: string };

const route = ref<Route>({ name: "discover" });
const stack = ref<Route[]>([]);

export function useNav() {
  function go(next: Route) {
    stack.value.push(route.value);
    route.value = next;
  }

  /** 切换主页面时清掉详情栈，避免"返回"把用户带到别的栏目里。 */
  function goRoot(next: Route) {
    stack.value = [];
    route.value = next;
  }

  function back() {
    const previous = stack.value.pop();
    if (previous) route.value = previous;
    else route.value = { name: "discover" };
  }

  return {
    route,
    canGoBack: () => stack.value.length > 0,
    go,
    goRoot,
    back,
  };
}
