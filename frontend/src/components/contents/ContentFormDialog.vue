<script setup lang="ts">
import {computed, ref, watch} from "vue";
import type {ContentDetailItem, ContentItem} from "../../api/contents";
import ModalDialog from "../ui/ModalDialog.vue";
import ContentPicker from "./ContentPicker.vue";
import {useContentsStore} from "../../stores/contents";
import {
  BODY_FORMAT_OPTIONS,
  CONTENT_TYPE_OPTIONS,
} from "../../views/contents/contents-rules";
import {
  BODY_MAX,
  buildContentCreatePayload,
  buildContentUpdatePayload,
  type ContentFormFields,
  contentToForm,
  DESCRIPTION_MAX,
  emptyContentForm,
  SOURCE_URL_MAX,
  TITLE_MAX,
} from "../../views/contents/contents-forms";

const props = defineProps<{
  open: boolean;
  content: ContentDetailItem | null;
}>();

const emit = defineEmits<{ close: []; saved: [] }>();

const store = useContentsStore();

const fields = ref<ContentFormFields>(emptyContentForm());
const parentLabel = ref("根层级");
const formError = ref("");
const submitting = ref(false);

const isEdit = computed(() => props.content !== null);
const errorText = computed(() => formError.value || store.error);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    if (props.content) {
      fields.value = contentToForm(props.content);
      parentLabel.value = props.content.parent_id
        ? store.titles[props.content.parent_id] ?? "上级内容"
        : "根层级";
    } else {
      // 新建默认落在当前浏览的层级下，符合「在哪一层建就在哪一层」
      fields.value = emptyContentForm(store.parentId ?? "");
      parentLabel.value = store.parentId
        ? store.titles[store.parentId] ?? "上级内容"
        : "根层级";
    }

    formError.value = "";
    store.clearError();
  },
);

function handlePickParent(item: ContentItem): void {
  fields.value.parentId = item.id;
  parentLabel.value = item.title;
}

function clearParent(): void {
  fields.value.parentId = "";
  parentLabel.value = "根层级";
}

async function handleSubmit(): Promise<void> {
  if (submitting.value) {
    return;
  }

  formError.value = "";
  submitting.value = true;

  try {
    if (props.content) {
      const built = buildContentUpdatePayload(props.content, fields.value);

      if (!built.ok) {
        formError.value = built.error;
        return;
      }

      // 未变化的字段不提交；全都没变就直接收起，避免一次无意义的 PATCH
      if (Object.keys(built.payload).length === 0) {
        emit("close");
        return;
      }

      if (await store.save(props.content.id, built.payload)) {
        emit("saved");
        emit("close");
      }

      return;
    }

    const built = buildContentCreatePayload(fields.value);

    if (!built.ok) {
      formError.value = built.error;
      return;
    }

    if (await store.create(built.payload)) {
      emit("saved");
      emit("close");
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <ModalDialog
    :description="
      isEdit
        ? '只提交真正改动的字段；正文按原样保存，不做渲染。'
        : '内容可以是知识点、技能点或语言项，同一标题允许重复创建。'
    "
    :open="open"
    :title="isEdit ? '编辑内容' : '新建内容'"
    @close="emit('close')"
  >
    <form class="pf-form" @submit.prevent="handleSubmit">
      <div class="pf-group">
        <label class="pf-label" for="content-title">
          标题
          <span class="pf-required">*</span>
        </label>

        <input
          id="content-title"
          v-model="fields.title"
          class="pf-input"
          :maxlength="TITLE_MAX"
          placeholder="例如：函数极限的 ε-δ 定义"
          required
          type="text"
        />
      </div>

      <div class="pf-row">
        <div class="pf-group">
          <label class="pf-label" for="content-type">内容类型</label>

          <select id="content-type" v-model="fields.contentType" class="pf-input">
            <option v-for="option in CONTENT_TYPE_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="pf-group">
          <label class="pf-label" for="content-body-format">正文格式</label>

          <select id="content-body-format" v-model="fields.bodyFormat" class="pf-input">
            <option v-for="option in BODY_FORMAT_OPTIONS" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="content-description">
          摘要
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="content-description"
          v-model="fields.description"
          class="pf-input"
          :maxlength="DESCRIPTION_MAX"
          placeholder="一句话说明这份内容解决什么问题"
        ></textarea>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="content-body">
          正文
          <span class="pf-optional">选填</span>
        </label>

        <textarea
          id="content-body"
          v-model="fields.body"
          class="pf-input body-input"
          :maxlength="BODY_MAX"
          placeholder="Markdown 或纯文本原文，保存后原样展示"
        ></textarea>

        <p class="pf-hint">
          正文只保存原始文本：后端不渲染、不净化，前端也按源码展示，不注入 HTML。
        </p>
      </div>

      <div class="pf-group">
        <label class="pf-label" for="content-source-url">
          出处链接
          <span class="pf-optional">选填</span>
        </label>

        <input
          id="content-source-url"
          v-model="fields.sourceUrl"
          class="pf-input"
          :maxlength="SOURCE_URL_MAX"
          placeholder="https://…"
          type="url"
        />
      </div>

      <div class="pf-group">
        <label class="pf-label">
          上级内容
          <span class="pf-optional">当前：{{ parentLabel }}</span>
        </label>

        <ContentPicker
          :exclude-ids="content ? [content.id] : []"
          placeholder="搜索上级内容…"
          @pick="handlePickParent"
          @clear="clearParent"
        />

        <p class="pf-hint">
          留空即根层级。上级内容会校验同归属、不成环、深度不超过 10 层。
        </p>
      </div>

      <p v-if="errorText" class="pf-error">{{ errorText }}</p>

      <div class="pf-actions">
        <button :disabled="submitting" class="pf-btn pf-btn--ghost" type="button" @click="emit('close')">
          取消
        </button>

        <button :disabled="submitting" class="pf-btn pf-btn--primary" type="submit">
          <span v-if="submitting" class="pf-spinner"></span>
          {{ submitting ? "保存中..." : isEdit ? "保存修改" : "创建内容" }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>

<style scoped>
.body-input {
  min-height: 160px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
}
</style>
