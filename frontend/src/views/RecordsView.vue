<template>
  <main class="records-page">
    <div class="records-container">
      <!-- 页眉 -->
      <header class="page-header">
        <div class="header-content">
          <div class="header-title">
            <h1>学习记录</h1>
            <p>页面描述</p>
          </div>
          <button class="btn btn-primary create-btn" @click="openCreate">
            <span class="plus-icon">+</span>
            新建记录
          </button>
        </div>
      </header>

      <!-- 表单 -->
      <Transition name="form">
        <section v-if="showForm" aria-labelledby="form-title" class="form-card">
          <div class="section-header">
            <div>
              <h2 id="form-title">{{ isEditing ? "编辑学习记录" : "新建学习记录" }}</h2>
              <p class="section-description">{{
                  isEditing ? "修改本次学习记录的信息。" : "填写本次学习的基本信息，保存你的学习成果。"
                }}</p>
            </div>
            <span :class="{ 'edit-badge': isEditing }" class="mode-badge">
              {{ isEditing ? "编辑模式" : "新建模式" }}
            </span>
          </div>

          <form class="record-form" @submit.prevent="handleSubmit">
            <!-- 学习标题 -->
            <div class="form-group">
              <label for="record-title">
                学习标题
                <span class="required">*</span>
              </label>

              <input id="record-title" v-model="form.title.value" :disabled="loading" maxlength="200"
                     placeholder="例如：学习 Rust Axum 中间件" required type="text"/>
            </div>
            <!-- 科目和时长 -->
            <div class="form-row">

              <div class="form-group">
                <label for="record-subject">
                  学习科目
                  <span class="optional">选填</span>
                </label>

                <input id="record-subject" v-model="form.subject.value" :disabled="loading" maxlength="100"
                       placeholder="例如：Rust、数据结构" type="text"/>
              </div>

              <div class="form-group">
                <label for="record-duration">
                  学习时长
                  <span class="optional">分钟 · 选填</span>
                </label>

                <div class="input-with-suffix">
                  <input id="record-duration" v-model="form.duration_minutes.value" :disabled="loading" min="0"
                         placeholder="例如：60"
                         step="1" type="number"/>
                  <span class="input-suffix">分钟</span>
                </div>
              </div>

            </div>

            <!-- 学习内容 -->
            <div class="form-group">
              <label for="record-content">
                学习内容
                <span class="optional">选填</span>
              </label>

              <textarea id="record-content" v-model="form.content.value" :disabled="loading" maxlength="5000"
                        placeholder="记录今天学习的知识点、完成的任务和遇到的问题……" rows="5"></textarea>

              <div class="textarea-footer">
                <span>简要记录学习过程，方便日后回顾。</span>
                <span>{{ form.content.value.length }}/5000</span>
              </div>
            </div>

            <!-- 表单操作 -->
            <div class="form-actions">
              <button v-if="isEditing" :disabled="loading" class="btn btn-secondary" type="button" @click="cancelEdit">
                取消
              </button>

              <button v-else :disabled="loading" class="btn btn-secondary" type="button" @click="cancelEdit">
                关闭
              </button>

              <button :disabled="loading" class="btn btn-primary" type="submit">
                <span v-if="loading" class="loading-spinner"></span>

                {{ loading ? "提交中..." : isEditing ? "保存修改" : "创建记录" }}
              </button>
            </div>
          </form>
        </section>
      </Transition>

      <!-- 错误提示 -->
      <Transition name="fade">
        <div v-if="errorMessage" class="error-msg" role="alert" aria-live="assertive">
          <span class="error-icon">!</span>
          <span>{{ errorMessage }}</span>
          <button type="button" class="error-close" aria-label="关闭错误提示" @click="errorMessage = ''">×</button>
        </div>
      </Transition>

      <!-- 列表区域 -->
      <section class="list-card" aria-labelledby="list-title">
        <!-- 列表头部 -->
        <div class="list-header">
          <div>
            <h2 id="list-title">全部记录</h2>
            <p class="list-description">查看和管理你的学习历程</p>
          </div>

          <div class="record-count">
            <span class="count-number">{{ total }}</span>
            <span class="count-label">条记录</span>
          </div>
        </div>

        <!-- 数据表格 -->
        <div class="table-wrapper">
          <table class="records-table">
            <thead>
            <tr>
              <th class="col-title">学习标题</th>
              <th class="col-subject">科目</th>
              <th class="col-duration">学习时长</th>
              <th class="col-date">更新时间</th>
              <th class="col-actions">操作</th>
            </tr>
            </thead>

            <tbody>
            <!-- 加载状态 -->
            <tr v-if="listLoading">
              <td colspan="5" class="state-cell">
                <div class="table-loading">
                  <span class="loading-spinner"></span>
                  <span>正在加载学习记录...</span>
                </div>
              </td>
            </tr>

            <!-- 空状态 -->
            <tr v-else-if="items.length === 0">
              <td colspan="5" class="state-cell">
                <div class="empty-state">
                  <div class="empty-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
                         stroke-linejoin="round" aria-hidden="true">
                      <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
                      <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z"/>
                      <path d="M8 7h8M8 11h6"/>
                    </svg>
                  </div>

                  <h3>还没有学习记录</h3>
                  <p>开始记录你的第一次学习，积累自己的知识成长轨迹。</p>
                  <button type="button" class="btn btn-primary empty-create-btn" @click="openCreate">
                    <span class="plus-icon">+</span>
                    创建第一条记录
                  </button>
                </div>
              </td>
            </tr>

            <!-- 正常数据 -->
            <tr v-for="item in items" v-else :key="item.id" class="record-row"
                :class="{ 'active-row': editingId === item.id }">
              <!-- 标题和内容 -->
              <td class="col-title">
                <div class="record-info">
                    <span class="record-title">
                      {{ item.title }}
                    </span>

                  <span v-if="item.content" class="record-content" :title="item.content">
                      {{ item.content }}
                    </span>
                </div>
              </td>

              <!-- 科目 -->
              <td class="col-subject">
                  <span v-if="item.subject" class="subject-tag">
                    {{ item.subject }}
                  </span>

                <span v-else class="empty-value">—</span>
              </td>

              <!-- 时长 -->
              <td class="col-duration">
                  <span class="duration-text">
                    {{ item.duration_minutes !== null ? `${item.duration_minutes} 分钟` : "—" }}
                  </span>
              </td>

              <!-- 更新时间 -->
              <td class="col-date">
                  <span class="date-text">
                    {{ formatDate(item.updated_at) }}
                  </span>
              </td>

              <!-- 操作 -->
              <td class="col-actions">
                <div class="row-actions">
                  <button type="button" class="action-btn edit-btn" :disabled="deletingId !== null"
                          @click="startEdit(item)">
                    编辑
                  </button>

                  <button type="button" class="action-btn delete-btn" :disabled="deletingId !== null"
                          @click="handleDelete(item.id)">
                    {{ deletingId === item.id ? "删除中..." : "删除" }}
                  </button>
                </div>
              </td>
            </tr>

            </tbody>
          </table>
        </div>

        <!-- 分页区域 -->
        <footer class="pagination-footer">

          <div class="pagination-info">
            <span>共</span>
            <strong>{{ total }}</strong>
            <span>条记录</span>
            <span class="pagination-divider">·</span>
            <span>每页 {{ PAGE_SIZE }} 条</span>
          </div>

          <nav class="pagination" aria-label="记录分页">

            <button type="button" class="page-btn page-arrow" :disabled="page <= 1 || listLoading"
                    @click="previousPage">
              <span aria-hidden="true">‹</span>
              <span class="arrow-text">上一页</span>
            </button>

            <span class="page-indicator">
              第<strong>{{ page }}</strong>/{{ totalPages }}页
            </span>

            <button type="button" class="page-btn page-arrow" :disabled="page >= totalPages || listLoading"
                    @click="nextPage">
              <span class="arrow-text">下一页</span>
              <span aria-hidden="true">›</span>
            </button>

          </nav>
        </footer>
      </section>
      <!-- 页面底部 -->
      <footer class="page-footer">
        <span>用心记录每一次进步。</span>
      </footer>
    </div>
  </main>
