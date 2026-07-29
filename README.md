# dayly_service

服务端:
axum + postgresql + docker

flutter客户端;flutter网页版:
flutter + sqlite + flutter_rust_bridge

网页版:
vue + wasm + tauri

### 数据库准备

MDX => SQL
从电子词典分享网站下载MDX格式的词典, 用脚本生成DB文件.

五笔数据 SVG格式:

词频数据:

### 维护

备份命令:

### 遇到的问题

- 在MacOS中实现频谱 困难,没实现.

- 一次MACOS编译失败:
  flutter MacOS 设置

> Troubleshooting | flutter_rust_bridge
> https://cjycode.com/flutter_rust_bridge/manual/troubleshooting

```
'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/librust_lib.a -lc++',
```

- 第一次存在sqlite时, flutter网页版因为sqlx,无法生成网页版.

- flutter网页版生成步骤:

```
flutter_rust_bridge_codegen build-web \
  --wasm-pack-rustflags \
  "-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals \
   -Clink-arg=--shared-memory \
   -Clink-arg=--import-memory \
   -Clink-arg=--export=__wasm_init_tls \
   -Clink-arg=--export=__tls_size \
   -Clink-arg=--export=__tls_align \
   -Clink-arg=--export=__tls_base \
   -Clink-arg=--export=__heap_base"
```

2026年7月27日22:31:26

# 1. Build wasm

flutter_rust_bridge_codegen build-web --release \
--wasm-pack-rustflags \
"-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals \
-Clink-arg=--shared-memory \
-Clink-arg=--import-memory \
-Clink-arg=--max-memory=33554432 \
-Clink-arg=--export=__wasm_init_tls \
-Clink-arg=--export=__tls_size \
-Clink-arg=--export=__tls_align \
-Clink-arg=--export=__tls_base \
-Clink-arg=--export=__heap_base"

# 2. Patch thread_stack_size 默认值 + 增大初始化内存

sed -i '' \
-e 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
-e 's/initial:25,maximum:512/initial:256,maximum:512/' \
web/pkg/rust_lib_kongde.js

```
fvm flutter run -d chrome --wasm --web-header=Cross-Origin-Opener-Policy=same-origin --web-header=Cross-Origin-Embedder-Policy=require-corp
```

### 开发

热更新:
`
cargo watch -x 'run -p lx_dayly_service'

`

flutter web:
`

`
