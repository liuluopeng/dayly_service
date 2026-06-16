# 先构建基础镜像阶段
FROM rust:1.92.0 AS base-builder

# 使用阿里云镜像加速 apt
# RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
#     && rm -rf /var/lib/apt/lists/*

RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
    apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    && curl -fsSL https://deb.nodesource.com/setup_26.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# 安装 pnpm
RUN npm install -g pnpm@11

# 配置 npm 国内镜像
RUN echo "registry=https://registry.npmmirror.com" > /root/.npmrc

# 配置 Rust 国内源加速
RUN echo "[source.crates-io]\n\
    replace-with = 'rsproxy-sparse'\n\
    [source.rsproxy]\n\
    registry = \"https://rsproxy.cn/crates.io-index\"\n\
    [source.rsproxy-sparse]\n\
    registry = \"sparse+https://rsproxy.cn/index/\"\n\
    [registries.rsproxy]\n\
    index = \"https://rsproxy.cn/crates.io-index\"\n\
    [net]\n\
    git-fetch-with-cli = true\n\
    " >> $CARGO_HOME/config.toml

# 安装 WASM 工具链
RUN rustup target add wasm32-unknown-unknown && \
    cargo install wasm-pack

# 使用基础镜像构建应用
FROM base-builder AS builder

# 创建并进入/app目录
WORKDIR /app

# 先复制Cargo.toml和Cargo.lock文件，利用缓存
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock

# 复制各个模块的Cargo.toml文件
COPY ./common/Cargo.toml /app/common/Cargo.toml
COPY ./sifu_axuum/Cargo.toml /app/sifu_axuum/Cargo.toml
COPY ./kongde/rust/Cargo.toml /app/kongde/rust/Cargo.toml
COPY ./my_type/Cargo.toml /app/my_type/Cargo.toml
COPY ./webbvueetauri/src/src-wasm/Cargo.toml /app/webbvueetauri/src/src-wasm/Cargo.toml
COPY ./webbvueetauri/src-tauri/Cargo.toml /app/webbvueetauri/src-tauri/Cargo.toml

# 创建占位文件以确保目录结构正确
RUN mkdir -p /app/common/src && echo 'fn main() {}' > /app/common/src/lib.rs
RUN mkdir -p /app/sifu_axuum/src && echo 'fn main() {}' > /app/sifu_axuum/src/lib.rs
RUN mkdir -p /app/kongde/rust/src && echo 'fn main() {}' > /app/kongde/rust/src/lib.rs
RUN mkdir -p /app/my_type/src && echo 'fn main() {}' > /app/my_type/src/lib.rs
RUN mkdir -p /app/webbvueetauri/src/src-wasm/src && echo 'fn main() {}' > /app/webbvueetauri/src/src-wasm/src/lib.rs
RUN mkdir -p /app/webbvueetauri/src-tauri/src && echo 'fn main() {}' > /app/webbvueetauri/src-tauri/src/lib.rs

# 构建依赖，利用缓存
RUN cargo build --release --bin lx_dayly_service || true

# 复制实际源代码
COPY ./common/src /app/common/src
COPY ./sifu_axuum/src /app/sifu_axuum/src
COPY ./my_type/src /app/my_type/src
COPY ./kongde/rust/src /app/kongde/rust/src

# 复制前端源码
COPY ./webbvueetauri /app/webbvueetauri
COPY ./webbvueetauri/src/src-wasm/src /app/webbvueetauri/src/src-wasm/src

# 复制其他必要文件
COPY ./sifu_axuum/static /app/sifu_axuum/static

# 编译WASM
WORKDIR /app/webbvueetauri/src/src-wasm
RUN wasm-pack build

# 构建前端
WORKDIR /app/webbvueetauri
RUN pnpm install
RUN pnpm build

# 复制前端dist到static
RUN mkdir -p /app/sifu_axuum/static/dist && cp -r dist/* /app/sifu_axuum/static/dist/

# 构建应用
WORKDIR /app/sifu_axuum

COPY ./sifu_axuum/.sqlx /app/sifu_axuum/.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release

# 最终阶段
FROM debian:stable-slim

# 安装运行时依赖（ffmpeg 用于音频处理）
RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
    apt-get update && apt-get install -y \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# 创建 /app 目录
WORKDIR /app
COPY --from=builder /app/target/release/lx_dayly_service /app/lx_dayly_service

# 复制 axum 的后台等静态文件
COPY --from=builder /app/sifu_axuum/static /app/static

# 词典 SQLite 数据库（通过 volume 挂载，不在镜像内）
# COPY dict.db /app/dict.db  ← 8.5GB 太大，走 volume

# 设置环境变量
ENV HOME=/root

WORKDIR /app
EXPOSE 23000
CMD ["/app/lx_dayly_service"]
