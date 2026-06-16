import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
// ignore_for_file: invalid_use_of_internal_member
import 'package:kongde/src/rust/frb_generated.dart';

/// SQLite 本地 KV 存储 — 全部逻辑在 Rust，Dart 只做 thin wrapper
class SqliteStorage {
  static final SqliteStorage _instance = SqliteStorage._();
  factory SqliteStorage() => _instance;
  SqliteStorage._();

  bool _ready = false;

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
    catch (_) { return false; }
  }

  Future<int?> getInt(String key) async =>
    (await RustLib.instance.api.crateApiDbKvGetInt(key: key))?.toInt();

  Future<bool> setInt(String key, int value) async {
    try { await RustLib.instance.api.crateApiDbKvSetInt(key: key, value: value); return true; }
    catch (_) { return false; }
  }

  Future<double?> getDouble(String key) async =>
    await RustLib.instance.api.crateApiDbKvGetDouble(key: key);

  Future<bool> setDouble(String key, double value) async {
    try { await RustLib.instance.api.crateApiDbKvSetDouble(key: key, value: value); return true; }
    catch (_) { return false; }
  }

  Future<bool> setJson(String key, dynamic value) async =>
    await setString(key, jsonEncode(value));

  Future<T?> getJson<T>(String key, T Function(Map<String, dynamic>) fromJson) async {
    final val = await getString(key);
    if (val == null) return null;
    try { return fromJson(jsonDecode(val) as Map<String, dynamic>); } catch (_) { return null; }
  }

  Future<List<T>?> getJsonList<T>(String key, T Function(Map<String, dynamic>) fromJson) async {
    final val = await getString(key);
    if (val == null) return null;
    try {
      final list = jsonDecode(val) as List<dynamic>;
      return list.map((e) => fromJson(e as Map<String, dynamic>)).toList();
    } catch (_) { return null; }
  }

  Future<bool> setJsonList(String key, List<dynamic> value) async =>
    await setString(key, jsonEncode(value));

  Future<bool> remove(String key) async =>
    await RustLib.instance.api.crateApiDbKvDelete(key: key);

  Future<void> clear() async {
    await RustLib.instance.api.crateApiDbKvClear();
  }
}