</template>

<script lang="ts" setup>
import {computed, onMounted, ref} from "vue";
import {
  createRecord,
  type CreateRecordInput,
  deleteRecord,
  fetchRecords,
  type RecordItem,
  type RecordListData,
  updateRecord
} from "../api/records.ts";
import {ApiError} from "../api/http.ts";
import {buildRecordPayload} from "./records-form.ts";

const PAGE_SIZE = 10;

// 学习记录列表
const items = ref<RecordItem[]>([]);
// 总记录数
const total = ref(0);
// 当前页码
const page = ref(1);
// 当前编辑记录 ID
const editingId = ref<string | null>(null);
// 是否显示表单
const showForm = ref(false);
// 错误信息
const errorMessage = ref("");
// 加载状态
const loading = ref(false);
// 列表加载状态
const listLoading = ref(false);
// 当前正在删除的记录 ID
const deletingId = ref<string | null>(null);

// 表单数据
const form = {
  title: ref(""),
  content: ref(""),
  subject: ref(""),
  duration_minutes: ref(""),
};

// 页码总计
const totalPages = computed(() =>
  Math.max(1, Math.ceil(total.value / PAGE_SIZE))
);

// 是否处于编辑模式
const isEditing = computed(() => editingId.value !== null);

// 获取错误信息
function getErrorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    return error.message;
  }

  return "请求失败，请稍后重试";
}

