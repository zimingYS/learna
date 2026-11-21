# ===== Stage 1: 构建 =====
FROM node:22-slim AS builder
WORKDIR /app

# 依赖
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

# 构建
COPY . .
RUN npm run build

# ===== Stage 2: 发布 =====
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
