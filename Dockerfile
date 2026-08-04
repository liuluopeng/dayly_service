# Flutter Web 构建镜像（国内代理，可 --build-arg 覆盖）
# 注意：固定 3.41.0 —— 3.44.0 构建的 flutter web 在 dart2js 下启动即崩（Uncaught Error）
ARG FLUTTER_IMAGE=ghcr.nju.edu.cn/cirruslabs/flutter:3.41.0

# 先构建基础镜像阶段
FROM rust:1.92.0 AS base-builder

# 使用阿里云镜像加速 apt
# RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
#     && rm -rf /var/lib/apt/lists/*

# arm64 基础镜像用 ports.ubuntu.com，一并换阿里云镜像
RUN sed -i 's|http://ports.ubuntu.com/ubuntu-ports|http://mirrors.aliyun.com/ubuntu-ports|g' /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null || \
    sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources 2>/dev/null || true
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler curl ca-certificates binaryen \
    && rm -rf /var/lib/apt/lists/*

# Node.js：官方源不稳定，用 npmmirror 二进制（arm64/x64 通用）
RUN ARCH=$(uname -m) && case "$ARCH" in \
        aarch64|arm64) NODE_ARCH=arm64 ;; \
        x86_64|amd64)  NODE_ARCH=x64 ;; \
        *) echo "unsupported arch: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://npmmirror.com/mirrors/node/v22.16.0/node-v22.16.0-linux-${NODE_ARCH}.tar.xz" | tar -xJ -C /usr/local --strip-components=1 && \
    node --version && npm --version

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

# ═══ Flutter Web 构建阶段（容器内构建，产物复制进 static/flutter）═══
FROM ${FLUTTER_IMAGE} AS flutter-web-builder

ENV PUB_HOSTED_URL=https://pub.flutter-io.cn \
    FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn \
    RUSTUP_DIST_SERVER=https://rsproxy.cn \
    RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup \
    PATH="/root/.cargo/bin:$PATH"

# Rust 工具链（FRB codegen / wasm-pack 需要）
# apt 源换阿里云镜像（国内网络访问 ports.ubuntu.com 不稳定）
RUN sed -i 's|http://ports.ubuntu.com/ubuntu-ports|http://mirrors.aliyun.com/ubuntu-ports|g' /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null || true \
    && apt-get update && apt-get install -y curl pkg-config libssl-dev protobuf-compiler git lld binaryen \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rustup target add wasm32-unknown-unknown

# binaryen（wasm-opt）升级到 123：apt 版本 108 过旧，优化多线程 wasm（atomics）
# 有 bug，导致 docker 内构建的 flutter web 启动即崩（宿主机 wasm-pack 用新版正常）
RUN ARCH=$(uname -m) && case "$ARCH" in \
        aarch64|arm64) BE_ARCH=aarch64 ;; \
        x86_64|amd64)  BE_ARCH=x86_64 ;; \
        *) echo "unsupported arch: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL --retry 3 "https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-${BE_ARCH}-linux.tar.gz" \
        | tar -xz -C /usr/local --strip-components=1 \
    && wasm-opt --version

# cargo 国内源
RUN echo "[source.crates-io]\n\
    replace-with = 'rsproxy-sparse'\n\
    [source.rsproxy]\n\
    registry = \"https://rsproxy.cn/crates.io-index\"\n\
    [source.rsproxy-sparse]\n\
    registry = \"sparse+https://rsproxy.cn/index/\"\n\
    [registries.rsproxy]\n\
    index = \"https://rsproxy.cn/crates.io-index\"\n\
    [net]\n\
    git-fetch-with-cli = true\n" >> $CARGO_HOME/config.toml

# FRB 工具链（编译耗时，独立缓存层）
# 预装与项目匹配的 wasm-bindgen-cli（0.2.126），避免 wasm-pack 在
# RUSTFLAGS 泄漏时临时安装导致失败
RUN cargo install wasm-pack \
    && cargo install flutter_rust_bridge_codegen --version 2.12.0 \
    && cargo install wasm-bindgen-cli --version 0.2.126

# FRB build-web 用 -Z build-std（需要 nightly + rust-src 组件）
# 固定 nightly-2026-07-15（与宿主机同代）：不同 nightly 的 build-std 生成的 wasm
# 行为不同——docker 内最新 nightly(2026-08-02) 构建的 flutter web 启动即崩（Uncaught Error）
# FRB 强制设置 RUSTUP_TOOLCHAIN=nightly：把固定 nightly 的工具链目录重命名为 nightly，
# 使该解析命中固定版本（rustup toolchain link 不允许标准 channel 名）
RUN rustup toolchain install nightly-2026-07-15 --profile minimal --component rust-src \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2026-07-15 \
    && rustup toolchain uninstall nightly 2>/dev/null || true \
    && mv /root/.rustup/toolchains/nightly-2026-07-15-aarch64-unknown-linux-gnu \
          /root/.rustup/toolchains/nightly-aarch64-unknown-linux-gnu

WORKDIR /app
# kongde/rust 继承根 workspace（workspace.dependencies），根 Cargo.toml 必须复制
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock
# kongde/rust 的 path 依赖（common / my_type）必须一起复制
COPY ./common /app/common
COPY ./my_type /app/my_type
COPY ./kongde /app/kongde
# cargo 解析 workspace 需要所有 member 的 Cargo.toml 存在（仅 manifest，不编译）
COPY ./sifu_axuum/Cargo.toml /app/sifu_axuum/Cargo.toml
COPY ./local-agent/Cargo.toml /app/local-agent/Cargo.toml
COPY ./wasm-demo/Cargo.toml /app/wasm-demo/Cargo.toml
COPY ./webbvueetauri/src-tauri/Cargo.toml /app/webbvueetauri/src-tauri/Cargo.toml
COPY ./webbvueetauri/src/src-wasm/Cargo.toml /app/webbvueetauri/src/src-wasm/Cargo.toml
# 占位 src：cargo 解析 workspace 时要求每个 member 有 targets（不参与编译）
RUN mkdir -p /app/sifu_axuum/src /app/local-agent/src /app/wasm-demo/src \
        /app/webbvueetauri/src-tauri/src /app/webbvueetauri/src/src-wasm/src \
    && echo 'fn main() {}' > /app/sifu_axuum/src/main.rs \
    && echo 'fn main() {}' > /app/local-agent/src/main.rs \
    && echo 'fn main() {}' > /app/wasm-demo/src/lib.rs \
    && echo 'fn main() {}' > /app/webbvueetauri/src-tauri/src/lib.rs \
    && echo 'fn main() {}' > /app/webbvueetauri/src/src-wasm/src/lib.rs

# 共享构建脚本（唯一事实来源：本地 build-frontends.sh / CI / Dockerfile 一致）
COPY ./scripts /app/scripts

WORKDIR /app/kongde
RUN flutter config --no-analytics \
    && flutter pub get

# FRB build-web + patch + flutter build web（统一走共享脚本）
RUN bash /app/scripts/build-flutter-web.sh

# 收集产物
RUN mkdir -p /app/sifu_axuum/static/flutter && rm -rf /app/sifu_axuum/static/flutter/* \
    && cp -r build/web/* /app/sifu_axuum/static/flutter/

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
COPY ./sifu_axuum/proto /app/sifu_axuum/proto
COPY ./sifu_axuum/build.rs /app/sifu_axuum/build.rs
COPY ./kongde/rust/Cargo.toml /app/kongde/rust/Cargo.toml
COPY ./local-agent/Cargo.toml /app/local-agent/Cargo.toml
COPY ./my_type/Cargo.toml /app/my_type/Cargo.toml
COPY ./webbvueetauri/src/src-wasm/Cargo.toml /app/webbvueetauri/src/src-wasm/Cargo.toml
# 占位 src：cargo 解析 workspace 时要求每个 member 有 targets（不参与编译）
RUN mkdir -p /app/sifu_axuum/src /app/local-agent/src /app/wasm-demo/src \
        /app/webbvueetauri/src-tauri/src /app/webbvueetauri/src/src-wasm/src \
    && echo 'fn main() {}' > /app/sifu_axuum/src/main.rs \
    && echo 'fn main() {}' > /app/local-agent/src/main.rs \
    && echo 'fn main() {}' > /app/wasm-demo/src/lib.rs \
    && echo 'fn main() {}' > /app/webbvueetauri/src-tauri/src/lib.rs \
    && echo 'fn main() {}' > /app/webbvueetauri/src/src-wasm/src/lib.rs
COPY ./wasm-demo/Cargo.toml /app/wasm-demo/Cargo.toml
COPY ./webbvueetauri/src-tauri/Cargo.toml /app/webbvueetauri/src-tauri/Cargo.toml

# 创建占位文件以确保目录结构正确
RUN mkdir -p /app/common/src && echo 'fn main() {}' > /app/common/src/lib.rs
RUN mkdir -p /app/sifu_axuum/src && echo 'fn main() {}' > /app/sifu_axuum/src/lib.rs
RUN mkdir -p /app/kongde/rust/src && echo 'fn main() {}' > /app/kongde/rust/src/lib.rs
RUN mkdir -p /app/local-agent/src && echo 'fn main() {}' > /app/local-agent/src/lib.rs
RUN mkdir -p /app/my_type/src && echo 'fn main() {}' > /app/my_type/src/lib.rs
RUN mkdir -p /app/webbvueetauri/src/src-wasm/src && echo 'fn main() {}' > /app/webbvueetauri/src/src-wasm/src/lib.rs
RUN mkdir -p /app/webbvueetauri/src-tauri/src && echo 'fn main() {}' > /app/webbvueetauri/src-tauri/src/lib.rs
RUN mkdir -p /app/wasm-demo/src && echo 'fn main() {}' > /app/wasm-demo/src/lib.rs

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

# 共享构建脚本
COPY ./scripts /app/scripts

# 复制静态资源（排除 Docker 内构建的 dist/flutter 目录，
# 宿主机旧产物不进入镜像，由容器内构建步骤产出）
COPY ./sifu_axuum/static /app/sifu_axuum/static
RUN rm -rf /app/sifu_axuum/static/dist /app/sifu_axuum/static/flutter /app/sifu_axuum/static/vue

# 编译 WASM + 构建前端（统一走共享脚本：wasm-pack + pnpm install + prebuild typecheck + vite build）
WORKDIR /app
RUN bash /app/scripts/build-vue.sh

# 复制前端 dist 到 static（/dist/ 与 /vue/ 都用容器内新构建产物）
RUN mkdir -p /app/sifu_axuum/static/dist /app/sifu_axuum/static/vue \
    && cp -r /app/webbvueetauri/dist/* /app/sifu_axuum/static/dist/ \
    && cp -r /app/webbvueetauri/dist/* /app/sifu_axuum/static/vue/

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

# 覆盖为容器内构建的 Flutter Web 产物
COPY --from=flutter-web-builder /app/sifu_axuum/static/flutter /app/static/flutter

# 词典 SQLite 数据库（通过 volume 挂载，不在镜像内）
# COPY dict.db /app/dict.db  ← 8.5GB 太大，走 volume

# 设置环境变量
ENV HOME=/root

WORKDIR /app
EXPOSE 23000
CMD ["/app/lx_dayly_service"]
