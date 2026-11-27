from __future__ import annotations

from collections.abc import Iterator

import pytest
from app.config.settings import get_settings
from app.main import app


@pytest.fixture(autouse=True)
def _isolate_settings() -> Iterator[None]:
    """settings 是 lru_cache, 依赖覆盖是全局状态, 逐用例重置避免串味"""
    get_settings.cache_clear()
    app.dependency_overrides.clear()
    yield
    get_settings.cache_clear()
    app.dependency_overrides.clear()
