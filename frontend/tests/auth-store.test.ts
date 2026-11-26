import {createPinia, setActivePinia} from "pinia";
import {afterEach, beforeEach, describe, expect, it, vi} from "vitest";
import {TOKEN_KEY, USER_KEY} from "../src/api/http.ts";
import {useAuthStore} from "../src/stores/auth.ts";

const {loginMock, registerMock} = vi.hoisted(() => ({
  loginMock: vi.fn(),
  registerMock: vi.fn(),
}));

vi.mock("../src/api/auth.ts", () => ({
  login: loginMock,
  register: registerMock,
}));

// node 环境无 localStorage，用内存替身
class MemoryStorage {
  private store = new Map<string, string>();

  getItem(key: string): string | null {
    return this.store.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.store.set(key, value);
  }

  removeItem(key: string): void {
    this.store.delete(key);
  }
}

const validUser = {id: "u-1", email: "a@example.com", display_name: "学习者"};

function seedStorage(token: string | null, user: string | null): MemoryStorage {
  const storage = new MemoryStorage();

  if (token !== null) {
    storage.setItem(TOKEN_KEY, token);
  }

  if (user !== null) {
    storage.setItem(USER_KEY, user);
  }

  vi.stubGlobal("localStorage", storage);

  return storage;
}

function freshStore(): ReturnType<typeof useAuthStore> {
  setActivePinia(createPinia());

  return useAuthStore();
}

beforeEach(() => {
  loginMock.mockReset();
  registerMock.mockReset();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("auth store 登录态恢复", () => {
  it("无本地凭据时为未登录", () => {
    seedStorage(null, null);
    const store = freshStore();

    expect(store.token).toBeNull();
    expect(store.user).toBeNull();
    expect(store.isLoggedIn).toBe(false);
  });

  it("恢复有效登录态", () => {
    seedStorage("t-1", JSON.stringify(validUser));
    const store = freshStore();

    expect(store.token).toBe("t-1");
    expect(store.user).toEqual(validUser);
    expect(store.isLoggedIn).toBe(true);
  });

  it("损坏的用户 JSON 会清理并登出", () => {
    const storage = seedStorage("t-1", "{not-json");
    const store = freshStore();

    expect(store.isLoggedIn).toBe(false);
    expect(storage.getItem(TOKEN_KEY)).toBeNull();
    expect(storage.getItem(USER_KEY)).toBeNull();
  });

  it("用户结构非法时清理并登出", () => {
    const storage = seedStorage("t-1", JSON.stringify({id: "u-1", email: "a@example.com"}));
    const store = freshStore();

    expect(store.isLoggedIn).toBe(false);
    expect(storage.getItem(TOKEN_KEY)).toBeNull();
  });

  it("token 与用户信息缺一时统一清理", () => {
    const storage = seedStorage("t-1", null);
    const store = freshStore();

    expect(store.isLoggedIn).toBe(false);
    expect(storage.getItem(TOKEN_KEY)).toBeNull();
    expect(storage.getItem(USER_KEY)).toBeNull();
  });
});

describe("auth store 登录与退出", () => {
  it("登录成功后持久化认证信息", async () => {
    const storage = seedStorage(null, null);
    const store = freshStore();

    loginMock.mockResolvedValue({token: "t-2", user: validUser});

    await store.login({email: validUser.email, password: "passw0rd"});

    expect(store.isLoggedIn).toBe(true);
    expect(store.token).toBe("t-2");
    expect(storage.getItem(TOKEN_KEY)).toBe("t-2");
    expect(JSON.parse(storage.getItem(USER_KEY) ?? "null")).toEqual(validUser);
  });

  it("注册成功后同样持久化认证信息", async () => {
    const storage = seedStorage(null, null);
    const store = freshStore();

    registerMock.mockResolvedValue({token: "t-3", user: validUser});

    await store.register({email: validUser.email, password: "passw0rd", display_name: "学习者"});

    expect(store.isLoggedIn).toBe(true);
    expect(storage.getItem(TOKEN_KEY)).toBe("t-3");
  });

  it("退出登录清理本地状态", () => {
    const storage = seedStorage("t-1", JSON.stringify(validUser));
    const store = freshStore();

    store.logout();

    expect(store.isLoggedIn).toBe(false);
    expect(store.token).toBeNull();
    expect(store.user).toBeNull();
    expect(storage.getItem(TOKEN_KEY)).toBeNull();
    expect(storage.getItem(USER_KEY)).toBeNull();
  });
});
