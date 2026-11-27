import { ref } from "vue";

/**
 * 页面数据缓存的版本号。
 *
 * 各页面被 KeepAlive 缓存后不会自己重新请求，这里用版本号做失效标记：
 * `App.vue` 把它拼进视图的 key，值一变所有页面实例重建，下次进入即重新拉取。
 * 切换账号和手动刷新都走这里，避免出现两套失效逻辑。
 */
const version = ref(0);

function invalidate() {
  version.value += 1;
}

export function useCache() {
  return { version, invalidate };
}
