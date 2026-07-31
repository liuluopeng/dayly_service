import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
// ignore_for_file: invalid_use_of_internal_member
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:kongde/utils.dart';

/// SQLite 本地 KV 存储 — 全部逻辑在 Rust，Dart 只做 thin wrapper
class SqliteStorage {
  static final SqliteStorage _instance = SqliteStorage._();
  factory SqliteStorage() => _instance;
  SqliteStorage._();

  bool _ready = false;

  /// 迁移沙盒数据：旧版本启用了 App Sandbox，数据在容器路径
  /// （~/Library/Containers/com.example.kongde/Data/Documents）。
  /// 关闭沙盒后 getApplicationDocumentsDirectory() 指向 ~/Documents，
  /// 需要把 app.db 与 covers 一次性迁移过去，避免数据"丢失"。
  static Future<void> migrateSandboxData() async {
    if (kIsWeb || !Platform.isMacOS) return;
    final newDir = await getApplicationDocumentsDirectory();
    final home = Platform.environment['HOME'];
    if (home == null) return;
    final oldDir =
        '$home/Library/Containers/com.example.kongde/Data/Documents';
    if (oldDir == newDir.path || !Directory(oldDir).existsSync()) return;

    LOGGER.i('检测到旧沙盒数据目录，开始迁移: $oldDir');

    for (final name in ['app.db', 'covers']) {
      final src = '$oldDir/$name';
      final dst = '${newDir.path}/$name';
      final srcFile = File(src);
      final srcDir = Directory(src);

      if (srcFile.existsSync() && !File(dst).existsSync()) {
        try {
          srcFile.rename(dst);
          LOGGER.i('已迁移文件: $name');
        } catch (e) {
          // 跨卷 rename 可能失败，退回拷贝
          try {
            srcFile.copy(dst);
            LOGGER.i('已拷贝文件: $name');
          } catch (e2) {
            LOGGER.w('迁移文件失败: $name: $e2');
          }
        }
      } else if (srcDir.existsSync() && !Directory(dst).existsSync()) {
        try {
          srcDir.rename(dst);
          LOGGER.i('已迁移目录: $name');
        } catch (e) {
          LOGGER.w('迁移目录失败（封面缓存可重新生成）: $name: $e');
        }
      }
    }
  }

  Future<void> init() async {
    if (_ready) return;
    if (kIsWeb) { _ready = true; return; }
    final dir = await getApplicationDocumentsDirectory();
    final path = '${dir.path}/app.db';
    await RustLib.instance.api.crateApiDbInitDb(dbPath: path);
    _ready = true;
  }

  Future<bool> containsKey(String key) async =>
    await RustLib.instance.api.crateApiDbKvGet(key: key) != null;

  Future<String?> getString(String key) async =>
    await RustLib.instance.api.crateApiDbKvGet(key: key);

  Future<bool> setString(String key, String value) async {
    try { await RustLib.instance.api.crateApiDbKvSet(key: key, value: value); return true; }
    catch (e) { LOGGER.w("[kv] setString($key) 失败: $e"); return false; }
  }

  Future<int?> getInt(String key) async =>
    (await RustLib.instance.api.crateApiDbKvGetInt(key: key))?.toInt();

  Future<bool> setInt(String key, int value) async {
    try { await RustLib.instance.api.crateApiDbKvSetInt(key: key, value: value as dynamic); return true; }
    catch (e) { LOGGER.w("[kv] setInt($key) 失败: $e"); return false; }
  }

  Future<double?> getDouble(String key) async =>
    await RustLib.instance.api.crateApiDbKvGetDouble(key: key);

  Future<bool> setDouble(String key, double value) async {
    try { await RustLib.instance.api.crateApiDbKvSetDouble(key: key, value: value); return true; }
    catch (e) { LOGGER.w("[kv] setDouble($key) 失败: $e"); return false; }
  }

  Future<bool> setJson(String key, dynamic value) async =>
    await setString(key, jsonEncode(value));

  Future<T?> getJson<T>(String key, T Function(Map<String, dynamic>) fromJson) async {
    final val = await getString(key);
    if (val == null) return null;
    try { return fromJson(jsonDecode(val) as Map<String, dynamic>); } catch (e) { LOGGER.w("[kv] getJson($key) 解析失败: $e"); return null; }
  }

  Future<List<T>?> getJsonList<T>(String key, T Function(Map<String, dynamic>) fromJson) async {
    final val = await getString(key);
    if (val == null) return null;
    try {
      final list = jsonDecode(val) as List<dynamic>;
      return list.map((e) => fromJson(e as Map<String, dynamic>)).toList();
    } catch (e) { LOGGER.w("[kv] getJsonList($key) 解析失败: $e"); return null; }
  }

  Future<bool> setJsonList(String key, List<dynamic> value) async =>
    await setString(key, jsonEncode(value));

  Future<bool> remove(String key) async =>
    await RustLib.instance.api.crateApiDbKvDelete(key: key);

  Future<void> clear() async {
    await RustLib.instance.api.crateApiDbKvClear();
  }
}
