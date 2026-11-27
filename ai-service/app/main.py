from __future__ import annotations

import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.api.internal.auth import (
    assert_internal_auth_configured,
    install_internal_auth_handler,
)
from app.api.internal.router import router as internal_router
from app.config.settings import get_settings

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s :: %(message)s",
)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(_app: FastAPI) -> AsyncIterator[None]:
    # 启动期校验 + 请求期拒绝, 两层都 fail-closed, 缺一层就可能带病上线
    assert_internal_auth_configured(get_settings())
    yield


app = FastAPI(title="learna-ai-service", lifespan=lifespan)
install_internal_auth_handler(app)
app.include_router(
    internal_router,
)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    """内部健康检查(免鉴权, 供存活探针使用)"""
    return {"status": "ok"}
