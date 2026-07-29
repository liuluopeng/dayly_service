import 'dart:io';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/play_local_music_page.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:kongde/utils.dart';

const _audioExts = {'mp3', 'flac', 'wav', 'm4a', 'ogg', 'aac', 'wma'};

class OpenFileService {
  static const MethodChannel _channel =
      MethodChannel('com.example.kongde/open_file');

  static void init() {
    LOGGER.i("[OpenFile] init: registering handler");
    _channel.setMethodCallHandler(_handleMethodCall);
    LOGGER.i("[OpenFile] handler registered");
  }

  static Future<dynamic> _handleMethodCall(MethodCall call) async {
    LOGGER.i("[OpenFile] method=${call.method} args=${call.arguments}");
    if (call.method == 'openFile') {
      final path = call.arguments as String;
      _handleOpenFile(path);
    }
  }

  static void _handleOpenFile(String path) async {
    LOGGER.i("[OpenFile] handling path=$path");

    final ext = path.split('.').last.toLowerCase();
    if (!_audioExts.contains(ext)) {
      LOGGER.w("[OpenFile] unsupported file type: $ext");
      return;
    }

    try {
      final file = File(path);
      if (!await file.exists()) {
        LOGGER.e("[OpenFile] file not found: $path");
        return;
      }

      final playlist = <Map<String, dynamic>>[];
      int startIndex = 0;

      final dir = file.parent;
      try {
        if (await dir.exists()) {
          final entities = await dir.list().toList();
          entities.sort((a, b) => a.path.compareTo(b.path));

          for (int i = 0; i < entities.length; i++) {
            final entity = entities[i];
            if (entity is! File) continue;
            final fext = entity.path.split('.').last.toLowerCase();
            if (!_audioExts.contains(fext)) continue;
            playlist.add({
              'url': entity.path,
              'title': entity.path.split('/').last,
              'artist': '',
              'album': '',
              'coverPath': '',
            });
            if (entity.path == path) startIndex = playlist.length - 1;
          }
        }
      } catch (e) {
        LOGGER.w("[OpenFile] dir listing failed (sandbox?), fallback to single file: $e");
        playlist.clear();
        playlist.add({
          'url': path,
          'title': path.split('/').last,
          'artist': '',
          'album': '',
          'coverPath': '',
        });
        startIndex = 0;
      }

      if (playlist.isEmpty) {
        LOGGER.e("[OpenFile] no audio files found");
        return;
      }

      LOGGER.i("[OpenFile] playlist=${playlist.length} start=$startIndex");
      final audioHandler = Get.find<AudioPlayerHandler>();
      await audioHandler.playFromPlaylist(playlist, startIndex);
      LOGGER.i("[OpenFile] navigating to player");
      Get.to(() => const PlayLocalMusicPage());
      LOGGER.i("[OpenFile] done");
    } catch (e, stack) {
      LOGGER.e("[OpenFile] error: $e\n$stack");
    }
  }
}
