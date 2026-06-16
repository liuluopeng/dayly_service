import 'dart:io';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
// ignore_for_file: invalid_use_of_internal_member
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:kongde/src/rust/api/db.dart';
import 'package:kongde/services/sqlite_storage.dart';

class MusicService {
  static const MethodChannel _channel = MethodChannel('com.example.kongde/music');

  static List<Map<String, dynamic>> _toMapList(List<LocalSong> songs) =>
    songs.map((s) => {
      'path': s.path, 'title': s.title, 'artist': s.artist,
      'album': s.album, 'duration': s.duration,
      'coverPath': s.coverPath, 'albumId': s.albumId,
      'primaryColor': s.primaryColor, 'secondaryColor': s.secondaryColor,
    }).toList();

  /// 从 SQLite 加载（唯一数据源）
  static Future<List<Map<String, dynamic>>> getMusicFromDevice() async {
    final store = SqliteStorage();
    await store.init();
    final songs = await RustLib.instance.api.crateApiDbGetLocalSongs();
    return _toMapList(songs);
  }

  /// 扫描设备 + 导入 Rust SQLite
  static Future<List<Map<String, dynamic>>> scanDeviceMusic() async {
    List<String> paths;
    if (Platform.isAndroid) {
      paths = await _getPathsFromMediaStore();
    } else {
      paths = await _getPathsFromFolder();
    }
    if (paths.isEmpty) return [];

    final dir = await getApplicationDocumentsDirectory();
    final songs = await RustLib.instance.api.crateApiDbImportLocalSongs(
      paths: paths, coversDir: '${dir.path}/covers',
    );
    return _toMapList(songs);
  }

  static Future<List<String>> _getPathsFromMediaStore() async {
    try {
      final List<dynamic> result = await _channel.invokeMethod('getMusicList');
      return result.map((item) => (item['path'] as String?) ?? '').where((p) => p.isNotEmpty).toList();
    } catch (_) { return []; }
  }

  static Future<List<String>> _getPathsFromFolder() async {
    if (Platform.isIOS || Platform.isMacOS) {
      final result = await FilePicker.platform.pickFiles(type: FileType.audio, allowMultiple: true);
      return result?.files.where((f) => f.path != null).map((f) => f.path!).toList() ?? [];
    }
    final dirPath = await FilePicker.platform.getDirectoryPath();
    if (dirPath == null) return [];
    final dir = Directory(dirPath);
    return dir.listSync().whereType<File>()
      .where((f) => f.path.endsWith('.mp3') || f.path.endsWith('.m4a') || f.path.endsWith('.wav') ||
        f.path.endsWith('.flac') || f.path.endsWith('.aac') || f.path.endsWith('.ogg'))
      .map((f) => f.path).toList();
  }

  static Future<void> clearLocalSongs() async {
    await RustLib.instance.api.crateApiDbClearLocalSongs();
  }
}
