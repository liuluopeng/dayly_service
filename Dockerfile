# Flutter Web 构建镜像（国内代理，可 --build-arg 覆盖）
ARG FLUTTER_IMAGE=ghcr.nju.edu.cn/cirruslabs/flutter:stable

# 先构建基础镜像阶段
FROM rust:1.92.0 AS base-builder

# 使用阿里云镜像加速 apt
# RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
#     && rm -rf /var/lib/apt/lists/*

RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources && \
    apt-get update &&     apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
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
# 镜像预装 nightly 但缺 rust-src，补装
RUN rustup component add rust-src --toolchain nightly-aarch64-unknown-linux-gnu \
    || rustup component add rust-src --toolchain nightly \
    || true

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

WORKDIR /app/kongde
RUN flutter config --no-analytics \
    && flutter pub get

# FRB build-web（atomics/shared-memory flags，与宿主机 build-frontends.sh 一致）
RUN flutter_rust_bridge_codegen build-web --release \
    --wasm-pack-rustflags \
    "-Clinker=wasm-ld -Ctarget-feature=+atomics,+bulk-memory,+mutable-globals \
     -Clink-arg=--shared-memory \
     -Clink-arg=--import-memory \
     -Clink-arg=--max-memory=33554432 \
     -Clink-arg=--export=__wasm_init_tls \
     -Clink-arg=--export=__tls_size \
     -Clink-arg=--export=__tls_align \
     -Clink-arg=--export=__tls_base \
     -Clink-arg=--export=__heap_base"

# Patch thread_stack_size 默认值 + 增大初始化内存（Linux sed 语法）
RUN sed -i \
  -e 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
  -e 's/initial:[0-9]*,maximum:512/initial:256,maximum:512/' \
  web/pkg/rust_lib_kongde.js

# Flutter Web（JS 模式，与宿主机一致）
RUN flutter build web --release --base-href=/flutter/

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

# 复制其他必要文件
COPY ./sifu_axuum/static /app/sifu_axuum/static

# 编译 WASM
WORKDIR /app/webbvueetauri/src/src-wasm
RUN wasm-pack build

# 构建前端
WORKDIR /app/webbvueetauri
RUN pnpm install
RUN cd /app/webbvueetauri && sed -i '/"prebuild"/d' package.json && pnpm build

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

# 覆盖为容器内构建的 Flutter Web 产物
COPY --from=flutter-web-builder /app/sifu_axuum/static/flutter /app/static/flutter

# 词典 SQLite 数据库（通过 volume 挂载，不在镜像内）
# COPY dict.db /app/dict.db  ← 8.5GB 太大，走 volume

# 设置环境变量
ENV HOME=/root

WORKDIR /app
EXPOSE 23000
CMD ["/app/lx_dayly_service"]
