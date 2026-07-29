import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart';
import 'package:kongde/utils.dart';

/// macOS 专用原生播放器：AVAudioEngine 实现播放 + 实时频谱 + 进度
class MacOSAudioPlayer {
  static const _channel = MethodChannel('com.kongde/macos_audio');
  static const _pcmChannel = EventChannel('com.kongde/macos_audio/pcm');
  static const _positionChannel = EventChannel('com.kongde/macos_audio/position');

  static Stream<List<double>>? _pcmStream;
  static StreamSubscription? _pcmSubscription;
  static StreamSubscription? _positionSubscription;
  static final _fftController = StreamController<List<double>>.broadcast();
  static final _positionController = StreamController<int>.broadcast();

  /// FFT 数据流（实时频谱）
  static Stream<List<double>> get fftStream => _fftController.stream;

  /// 播放位置流（毫秒）
  static Stream<int> get positionStream => _positionController.stream;

  /// 当前位置（毫秒）
  static int _currentPosition = 0;
  static int get currentPosition => _currentPosition;

  /// 总时长（毫秒）
  static int _duration = 0;
  static int get duration => _duration;

  /// 是否正在播放
  static bool _isPlaying = false;
  static bool get isPlaying => _isPlaying;

  static bool get isSupported => !kIsWeb && Platform.isMacOS;

  /// 播放本地文件
  static Future<bool> play(String filePath) async {
    if (!isSupported) return false;
    try {
      await _channel.invokeMethod('play', {'filePath': filePath});
      _isPlaying = true;
      _duration = await _channel.invokeMethod('getDuration') ?? 0;
      _startListening();
      return true;
    } catch (e) {
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
    _currentPosition = 0;
    _stopListening();
  }

  /// 跳转到指定位置（毫秒）
  static Future<void> seek(int positionMs) async {
    if (!isSupported) return;
    await _channel.invokeMethod('seek', {'positionMs': positionMs});
    _currentPosition = positionMs;
  }

  static void _startListening() {
    _stopListening();
    _pcmSubscription = _pcmChannel.receiveBroadcastStream().listen((event) {
      if (event is List) {
        final data = event.map((e) => (e as num).toDouble()).toList();
        LOGGER.i('[MacOSPlayer] PCM received: ${data.length} values, first=${data.isNotEmpty ? data.first.toStringAsFixed(2) : "empty"}');
        _fftController.add(data);
      }
    });
    _positionSubscription = _positionChannel.receiveBroadcastStream().listen((event) {
      if (event is int) {
        _currentPosition = event;
        _positionController.add(event);
      }
    });
  }

  static void _stopListening() {
    _pcmSubscription?.cancel();
    _pcmSubscription = null;
    _positionSubscription?.cancel();
    _positionSubscription = null;
  }

  static void dispose() {
    _stopListening();
    _fftController.close();
    _positionController.close();
  }
}