// 打开新建表单
function openCreate(): void {
  resetForm();

  editingId.value = null;
  errorMessage.value = "";
  showForm.value = true;
}

// 重置表单
function resetForm(): void {
  form.title.value = "";
  form.content.value = "";
  form.subject.value = "";
  form.duration_minutes.value = "";
}

// 格式化时间
function formatDate(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "--";
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

// 构建请求参数（校验规则在 records-form 纯函数中）
function buildPayload(): CreateRecordInput | null {
  const result = buildRecordPayload({
    title: form.title.value,
    content: form.content.value,
    subject: form.subject.value,
    duration_minutes: form.duration_minutes.value,
  });

  if (!result.ok) {
    errorMessage.value = result.error;
    return null;
  }

  return result.payload;
}

// 加载学习记录
async function loadRecords(): Promise<void> {
  listLoading.value = true;
  errorMessage.value = "";

  try {
    const result: RecordListData = await fetchRecords(
      page.value,
      PAGE_SIZE,
    );

    items.value = result.items;
    total.value = result.total;
  } catch (error: unknown) {
    errorMessage.value = getErrorMessage(error);
  } finally {
    listLoading.value = false;
  }
}

// 提交
async function handleSubmit(): Promise<void> {
  if (loading.value) {
    return;
  }
  errorMessage.value = "";
  const payload = buildPayload();

  if (!payload) {
    return;
  }
  loading.value = true;

  try {
    if (editingId.value === null) {
      // 新建记录
      await createRecord(payload);
      // 新记录按创建时间倒序排列，回到第一页查看
      page.value = 1;
    } else {
      // 更新记录
      await updateRecord(editingId.value, payload);
    }

    // 操作成功，重置表单
    resetForm();
    editingId.value = null;
    showForm.value = false;

    // 刷新列表
    await loadRecords();
  } catch (error: unknown) {
    errorMessage.value = getErrorMessage(error);
  } finally {
    loading.value = false;
  }
}

// 编辑
function startEdit(item: RecordItem): void {
  form.title.value = item.title;
  form.content.value = item.content ?? "";
  form.subject.value = item.subject ?? "";
  form.duration_minutes.value = item.duration_minutes?.toString() ?? "";

  editingId.value = item.id;
  showForm.value = true;
  errorMessage.value = "";

  window.scrollTo({
    top: 0,
    behavior: "smooth",
  });
}

// 取消编辑
function cancelEdit(): void {
  resetForm();

  editingId.value = null;
  showForm.value = false;
  errorMessage.value = "";
}

// 删除
async function handleDelete(id: string): Promise<void> {
  if (deletingId.value !== null) {
    return;
  }

  const confirmed = window.confirm(
    "确定删除这条学习记录吗？删除后无法恢复。",
  );

  if (!confirmed) {
    return;
  }

  errorMessage.value = "";
  deletingId.value = id;

  try {
    await deleteRecord(id);

    // 当前页只有一条记录且不是第一页，回退页码
    if (items.value.length === 1 && page.value > 1) {
      page.value--;
    }

    // 刷新列表
    await loadRecords();

    // 如果删除的是当前正在编辑的记录，退出编辑模式
    if (editingId.value === id) {
      cancelEdit();
    }
  } catch (error: unknown) {
    errorMessage.value = getErrorMessage(error);
  } finally {
    deletingId.value = null;
  }
}

// 上一页
async function previousPage(): Promise<void> {
  if (page.value <= 1 || listLoading.value) {
    return;
  }

  page.value--;
  await loadRecords();
}

// 下一页
async function nextPage(): Promise<void> {
  if (page.value >= totalPages.value || listLoading.value) {
    return;
  }

  page.value++;
  await loadRecords();
}

onMounted(() => {
  void loadRecords();
});
</script>

<style scoped>
/* ==================== 页面基础 ==================== */

.records-page {
  min-height: 100%;
  padding: 40px 28px;
  background: #f5f7fb;
  color: #263248;
}

.records-container {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

* {
  box-sizing: border-box;
}

/* ==================== 页面头部 ==================== */

.page-header {
  margin-bottom: 28px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}

.header-title h1 {
  margin: 0;
  color: #202b40;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.header-title p {
  margin: 9px 0 0;
  color: #8490a3;
  font-size: 14px;
  line-height: 1.6;
}

/* ==================== 通用按钮 ==================== */

.btn {
  display: inline-flex;
  min-height: 40px;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 18px;
  border: 1px solid transparent;
  border-radius: 8px;
  font-family: inherit;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s,
  border-color 0.2s,
  color 0.2s,
  transform 0.2s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #4f46e5;
  color: #fff;
  box-shadow: 0 2px 5px rgb(79 70 229 / 15%);
}

.btn-primary:hover:not(:disabled) {
  background: #4338ca;
  transform: translateY(-1px);
}

.btn-secondary {
  border-color: #dce2eb;
  background: #fff;
  color: #596579;
}

.btn-secondary:hover:not(:disabled) {
  border-color: #c5ccda;
  background: #f8f9fc;
}

.plus-icon {
  font-size: 21px;
  font-weight: 400;
  line-height: 1;
}

.create-btn {
  min-height: 42px;
}

/* ==================== 表单卡片 ==================== */

.form-card {
  margin-bottom: 24px;
  padding: 26px;
  border: 1px solid #e9edf4;
  border-radius: 12px;
  background: #fff;
  box-shadow: 0 3px 12px rgb(31 45 72 / 4%);
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 24px;
}

.section-header h2 {
  margin: 0;
  color: #263248;
  font-size: 19px;
  font-weight: 650;
}

.section-description {
  margin: 8px 0 0;
  color: #8994a7;
  font-size: 13px;
}

.mode-badge {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 5px;
  background: #eef0ff;
  color: #5148ca;
  font-size: 12px;
  font-weight: 500;
}

.edit-badge {
  background: #fff5e8;
  color: #be761e;
}

.record-form {
  display: flex;
  flex-direction: column;
  gap: 19px;
}

.form-group {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 9px;
}

.form-group label {
  color: #364152;
  font-size: 13px;
  font-weight: 600;
}

.required {
  margin-left: 3px;
  color: #e05252;
}

.optional {
  margin-left: 5px;
  color: #a0a9b7;
  font-size: 12px;
  font-weight: 400;
}

.form-group input,
.form-group textarea {
  width: 100%;
  border: 1px solid #dce2eb;
  border-radius: 7px;
  outline: none;
  background: #fff;
  color: #263248;
  font-family: inherit;
  font-size: 14px;
  transition: border-color 0.2s,
  box-shadow 0.2s;
}

.form-group input {
  height: 43px;
  padding: 0 13px;
}

.form-group textarea {
  min-height: 110px;
  padding: 12px 13px;
  resize: vertical;
  line-height: 1.7;
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: #adb5c2;
}

.form-group input:focus,
.form-group textarea:focus {
  border-color: #7771ec;
  box-shadow: 0 0 0 3px rgb(79 70 229 / 9%);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background: #f8f9fc;
  cursor: not-allowed;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
}

.input-with-suffix {
  position: relative;
}

.input-with-suffix input {
  padding-right: 52px;
}

.input-suffix {
  position: absolute;
  top: 50%;
  right: 13px;
  color: #9aa4b3;
  font-size: 12px;
  transform: translateY(-50%);
  pointer-events: none;
}

.textarea-footer {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  color: #9ba5b4;
  font-size: 11px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 5px;
}

/* ==================== 错误提示 ==================== */

.error-msg {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
  padding: 12px 15px;
  border: 1px solid #f4caca;
  border-radius: 8px;
  background: #fff3f3;
  color: #cf4545;
  font-size: 13px;
}

.error-icon {
  display: inline-flex;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: #ed6868;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.error-close {
  margin-left: auto;
  padding: 0 4px;
  border: none;
  background: transparent;
  color: #c66b6b;
  font-size: 20px;
  cursor: pointer;
}

/* ==================== 列表卡片 ==================== */

.list-card {
  overflow: hidden;
  border: 1px solid #e9edf4;
  border-radius: 12px;
  background: #fff;
  box-shadow: 0 3px 12px rgb(31 45 72 / 4%);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 23px 25px;
}

.list-header h2 {
  margin: 0;
  color: #263248;
  font-size: 18px;
  font-weight: 650;
}

.list-description {
  margin: 7px 0 0;
  color: #939dae;
  font-size: 12px;
}

.record-count {
  display: flex;
  align-items: baseline;
  gap: 5px;
}

.count-number {
  color: #4f46e5;
  font-size: 23px;
  font-weight: 700;
}

.count-label {
  color: #8b96a8;
  font-size: 12px;
}

/* ==================== 表格 ==================== */

.table-wrapper {
  width: 100%;
  overflow-x: auto;
}

.records-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  text-align: left;
}

.records-table thead {
  background: #f8f9fc;
}

.records-table th {
  height: 46px;
  padding: 0 15px;
  border-top: 1px solid #edf0f5;
  border-bottom: 1px solid #edf0f5;
  color: #7c8799;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.records-table td {
  padding: 16px 15px;
  border-bottom: 1px solid #eef1f5;
  vertical-align: middle;
}

.records-table tbody tr:last-child td {
  border-bottom: none;
}

.records-table .col-title {
  width: 34%;
  padding-left: 25px;
}

.records-table .col-subject {
  width: 16%;
}

.records-table .col-duration {
  width: 13%;
}

.records-table .col-date {
  width: 21%;
}

.records-table .col-actions {
  width: 16%;
  padding-right: 20px;
}

.record-row {
  transition: background 0.2s;
}

.record-row:hover {
  background: #fafbff;
}

.record-row.active-row {
  background: #f5f5ff;
}

.record-info {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 6px;
}

.record-title {
  overflow: hidden;
  color: #29354b;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.record-content {
  display: -webkit-box;
  overflow: hidden;
  color: #909bad;
  font-size: 12px;
  line-height: 1.5;
  text-overflow: ellipsis;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1;
  overflow-wrap: anywhere;
}

.subject-tag {
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  padding: 5px 9px;
  border-radius: 5px;
  background: #f0efff;
  color: #635bd0;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-value {
  color: #c0c6d0;
}

.duration-text {
  color: #58657a;
  font-size: 13px;
  white-space: nowrap;
}

.date-text {
  color: #8792a4;
  font-size: 12px;
  white-space: nowrap;
}

.row-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-btn {
  padding: 5px 0;
  border: none;
  background: transparent;
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: color 0.2s;
}

.action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.edit-btn {
  color: #5148d1;
}

.edit-btn:hover:not(:disabled) {
  color: #3329b3;
}

.delete-btn {
  color: #d65a5a;
}

.delete-btn:hover:not(:disabled) {
  color: #b82f2f;
}

/* ==================== 空状态 ==================== */

.state-cell {
  height: 300px;
  padding: 35px !important;
  text-align: center;
}

.empty-state {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 12px;
}

.empty-icon {
  display: flex;
  width: 62px;
  height: 62px;
  align-items: center;
  justify-content: center;
  border-radius: 19px;
  background: #eef0ff;
  color: #625be1;
}

.empty-icon svg {
  width: 30px;
  height: 30px;
}

.empty-state h3 {
  margin: 4px 0 0;
  color: #37445a;
  font-size: 15px;
  font-weight: 650;
}

.empty-state p {
  max-width: 340px;
  margin: 0;
  color: #96a0af;
  font-size: 12px;
  line-height: 1.7;
}

.empty-create-btn {
  margin-top: 7px;
}

/* ==================== 加载状态 ==================== */

.table-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #8994a5;
  font-size: 13px;
}

