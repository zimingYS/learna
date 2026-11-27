<script setup lang="ts">
const emit = defineEmits<{ purge: [] }>();
</script>

<template>
  <section class="card" aria-labelledby="danger-title">
    <h2 id="danger-title">彻底删除</h2>
    <p class="card-hint">
      需要重输项目名才能执行。规划数据会被永久删除，学习记录只解除归属关系。
    </p>

    <button class="danger-btn" type="button" @click="emit('purge')">彻底删除项目</button>
  </section>
</template>

<style scoped>
.card {
  padding: 22px 24px;

  border: 1px solid #f3dede;
  border-radius: 14px;

  background: #fff;
  box-shadow: 0 2px 10px rgb(31 45 72 / 4%);
}

.card h2 {
  margin: 0 0 14px;

  color: #263248;
  font-size: 16px;
  font-weight: 650;
}

.card-hint {
  margin: 0 0 14px;
  color: #8490a3;
  font-size: 12.5px;
  line-height: 1.7;
}

.danger-btn {
  width: 100%;
  min-height: 40px;

  border: 1px solid #f0cdcd;
  border-radius: 9px;

  background: #fff8f8;
  color: #c04141;

  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;

  transition: background-color 0.2s ease;
}

.danger-btn:hover {
  background: #fdefef;
}

.danger-btn:focus-visible {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}

@media (max-width: 600px) {
  .card {
    padding: 18px;
  }
}
</style>
