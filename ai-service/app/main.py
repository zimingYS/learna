from __future__ import annotations

import logging

from fastapi import FastAPI

from app.api.internal.auth import install_internal_auth_handler
from app.api.internal.router import router as internal_router

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s :: %(message)s",
)
logger = logging.getLogger(__name__)

app = FastAPI(title="learna-ai-service")
install_internal_auth_handler(app)
app.include_router(
    internal_router,
)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    """内部健康检查"""
    return {"status": "ok"}
