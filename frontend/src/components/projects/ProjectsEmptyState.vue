<script setup lang="ts">
defineProps<{
  // true = 有筛选条件但没结果，文案与操作都不同
  filtered: boolean;
}>();

const emit = defineEmits<{
  create: [];
  reset: [];
}>();
</script>

<template>
  <section class="empty-state">
    <div class="empty-icon">
      <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
           stroke-linejoin="round" stroke-width="1.5" viewBox="0 0 24 24">
        <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>
      </svg>
    </div>

    <template v-if="filtered">
      <h2>该筛选下暂无项目</h2>
      <p>换个状态看看，或清除筛选回到全部项目。</p>
      <button class="ghost-btn" type="button" @click="emit('reset')">清除筛选</button>
    </template>

    <template v-else>
      <h2>还没有学习项目</h2>
      <p>先建一个项目作为容器，之后的目标、阶段与任务都挂在它下面。</p>
      <button class="create-btn" type="button" @click="emit('create')">
        <span aria-hidden="true" class="plus-icon">+</span>
        创建第一个项目
      </button>
    </template>
  </section>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;

  padding: 56px 24px;

  border: 1px dashed #dde3ee;
  border-radius: 16px;

  background: #fff;
  text-align: center;
}

.empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 54px;
  height: 54px;
  margin-bottom: 16px;

  border-radius: 16px;

  background: #eef1fe;
  color: #4f46e5;
}

.empty-icon svg {
  width: 26px;
  height: 26px;
}

.empty-state h2 {
  margin: 0;

  color: #263248;
  font-size: 17px;
  font-weight: 650;
}

.empty-state p {
  max-width: 420px;
  margin: 8px 0 20px;

  color: #8490a3;
  font-size: 13px;
  line-height: 1.7;
}

.create-btn {
  display: inline-flex;
  min-height: 42px;
  align-items: center;
  gap: 8px;
  padding: 0 18px;

  border: none;
  border-radius: 9px;

  background: #4f46e5;
  box-shadow: 0 3px 10px rgb(79 70 229 / 22%);
  color: #fff;

  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.create-btn:hover {
  background: #4338ca;
}

.plus-icon {
  font-size: 20px;
  font-weight: 400;
  line-height: 1;
}

.ghost-btn {
  min-height: 38px;
  padding: 0 16px;

  border: 1px solid #dce2eb;
  border-radius: 8px;

  background: #fff;
  color: #596579;

  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.ghost-btn:hover {
  background: #f8f9fc;
}

.create-btn:focus-visible,
.ghost-btn:focus-visible {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}
</style>
