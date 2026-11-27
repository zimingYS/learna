<template>
  <section aria-busy="true" class="project-grid">
    <div v-for="index in 6" :key="index" class="skeleton-card">
      <div class="skeleton-line skeleton-line--title"></div>
      <div class="skeleton-line"></div>
      <div class="skeleton-line skeleton-line--short"></div>
    </div>
  </section>
</template>

<style scoped>
.project-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
}

.skeleton-card {
  display: flex;
  flex-direction: column;
  gap: 12px;

  height: 186px;
  padding: 20px;

  border: 1px solid #eef1f7;
  border-radius: 14px;

  background: #fff;
}

.skeleton-line {
  height: 12px;

  border-radius: 6px;

  background: linear-gradient(90deg, #f1f3f9 25%, #f7f9fd 50%, #f1f3f9 75%);
  background-size: 200% 100%;

  animation: skeleton 1.4s ease-in-out infinite;
}

.skeleton-line--title {
  width: 55%;
  height: 16px;
}

.skeleton-line--short {
  width: 70%;
}

@keyframes skeleton {
  from {
    background-position: 200% 0;
  }

  to {
    background-position: -200% 0;
  }
}
</style>
