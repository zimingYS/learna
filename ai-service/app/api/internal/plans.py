from __future__ import annotations

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from app.schemas.base import ApiResponse

router = APIRouter()


@router.post("/plans/propose-adjustment")
async def propose_plan_adjustment() -> JSONResponse:
    return JSONResponse(
        status_code=501,
        content=ApiResponse.error(
            "NOT_IMPLEMENTED", "POST /internal/plans/propose-adjustment not implemented"
        ).model_dump(),
    )
