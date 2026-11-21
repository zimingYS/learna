import {defineStore} from "pinia";
import {ref} from "vue";

export const useAppStore = defineStore("app", () => {
  const title = ref("学习轨迹");

  function setTitle(next: string) {
    title.value = next;
  }

  return {title, setTitle};
});
