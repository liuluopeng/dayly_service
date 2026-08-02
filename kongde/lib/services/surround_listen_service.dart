import 'dart:async';

import 'package:audio_session/audio_session.dart';
import 'package:flutter/foundation.dart'
    show debugPrint, defaultTargetPlatform, kIsWeb, TargetPlatform;
import 'package:flutter/services.dart';
import 'package:flutter_sound/flutter_sound.dart';
import 'package:get/get.dart';
import 'package:permission_handler/permission_handler.dart';

/// 环境监听服务（单例）：macOS 走原生 AVAudioEngine 直通，
/// 其他平台走 flutter_sound 流式回环。
/// 监听页与首页快捷小组件共享同一实例，状态实时同步。
class SurroundListenService extends GetxController {
  static SurroundListenService get instance =>
      Get.put(SurroundListenService(), permanent: true);

  static const MethodChannel _macChannel = MethodChannel('kongde/native_audio');

  final FlutterSoundRecorder _recorder = FlutterSoundRecorder();
  final FlutterSoundPlayer _player = FlutterSoundPlayer();
  StreamController<Uint8List>? _loopbackController;

  final RxBool listening = false.obs;
  final RxBool paused = false.obs;
  final RxDouble gain = 1.0.obs;
  final RxString status = ''.obs;

  bool _initialized = false;

  bool get isMacOS => !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

  Future<void> ensureInit() async {
    if (_initialized || kIsWeb) return;
    _initialized = true;
    try {
      if (!isMacOS) {
        final session = await AudioSession.instance;
        await session.configure(AudioSessionConfiguration(
          avAudioSessionCategory: AVAudioSessionCategory.playAndRecord,
          avAudioSessionCategoryOptions: AVAudioSessionCategoryOptions
                  .defaultToSpeaker |
              AVAudioSessionCategoryOptions.mixWithOthers |
              AVAudioSessionCategoryOptions.allowBluetooth,
          avAudioSessionMode: AVAudioSessionMode.measurement,
          androidAudioAttributes: AndroidAudioAttributes(
            contentType: AndroidAudioContentType.speech,
            usage: AndroidAudioUsage.voiceCommunication,
            flags: AndroidAudioFlags.none,
          ),
          androidAudioFocusGainType: AndroidAudioFocusGainType.gain,
          androidWillPauseWhenDucked: false,
        ));
        await _player.openPlayer();
        await _recorder.openRecorder();
      }
    } catch (e) {
      debugPrint('监听初始化失败: $e');
    }
  }

  Future<void> _ensurePermission() async {
    if (defaultTargetPlatform == TargetPlatform.android) {
      final s = await Permission.microphone.request();
      if (s != PermissionStatus.granted) {
        throw Exception('需要麦克风权限');
      }
    }
  }

  Future<void> start() async {
    await ensureInit();
    await _ensurePermission();
    if (isMacOS) {
      await _macChannel.invokeMethod('surroundStart');
      await _macChannel.invokeMethod('surroundSetGain', {'gain': gain.value});
    } else {
      await _startLoopback();
    }
    listening.value = true;
    paused.value = false;
    status.value = 'surround.listenHint'.tr;
  }

  Future<void> stop() async {
    if (isMacOS) {
      try {
        await _macChannel.invokeMethod('surroundStop');
      } catch (_) {}
    } else {
      try {
        await _recorder.stopRecorder();
      } catch (_) {}
      try {
        await _player.stopPlayer();
      } catch (_) {}
      await _loopbackController?.close();
      _loopbackController = null;
    }
    listening.value = false;
    paused.value = false;
    status.value = '';
  }

  Future<void> togglePause() async {
    if (isMacOS) {
      try {
        if (paused.value) {
          await _macChannel.invokeMethod('surroundResume');
        } else {
          await _macChannel.invokeMethod('surroundPause');
        }
      } catch (_) {}
    } else {
      paused.value = !paused.value;
      return;
    }
    paused.value = !paused.value;
  }

  Future<void> setGain(double v) async {
    gain.value = v;
    if (isMacOS && listening.value) {
      try {
        await _macChannel.invokeMethod('surroundSetGain', {'gain': v});
      } catch (_) {}
    }
  }

  Future<void> _startLoopback() async {
    const int sampleRate = 44100;
    const int numChannels = 1;
    const int bufferSize = 8192;

    await _player.startPlayerFromStream(
      codec: Codec.pcm16,
      numChannels: numChannels,
      sampleRate: sampleRate,
      bufferSize: bufferSize,
      interleaved: true,
    );

    _loopbackController = StreamController<Uint8List>(onCancel: () async {
      try {
        await _player.stopPlayer();
      } catch (_) {}
    });

    _loopbackController!.stream.listen((Uint8List data) {
      if (paused.value) return;
      final bytes = data.toList();
      for (int i = 0; i + 1 < bytes.length; i += 2) {
        int sample = (bytes[i] & 0xFF) | ((bytes[i + 1] & 0xFF) << 8);
        if ((sample & 0x8000) != 0) sample = sample - 0x10000;
        double amplified = sample * gain.value;
        if (amplified > 32767) amplified = 32767;
        if (amplified < -32768) amplified = -32768;
        int s = amplified.toInt();
        if (s < 0) s = 0x10000 + s;
        bytes[i] = s & 0xFF;
        bytes[i + 1] = (s >> 8) & 0xFF;
      }
      _player.feedUint8FromStream(Uint8List.fromList(bytes));
    });

    await _recorder.startRecorder(
      toStream: _loopbackController!.sink,
      codec: Codec.pcm16,
      sampleRate: sampleRate,
      numChannels: numChannels,
      audioSource: AudioSource.microphone,
    );
  }

  @override
  void onClose() {
    stop();
    super.onClose();
  }
}
