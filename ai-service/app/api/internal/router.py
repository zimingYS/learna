from __future__ import annotations

from fastapi import APIRouter, Depends

from app.api.internal import documents, learning, plans, reports, roadmaps
from app.api.internal.auth import require_internal_token

router = APIRouter(
    prefix="/internal",
    dependencies=[Depends(require_internal_token)],
)

for sub in (roadmaps, learning, plans, reports, documents):
    router.include_router(sub.router, tags=["internal"])
