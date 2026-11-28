<script setup lang="ts">
import {onMounted, ref} from "vue";
import type {ContentDetailItem} from "../api/contents";
import ContentDetailPanel from "../components/contents/ContentDetailPanel.vue";
import ContentFormDialog from "../components/contents/ContentFormDialog.vue";
import ContentRelationPanel from "../components/contents/ContentRelationPanel.vue";
import ContentTagPanel from "../components/contents/ContentTagPanel.vue";
import ContentTreePanel from "../components/contents/ContentTreePanel.vue";
import ProjectLinkPanel from "../components/contents/ProjectLinkPanel.vue";
import {useContentsStore} from "../stores/contents";
import {formatSimilarity} from "./contents/contents-rules";

const store = useContentsStore();

const formOpen = ref(false);
const editing = ref<ContentDetailItem | null>(null);

function openCreate(): void {
  editing.value = null;
  formOpen.value = true;
}

function openEdit(): void {
  editing.value = store.selected;
  formOpen.value = true;
}

async function openItem(id: string): Promise<void> {
  await store.select(id);
}

onMounted(async () => {
  await Promise.all([store.load(), store.loadVocabulary()]);
});
</script>

<template>
  <main class="contents-page">
    <div class="contents-container">
      <header class="page-header">
        <div>
          <h1>学习内容</h1>
          <p>
            知识点、技能点与语言项的统一内容库。同一份内容可以被多个项目复用，
            每个项目各自维护进度，互不影响。
          </p>
        </div>
      </header>

      <p v-if="store.error" class="error-banner" role="alert">{{ store.error }}</p>

      <section v-if="store.similarHints.length > 0" class="hint-banner">
        <div class="hint-body">
          <p class="hint-title">
            创建成功。库里还有 {{ store.similarHints.length }} 份标题相近的内容，确认一下是否重复：
          </p>

          <ul class="hint-list">
            <li v-for="hint in store.similarHints" :key="hint.id">
              <button class="hint-link" type="button" @click="openItem(hint.id)">
                {{ hint.title }}
              </button>
              <span class="hint-score">相似度 {{ formatSimilarity(hint.similarity) }}</span>
            </li>
          </ul>
        </div>

        <button class="hint-close" type="button" @click="store.clearHints()">知道了</button>
      </section>

      <div class="contents-body">
        <article class="column-card">
          <ContentTreePanel @create="openCreate" @open="openItem"/>
        </article>

        <div class="detail-column">
          <ContentDetailPanel @edit="openEdit"/>

          <template v-if="store.selected">
            <ContentTagPanel/>
            <ContentRelationPanel/>
            <ProjectLinkPanel/>
          </template>
        </div>
      </div>
    </div>

    <ContentFormDialog
      :content="editing"
      :open="formOpen"
      @close="formOpen = false"
    />
  </main>
</template>

<style scoped>
.contents-page {
  min-height: 100%;
  color: var(--ink-1);
}

.contents-container {
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  color: var(--ink-1);
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.page-header p {
  margin: 9px 0 0;
  color: var(--ink-3);
  font-size: 14px;
  line-height: 1.6;
}

.error-banner {
  padding: 11px 14px;
  margin: 0 0 16px;

  border: 1px solid #f7dada;
  border-radius: 10px;

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 13px;
  line-height: 1.6;
}

.hint-banner {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;

  padding: 14px 16px;
  margin-bottom: 16px;

  border: 1px solid #f6e3c4;
  border-radius: var(--radius-md);

  background: #fffaf1;
}

.hint-body {
  min-width: 0;
}

.hint-title {
  margin: 0;
  color: #8a5a12;
  font-size: 12.8px;
  line-height: 1.6;
}

.hint-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;

  padding: 0;
  margin: 8px 0 0;

  list-style: none;
}

.hint-list li {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hint-link {
  padding: 0;

  border: none;
  background: transparent;
  color: var(--ink-1);

  font-family: inherit;
  font-size: 12.5px;
  font-weight: 600;
  text-align: left;
  text-decoration: underline;
  cursor: pointer;
}

.hint-score {
  color: var(--ink-4);
  font-size: 11.5px;
}

.hint-close {
  flex-shrink: 0;

  padding: 5px 12px;

  border: 1px solid #f0dcbb;
  border-radius: var(--radius-pill);

  background: #fff;
  color: #8a5a12;

  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.contents-body {
  display: grid;
  gap: 16px;
  align-items: start;
  grid-template-columns: minmax(0, 400px) minmax(0, 1fr);
}

.column-card {
  padding: 20px 22px;

  border: 1px solid var(--line);
  border-radius: var(--radius-lg);

  background: #fff;
}

.detail-column {
  display: flex;
  flex-direction: column;
  gap: 16px;

  min-width: 0;
}

@media (max-width: 1080px) {
  .contents-body {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .column-card {
    padding: 18px 16px;
  }
}
</style>
