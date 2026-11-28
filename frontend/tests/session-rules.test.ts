import {describe, expect, it} from "vitest";
import {
  activityTone,
  formatElapsed,
  formatMinutes,
  formatTimeRange,
  intervalMinutes,
} from "../src/views/sessions/session-rules.ts";

/** 用本地时间构造，避免断言随运行机器的时区变化 */
function localIso(hour: number, minute: number, day = 28): string {
  return new Date(2026, 10, day, hour, minute).toISOString();
}

describe("区间与时长", () => {
  it("区间分钟数向下取整", () => {
    expect(intervalMinutes(localIso(9, 0), localIso(10, 0))).toBe(60);
    expect(intervalMinutes(localIso(9, 0), localIso(9, 59))).toBe(59);
  });

  it("零长度区间为 0，倒序区间为 null", () => {
    expect(intervalMinutes(localIso(9, 0), localIso(9, 0))).toBe(0);
    expect(intervalMinutes(localIso(10, 0), localIso(9, 0))).toBeNull();
  });

  it("非法时间返回 null，不抛异常", () => {
    expect(intervalMinutes("", localIso(9, 0))).toBeNull();
  });

  it("分钟数在跨小时时折算成小时 + 分钟", () => {
    expect(formatMinutes(0)).toBe("0 分钟");
    expect(formatMinutes(45)).toBe("45 分钟");
    expect(formatMinutes(60)).toBe("1 小时");
    expect(formatMinutes(155)).toBe("2 小时 35 分钟");
  });
});

describe("时间区间展示", () => {
  it("同一天只在左侧写日期", () => {
    expect(formatTimeRange(localIso(9, 0), localIso(11, 30))).toBe("11-28 09:00 → 11:30");
  });

  it("跨午夜两侧都写日期，避免看起来像倒流", () => {
    expect(formatTimeRange(localIso(23, 30, 28), localIso(0, 30, 29))).toBe(
      "11-28 23:30 → 11-29 00:30",
    );
  });

  it("非法时间退化为占位符", () => {
    expect(formatTimeRange("", localIso(9, 0))).toBe("—");
  });
});

describe("计时器展示", () => {
  it("按 HH:MM:SS 补零", () => {
    expect(formatElapsed(0)).toBe("00:00:00");
    expect(formatElapsed(61_000)).toBe("00:01:01");
    expect(formatElapsed(3_661_000)).toBe("01:01:01");
  });

  it("负数按 0 处理（时钟回拨时不要显示负号）", () => {
    expect(formatElapsed(-5_000)).toBe("00:00:00");
  });
});

describe("活动类型配色", () => {
  it("复习与练习取不同 tone，未知语义落中性", () => {
    expect(activityTone("theory")).toBe("primary");
    expect(activityTone("practice")).toBe("success");
    expect(activityTone("review")).toBe("warning");
    expect(activityTone("other")).toBe("neutral");
  });
});
