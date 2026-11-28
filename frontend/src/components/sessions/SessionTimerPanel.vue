<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref} from "vue";
import {useSessionsStore, type TimerSnapshot} from "../../stores/sessions";
import {formatElapsed} from "../../views/sessions/session-rules";

const emit = defineEmits<{ finish: [snapshot: TimerSnapshot] }>();

const store = useSessionsStore();

// 计时状态只在客户端内存里；这里只负责每秒刷新显示
const now = ref(Date.now());
let ticker: number | undefined;

const elapsed = computed(() => store.timerElapsedMs(now.value));

onMounted(() => {
  ticker = window.setInterval(() => {
    now.value = Date.now();
  }, 1000);
});

onBeforeUnmount(() => {
  if (ticker !== undefined) {
    window.clearInterval(ticker);
  }
});

function toggle(): void {
  if (store.timerRunning) {
    store.pauseTimer();
  } else {
    store.startTimer();
  }

  now.value = Date.now();
}

function finish(): void {
  const snapshot = store.timerSnapshot(now.value);

  if (!snapshot) {
    return;
  }

  emit("finish", snapshot);
  store.resetTimer();
  now.value = Date.now();
}

function reset(): void {
  store.resetTimer();
  now.value = Date.now();
}
</script>

<template>
  <section class="timer-panel">
    <div class="timer-head">
      <div>
        <h2>计时器</h2>
        <p>本地计时：开始在学、暂停算休息，结束后把起止时间一次性提交。</p>
      </div>

      <span v-if="store.timerRunning" class="timer-dot" aria-label="计时中"/>
    </div>

    <p class="clock">{{ formatElapsed(elapsed) }}</p>

    <div class="timer-actions">
      <button class="pf-btn pf-btn--primary" type="button" @click="toggle">
        {{ store.timerRunning ? "暂停" : store.timerStarted ? "继续" : "开始计时" }}
      </button>

      <button
        :disabled="!store.timerStarted"
        class="pf-btn pf-btn--ghost"
        type="button"
        @click="finish"
      >
        记录这次
      </button>

      <button
        :disabled="!store.timerStarted"
        class="pf-btn pf-btn--ghost"
        type="button"
        @click="reset"
      >
        重置
      </button>
    </div>

    <p class="timer-hint">
      计时状态只保存在本页内存里，刷新会丢失；这是刻意的——服务端不存 timer（ADR-010 §2）。
    </p>
  </section>
</template>

<style scoped>
.timer-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 18px 20px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: linear-gradient(150deg, #fbfbff, #f5f4ff);
}

.timer-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.timer-head h2 {
  margin: 0;
  color: var(--ink-1);
  font-size: 15px;
  font-weight: 660;
}

.timer-head p {
  margin: 5px 0 0;
  color: var(--ink-3);
  font-size: 12px;
  line-height: 1.6;
}

.timer-dot {
  flex-shrink: 0;

  width: 9px;
  height: 9px;
  margin-top: 6px;

  border-radius: 50%;

  background: #ef4444;
  box-shadow: 0 0 0 4px rgb(239 68 68 / 12%);
}

.clock {
  margin: 0;

  color: var(--ink-1);
  font-size: 30px;
  font-variant-numeric: tabular-nums;
  font-weight: 700;
  letter-spacing: 1px;
}

.timer-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.timer-actions .pf-btn {
  min-height: 34px;
  padding: 0 14px;
  font-size: 12.5px;
}

.timer-hint {
  margin: 0;
  color: var(--ink-4);
  font-size: 11.5px;
  line-height: 1.6;
}
</style>
