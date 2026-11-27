from __future__ import annotations

import asyncio
import re

import httpx
import pytest
from app.api.internal.auth import (
    UNCONFIGURED_CODE,
    UNCONFIGURED_MESSAGE,
    InternalAuthError,
    assert_internal_auth_configured,
    require_internal_token,
)
from app.config.settings import Settings, get_settings
from app.main import app
from fastapi.testclient import TestClient

TOKEN = "test-internal-token"
HEADER = "X-Internal-Token"
ENVELOPE_KEYS = {"code", "message", "data", "request_id", "schema_version"}
REQUEST_ID_PATTERN = re.compile(r"^[0-9a-f]{32}$")
PLACEHOLDER_ENDPOINTS = [
    "/internal/roadmaps/generate",
    "/internal/plans/propose-adjustment",
    "/internal/learning/analyze",
    "/internal/reports/generate",
    "/internal/documents/parse",
]


def _assert_envelope(body: dict[str, object]) -> None:
    assert set(body) == ENVELOPE_KEYS
    assert body["schema_version"] == "v1"
    assert isinstance(body["request_id"], str)
    assert REQUEST_ID_PATTERN.match(body["request_id"])


def _post_without_lifespan(path: str, headers: dict[str, str]) -> httpx.Response:
    """走 ASGITransport 绕过 lifespan, 单独验证请求期闸门"""

    async def _call() -> httpx.Response:
        transport = httpx.ASGITransport(app=app)
        async with httpx.AsyncClient(
            transport=transport, base_url="http://test"
        ) as client:
            return await client.post(path, headers=headers)

    return asyncio.run(_call())


def test_healthz_is_public(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("AI_SERVICE_TOKEN", TOKEN)
    get_settings.cache_clear()

    with TestClient(app) as client:
        resp = client.get("/healthz")

    assert resp.status_code == 200
    assert resp.json() == {"status": "ok"}


def test_missing_token_rejected(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("AI_SERVICE_TOKEN", TOKEN)
    get_settings.cache_clear()

    with TestClient(app) as client:
        resp = client.post("/internal/roadmaps/generate")

    body = resp.json()
    assert resp.status_code == 401
    assert body["code"] == "40101"
    assert body["data"] is None
    _assert_envelope(body)


def test_wrong_token_rejected(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("AI_SERVICE_TOKEN", TOKEN)
    get_settings.cache_clear()

    with TestClient(app) as client:
        resp = client.post("/internal/roadmaps/generate", headers={HEADER: "nope"})

    assert resp.status_code == 401
    assert resp.json()["code"] == "40101"


def test_non_ascii_token_header_rejected_not_crashed() -> None:
    # compare_digest 对非 ASCII str 抛 TypeError, 必须先编码, 否则非法 header 变 500
    # httpx 拒发非 ASCII header, 故直接调依赖函数模拟 uvicorn 的 latin-1 解码结果
    with pytest.raises(InternalAuthError) as excinfo:
        asyncio.run(
            require_internal_token(
                "tökén-with-umlaut", Settings(ai_service_token=TOKEN)
            )
        )

    assert excinfo.value.status_code == 401
    assert excinfo.value.detail["code"] == "40101"


@pytest.mark.parametrize("path", PLACEHOLDER_ENDPOINTS)
def test_valid_token_reaches_placeholder(
    monkeypatch: pytest.MonkeyPatch, path: str
) -> None:
    monkeypatch.setenv("AI_SERVICE_TOKEN", TOKEN)
    get_settings.cache_clear()

    with TestClient(app) as client:
        resp = client.post(path, headers={HEADER: TOKEN})

    body = resp.json()
    assert resp.status_code == 501
    assert body["code"] == "NOT_IMPLEMENTED"
    _assert_envelope(body)


def test_unconfigured_token_refuses_even_with_header() -> None:
    # 显式传入 Settings 命中, 压过宿主环境变量, 保证「未配置」语义确定
    app.dependency_overrides[get_settings] = lambda: Settings(ai_service_token=None)

    resp = _post_without_lifespan("/internal/roadmaps/generate", {HEADER: TOKEN})

    body = resp.json()
    assert resp.status_code == 503
    assert body["code"] == UNCONFIGURED_CODE
    assert body["message"] == UNCONFIGURED_MESSAGE
    assert body["data"] is None
    _assert_envelope(body)


@pytest.mark.parametrize("blank", ["", "   ", "\n"])
def test_blank_token_counts_as_unconfigured(blank: str) -> None:
    app.dependency_overrides[get_settings] = lambda: Settings(ai_service_token=blank)

    resp = _post_without_lifespan("/internal/roadmaps/generate", {HEADER: blank})

    assert resp.status_code == 503
    assert resp.json()["code"] == UNCONFIGURED_CODE


def test_token_with_surrounding_whitespace_is_normalized() -> None:
    # 密钥文件/echo 写入常带尾随换行, 配置侧归一化后仍应匹配无空白 header
    app.dependency_overrides[get_settings] = lambda: Settings(
        ai_service_token=f"  {TOKEN}\n"
    )

    resp = _post_without_lifespan("/internal/roadmaps/generate", {HEADER: TOKEN})

    assert resp.status_code == 501


def test_startup_fails_without_token(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        "app.main.get_settings", lambda: Settings(ai_service_token=None)
    )

    with pytest.raises(RuntimeError, match="AI_SERVICE_TOKEN"), TestClient(app):
        pass


def test_startup_passes_with_token(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        "app.main.get_settings", lambda: Settings(ai_service_token=TOKEN)
    )

    with TestClient(app) as client:
        assert client.get("/healthz").status_code == 200


def test_startup_guard_rejects_blank_token() -> None:
    with pytest.raises(RuntimeError, match="AI_SERVICE_TOKEN"):
        assert_internal_auth_configured(Settings(ai_service_token="   "))
