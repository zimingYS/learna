import type {CreateRecordInput} from "../api/records.ts";

/// 表单原始字段（输入框均为字符串）
export interface RecordFormFields {
  title: string;
  content: string;
  subject: string;
  duration_minutes: string;
}

/// 构建结果：成功给 payload，失败给错误文案
export type BuildPayloadResult =
  | {ok: true; payload: CreateRecordInput}
  | {ok: false; error: string};

/// 由表单字段构建创建请求体（纯函数，视图与测试共用）
export function buildRecordPayload(fields: RecordFormFields): BuildPayloadResult {
  const title = fields.title.trim();

  if (!title) {
    return {ok: false, error: "请输入学习标题"};
  }

  const payload: CreateRecordInput = {title};
  const content = fields.content.trim();
  const subject = fields.subject.trim();
  const duration = fields.duration_minutes.trim();

  if (content) {
    payload.content = content;
  }

  if (subject) {
    payload.subject = subject;
  }

  if (duration !== "") {
    const minutes = Number(duration);

    if (!Number.isSafeInteger(minutes) || minutes < 0) {
      return {ok: false, error: "学习时长必须是非负整数"};
    }

    payload.duration_minutes = minutes;
  }

  return {ok: true, payload};
}
