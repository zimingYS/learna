<script setup lang="ts">
import {onBeforeUnmount, onMounted, watch} from "vue";

const props = defineProps<{
  open: boolean;
  title: string;
  description?: string;
}>();

const emit = defineEmits<{ close: [] }>();

// Esc 关闭 + 打开期间锁滚动, 避免背景跟着滚
function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape" && props.open) {
    emit("close");
  }
}

function syncScrollLock(open: boolean): void {
  document.body.style.overflow = open ? "hidden" : "";
}

watch(() => props.open, syncScrollLock);

onMounted(() => window.addEventListener("keydown", handleKeydown));

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  document.body.style.overflow = "";
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="open" class="modal-backdrop" @click.self="emit('close')">
        <section class="modal-panel" role="dialog" aria-modal="true" :aria-label="title">
          <header class="modal-header">
            <div class="modal-heading">
              <h2>{{ title }}</h2>
              <p v-if="description">{{ description }}</p>
            </div>

            <button type="button" class="modal-close" aria-label="关闭" @click="emit('close')">
              ×
            </button>
          </header>

          <div class="modal-body">
            <slot/>
          </div>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  z-index: 60;
  display: flex;
  overflow-y: auto;
  align-items: flex-start;
  justify-content: center;

  inset: 0;
  padding: 64px 20px 40px;

  background: rgb(18 26 43 / 45%);
}

.modal-panel {
  width: 100%;
  max-width: 560px;
  border: 1px solid #e9edf4;
  border-radius: 16px;
  background: #fff;
  box-shadow: 0 24px 60px rgb(20 30 55 / 18%);
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;

  padding: 24px 26px 18px;
  border-bottom: 1px solid #f0f3f8;
}

.modal-heading h2 {
  margin: 0;
  color: #202b40;
  font-size: 18px;
  font-weight: 650;
}

.modal-heading p {
  margin: 7px 0 0;
  color: #8490a3;
  font-size: 13px;
  line-height: 1.6;
}

.modal-close {
  flex-shrink: 0;

  width: 30px;
  height: 30px;
  padding: 0;

  border: none;
  border-radius: 8px;

  background: transparent;
  color: #8c96a8;

  font-size: 20px;
  line-height: 1;
  cursor: pointer;

  transition: background-color 0.2s ease, color 0.2s ease;
}

.modal-close:hover {
  background: #f3f5fa;
  color: #495368;
}

.modal-close:focus-visible {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}

.modal-body {
  padding: 22px 26px 26px;
}

/* 遮罩淡入 + 面板上浮 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.18s ease;
}

.modal-enter-active .modal-panel,
.modal-leave-active .modal-panel {
  transition: transform 0.18s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-panel,
.modal-leave-to .modal-panel {
  transform: translateY(-8px);
}

@media (max-width: 600px) {
  .modal-backdrop {
    padding: 24px 12px;
  }

  .modal-header {
    padding: 20px 20px 16px;
  }

  .modal-body {
    padding: 18px 20px 22px;
  }
}
</style>
