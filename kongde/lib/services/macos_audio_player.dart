import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart';
import 'package:kongde/utils.dart';

/// macOS 专用原生播放器：AVAudioEngine 实现播放 + 实时频谱 + 进度
class MacOSAudioPlayer {
  static const _channel = MethodChannel('com.kongde/macos_audio');

  static final _fftController = StreamController<List<double>>.broadcast();

  /// FFT 数据流（实时频谱）
  static Stream<List<double>> get fftStream => _fftController.stream;

  /// 是否正在播放
  static bool _isPlaying = false;
  static bool get isPlaying => _isPlaying;

  static bool get isSupported => !kIsWeb && Platform.isMacOS;

  static bool _listenerSetup = false;

  static void _ensureListener() {
    if (_listenerSetup) return;
    _listenerSetup = true;
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'onFFT' && call.arguments is List) {
        final data = (call.arguments as List).map((e) => (e as num).toDouble()).toList();
        _fftController.add(data);
      }
    });
  }

  /// 播放本地文件
  static Future<bool> play(String filePath) async {
    if (!isSupported) return false;
    try {
      _ensureListener();
      await _channel.invokeMethod('play', {'filePath': filePath});
      _isPlaying = true;
      return true;
    } catch (e) {
      LOGGER.w('[MacOSPlayer] play failed: $e');
      return false;
    }
  }

  /// 暂停
  static Future<void> pause() async {
    if (!isSupported) return;
    await _channel.invokeMethod('pause');
    _isPlaying = false;
  }

  /// 恢复
  static Future<void> resume() async {
    if (!isSupported) return;
    await _channel.invokeMethod('resume');
    _isPlaying = true;
  }

  /// 停止
  static Future<void> stop() async {
    if (!isSupported) return;
    await _channel.invokeMethod('stop');
    _isPlaying = false;
  }

  /// 跳转到指定位置（毫秒）
  static Future<void> seek(int positionMs) async {
    if (!isSupported) return;
    await _channel.invokeMethod('seek', {'positionMs': positionMs});
  }

  static void dispose() {
    _fftController.close();
  }
}
