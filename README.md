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

flutter MacOS 设置

> Troubleshooting | flutter_rust_bridge
> https://cjycode.com/flutter_rust_bridge/manual/troubleshooting

```
'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/librust_lib.a -lc++',
```

flutter网页版生成步骤:

```
flutter_rust_bridge_codegen build-web --wasm-pack-rustflags "-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals -Clink-arg=--shared-memory -Clink-arg=--import-memory -Clink-arg=--export=__wasm_init_tls -Clink-arg=--export=__tls_size -Clink-arg=--export=__tls_align -Clink-arg=--export=__tls_base"
```

```
flutter run -d chrome --wasm --web-header=Cross-Origin-Opener-Policy=same-origin --web-header=Cross-Origin-Embedder-Policy=require-corp
```
