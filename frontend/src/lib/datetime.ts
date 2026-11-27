// 时间输入与展示的公共工具：datetime-local 只到分钟，后端返回值带毫秒/时区，
// 两侧互相转换时必须按分钟截断，否则「原样打开再保存」会被判成改过时间。

// datetime-local 需要本地时间且不含时区后缀
export function toLocalInput(iso: string | null): string {
  if (!iso) {
    return "";
  }

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  const pad = (value: number) => String(value).padStart(2, "0");

  return [
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
    `${pad(date.getHours())}:${pad(date.getMinutes())}`,
  ].join("T");
}

// 后端要求 RFC3339；空串表示未填
export function toIso(input: string): string | null {
  const trimmed = input.trim();

  if (!trimmed) {
    return null;
  }

  const date = new Date(trimmed);

  if (Number.isNaN(date.getTime())) {
    return null;
  }

  return date.toISOString();
}

// 按分钟比较两个时刻；都为空视为相同
export function sameInstant(a: string | null, b: string | null): boolean {
  const toMinute = (iso: string | null): number | null => {
    if (!iso) {
      return null;
    }

    const time = new Date(iso).getTime();

    return Number.isNaN(time) ? null : Math.floor(time / 60_000);
  };

  return toMinute(a) === toMinute(b);
}

export function formatDateTime(iso: string | null): string {
  if (!iso) {
    return "—";
  }

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "—";
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

export function formatDate(iso: string | null): string {
  if (!iso) {
    return "—";
  }

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) {
    return "—";
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
}

/// 剩余天数：按本地自然日计算，已逾期为负
export function daysUntil(iso: string | null): number | null {
  if (!iso) {
    return null;
  }

  const target = new Date(iso);

  if (Number.isNaN(target.getTime())) {
    return null;
  }

  const startOfTarget = new Date(target.getFullYear(), target.getMonth(), target.getDate());
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  return Math.round((startOfTarget.getTime() - startOfToday.getTime()) / 86_400_000);
}
