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

- [2026-08-03] Docker 内构建的 Flutter Web 部署后白屏（`Uncaught Error`），本机构建正常——最终定位是 **apt 的 binaryen(wasm-opt) 108 太旧**，升级 123 解决。完整排查过程见文末《Docker 构建 Flutter Web 白屏排查》。

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

### Docker 构建 Flutter Web 白屏排查（2026-08-03）

**症状**：`docker compose build` 构建的 flutter web 部署到 `/flutter/` 后白屏，
控制台 `main.dart.js` 早期 `Uncaught Error`（flutter_bootstrap → dartProgram 阶段）；
同一份代码用 `fvm flutter run -d chrome --wasm`（本机）一切正常。

**关键方法论**：

1. **对照实验分离变量**：dev（dart2wasm）正常 vs 部署（dart2js）崩——先验证是否
   dart2js 构建问题：`flutter run -d chrome --release`（release + dart2js + dev server）
   ——若也崩则与部署环境无关。
2. **产物交叉对照定位组件**：把本机构建的 `main.dart.js` / `pkg/rust_lib_kongde.js`
   分别 `docker cp` 进容器替换 docker 产物，二分定位是哪一侧产物崩。
   实测：docker 的 main.dart.js + 本机 pkg = 正常 → **元凶在 wasm(pkg) 构建**。
3. **md5 对比 + diff 排除"看似不同"**：docker 与本机 main.dart.js md5 不同，
   但 diff 后只是 dart2js 混淆符号名不同（`c1V` vs `c1t` 是同一抛异常函数），
   语义相同 → 排除 main.dart.js。
4. **逐一排除工具链版本漂移**（每步重建后隐私窗口实测）：

   | 嫌疑 | 结论 |
   |---|---|
   | Flutter 3.44.0（docker stable）vs 本机 3.41.0 | 固定 `ghcr.nju.edu.cn/cirruslabs/flutter:3.41.0`（与 fvm 同 commit 44a626f4f0） |
   | rustup 自动拉最新 nightly（08-02）vs 本机（07-15）| 固定 `nightly-2026-07-15`（FRB 强制 `RUSTUP_TOOLCHAIN=nightly`，用工具链目录重命名使其命中固定版本）——**非根因** |
   | `-Clinker=wasm-ld`（apt lld 18）| 去掉后 wasm md5 完全不变 → 排除 |
   | **apt binaryen(wasm-opt) 108（2023）vs 本机 wasm-pack 自动下载 117** | **根因**：旧版 wasm-opt 优化多线程(shared-memory/atomics) wasm 有 bug → 升级 `binaryen version_123`（下载 release tar 覆盖 /usr/local）✓ 修复 |

**同批发现并修复的问题**（都与 flutter web 部署相关）：

- **GetX 5.0.0-rc → 4.7.3**：dart2js 下匿名路由名 = `/${runtimeType}`（混淆后
  `/minified:Eg`）+ 强制 PathUrlStrategy → 点按钮"平滑回退"、二次点击无反应。
  稳定版 4.7.3 用普通 Navigator（匿名路由不进 URL），规避。配套改
  `SnackPosition` 枚举大小写、flutter_audio_visualizer 补 `flutter_web_plugins` 依赖。
- **web 下 KV 存储是空 stub**（`db_wasm.rs` 永远返回 None）→ 配置/token 刷新即丢、
  表现为"循环登录页" → 改 localStorage 持久化（`web-sys`，仅 wasm32 目标启用）。
- **同源模式 base_url 写死 `localhost:23001`**（`init_wasm.rs`）→ docker 部署时
  所有请求发错端口 → 改为跟随 `window.location.origin()`。
- **wasm 内存上限 32MB 太小** → `--max-memory=134217728`（128MB），
  sed patch 泛化 `initial:256,maximum:2048`。
- **service worker 缓存旧产物**：排查期间反复"白屏/异常"的干扰源——测试必须用
  隐私窗口或 Clear site data。

**当前构建参数**（Dockerfile / build-frontends.sh 已同步）：

```
--max-memory=134217728
sed: 's/wasm.__wbindgen_start(thread_stack_size);/wasm.__wbindgen_start(thread_stack_size || 1048576);/' \
     's/initial:[0-9]*,maximum:[0-9]*/initial:256,maximum:2048/'
```