.loading-spinner {
  display: inline-block;
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  border: 2px solid #e0e3f7;
  border-top-color: #5148d1;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ==================== 分页 ==================== */

.pagination-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 15px;
  padding: 17px 24px;
  border-top: 1px solid #edf0f5;
}

.pagination-info {
  display: flex;
  align-items: center;
  gap: 5px;
  color: #929dad;
  font-size: 12px;
}

.pagination-info strong {
  color: #59657a;
  font-weight: 600;
}

.pagination-divider {
  margin: 0 3px;
  color: #d2d7e0;
}

.pagination {
  display: flex;
  align-items: center;
  gap: 13px;
}

.page-btn {
  display: inline-flex;
  min-height: 32px;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 0 9px;
  border: 1px solid #e1e5ed;
  border-radius: 6px;
  background: #fff;
  color: #536078;
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.page-btn:hover:not(:disabled) {
  border-color: #7771ec;
  color: #5148d1;
}

.page-btn:disabled {
  background: #fafbfc;
  color: #bdc3ce;
  cursor: not-allowed;
}

.page-arrow span[aria-hidden="true"] {
  font-size: 19px;
  line-height: 1;
}

.page-indicator {
  color: #8994a5;
  font-size: 12px;
  white-space: nowrap;
}

.page-indicator strong {
  color: #5148d1;
  font-weight: 650;
}

/* ==================== 页面底部 ==================== */

.page-footer {
  padding: 24px 0 5px;
  color: #a5adba;
  font-size: 11px;
  text-align: center;
}

/* ==================== 过渡动画 ==================== */
/*
.form-enter-active,
.form-leave-active {
  transition: opacity 0.2s ease,
  transform 0.2s ease;
}

.form-enter-from,
.form-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
*/
/* ==================== 响应式布局 ==================== */

@media (max-width: 768px) {
  .records-page {
    padding: 25px 16px;
  }

  .header-title h1 {
    font-size: 24px;
  }

  .form-card {
    padding: 20px;
  }

  .form-row {
    grid-template-columns: 1fr;
  }

  .records-table {
    min-width: 760px;
  }

  .list-header {
    padding: 19px;
  }

  .pagination-footer {
    align-items: flex-start;
    flex-direction: column;
    padding: 16px 19px;
  }

  .pagination {
    width: 100%;
    justify-content: space-between;
  }
}

@media (max-width: 480px) {
  .records-page {
    padding: 20px 12px;
  }

  .header-content {
    align-items: flex-start;
    flex-direction: column;
  }

  .create-btn {
    width: 100%;
  }

  .form-card {
    padding: 17px;
  }

  .section-header h2 {
    font-size: 17px;
  }

  .section-description {
    line-height: 1.6;
  }

  .textarea-footer {
    font-size: 10px;
  }

  .form-actions .btn {
    flex: 1;
  }

  .page-arrow {
    padding: 0 10px;
  }

  .arrow-text {
    font-size: 11px;
  }
}
</style>
