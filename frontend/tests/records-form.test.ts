import {describe, expect, it} from "vitest";
import {buildRecordPayload, type RecordFormFields} from "../src/views/records-form.ts";

function fields(overrides: Partial<RecordFormFields> = {}): RecordFormFields {
  return {
    title: "",
    content: "",
    subject: "",
    duration_minutes: "",
    ...overrides,
  };
}

describe("buildRecordPayload 表单校验", () => {
  it("标题为空时报错", () => {
    const result = buildRecordPayload(fields({title: "   "}));

    expect(result).toEqual({ok: false, error: "请输入学习标题"});
  });

  it("标题与可选字段去除首尾空白", () => {
    const result = buildRecordPayload(
      fields({title: " 高数 ", content: " 极限 ", subject: " math "}),
    );

    expect(result).toEqual({
      ok: true,
      payload: {title: "高数", content: "极限", subject: "math"},
    });
  });

  it("可选字段为空时不进入请求体", () => {
    const result = buildRecordPayload(fields({title: "仅标题"}));

    expect(result.ok).toBe(true);

    if (result.ok) {
      expect(result.payload).toEqual({title: "仅标题"});
      expect(result.payload).not.toHaveProperty("duration_minutes");
    }
  });

  it("时长为合法非负整数时写入", () => {
    const result = buildRecordPayload(fields({title: "刷题", duration_minutes: " 30 "}));

    expect(result).toEqual({ok: true, payload: {title: "刷题", duration_minutes: 30}});
  });

  it("时长为 0 时保留", () => {
    const result = buildRecordPayload(fields({title: "打卡", duration_minutes: "0"}));

    expect(result).toEqual({ok: true, payload: {title: "打卡", duration_minutes: 0}});
  });

  it("时长非数字时报错", () => {
    const result = buildRecordPayload(fields({title: "x", duration_minutes: "abc"}));

    expect(result).toEqual({ok: false, error: "学习时长必须是非负整数"});
  });

  it("时长为负数时报错", () => {
    const result = buildRecordPayload(fields({title: "x", duration_minutes: "-5"}));

    expect(result).toEqual({ok: false, error: "学习时长必须是非负整数"});
  });

  it("时长为小数时报错", () => {
    const result = buildRecordPayload(fields({title: "x", duration_minutes: "1.5"}));

    expect(result).toEqual({ok: false, error: "学习时长必须是非负整数"});
  });

  it("时长超出安全整数时报错", () => {
    const result = buildRecordPayload(fields({title: "x", duration_minutes: "9007199254740993"}));

    expect(result).toEqual({ok: false, error: "学习时长必须是非负整数"});
  });
});
