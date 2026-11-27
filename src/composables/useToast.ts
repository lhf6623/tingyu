import { ref } from "vue";

export type Toast = { id: number; text: string; bad: boolean };

const toasts = ref<Toast[]>([]);
let nextId = 1;

function push(text: string, bad: boolean) {
  const id = nextId++;
  toasts.value = [...toasts.value, { id, text, bad }];
  const ttl = bad ? 6000 : 3000;
  setTimeout(() => {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  }, ttl);
}

export function useToast() {
  return {
    toasts,
    info: (text: string) => push(text, false),
    error: (text: string) => push(text, true),
  };
}
