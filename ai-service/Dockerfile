FROM python:3.13-slim
WORKDIR /app

# 安装UV
COPY --from=ghcr.io/astral-sh/uv:0.5.11 /uv /uvx /usr/local/bin/

# 配置环境变量
ENV UV_COMPILE_BYTECODE=1 \
        UV_LINK_MODE=copy \
        PYTHONUNBUFFERED=1

# 添加依赖
COPY pyproject.toml uv.lock ./
RUN --mount=type=cache,target=/root/.cache/uv \
        uv sync --locked --no-dev --no-install-project

# 源码
COPY app ./app
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --locked --no-dev --no-install-project

EXPOSE 8001

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=5 \
    CMD python -c "import urllib.request,sys; \
        sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:8001/healthz').status==200 else 1)"

CMD ["uv", "run", "--no-sync", "uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8001"]
