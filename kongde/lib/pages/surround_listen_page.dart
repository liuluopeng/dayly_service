import 'dart:async';

import 'package:audio_session/audio_session.dart';
import 'package:flutter/foundation.dart' show kIsWeb, defaultTargetPlatform, TargetPlatform;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_sound/flutter_sound.dart';
import 'package:get/get.dart';
import 'package:permission_handler/permission_handler.dart';

/// 环境监听页：戴耳机时把麦克风采集的外界声音实时回放，
/// 避免错过环境动静（如有人叫你）。
///
/// 实现：
/// - macOS：原生 AVAudioEngine 麦克风直通耳机（MethodChannel `kongde/native_audio`）
/// - 其他平台：flutter_sound 流式回环（录音流 → 播放器）
class SurroundListenPage extends StatefulWidget {
  const SurroundListenPage({super.key});

  @override
  State<SurroundListenPage> createState() => _SurroundListenPageState();
}

class _SurroundListenPageState extends State<SurroundListenPage> {
  static const MethodChannel _macChannel = MethodChannel('kongde/native_audio');

  final FlutterSoundRecorder _recorder = FlutterSoundRecorder();
  final FlutterSoundPlayer _player = FlutterSoundPlayer();
  StreamController<Uint8List>? _loopbackController;

  bool _listening = false;
  bool _paused = false;
  double _gain = 1.0;
  String _status = '';

  bool get _isMacOS =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

  @override
  void initState() {
    super.initState();
    _init();
  }

  @override
  void dispose() {
    _stopListening();
    super.dispose();
  }

  Future<void> _init() async {
    try {
      if (!_isMacOS) {
        final session = await AudioSession.instance;
        await session.configure(AudioSessionConfiguration(
          avAudioSessionCategory: AVAudioSessionCategory.playAndRecord,
          avAudioSessionCategoryOptions: AVAudioSessionCategoryOptions.defaultToSpeaker |
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
      final status = await Permission.microphone.request();
      if (status != PermissionStatus.granted) {
        throw Exception('需要麦克风权限');
      }
    }
    // iOS/macOS 首次使用麦克风时系统自动弹窗
  }

  Future<void> _toggleListening() async {
    if (_listening) {
      await _stopListening();
      return;
    }
    try {
      await _ensurePermission();
      if (_isMacOS) {
        await _macChannel.invokeMethod('surroundStart');
        await _macChannel.invokeMethod('surroundSetGain', {'gain': _gain});
      } else {
        await _startLoopback();
      }
      if (!mounted) return;
      setState(() {
        _listening = true;
        _paused = false;
        _status = 'surround.listenHint'.tr;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _status = 'surround.startFailed'.trParams({'error': '$e'}));
    }
  }

  Future<void> _stopListening() async {
    if (_isMacOS) {
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
    if (mounted) {
      setState(() {
        _listening = false;
        _paused = false;
      });
    }
  }

  Future<void> _togglePause() async {
    if (_isMacOS) {
      try {
        if (_paused) {
          await _macChannel.invokeMethod('surroundResume');
        } else {
          await _macChannel.invokeMethod('surroundPause');
        }
      } catch (_) {}
    } else {
      if (mounted) setState(() => _paused = !_paused);
      return;
    }
    if (mounted) setState(() {});
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
      if (_paused) return;
      final bytes = data.toList();
      for (int i = 0; i + 1 < bytes.length; i += 2) {
        int sample = (bytes[i] & 0xFF) | ((bytes[i + 1] & 0xFF) << 8);
        if ((sample & 0x8000) != 0) sample = sample - 0x10000;
        double amplified = sample * _gain;
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

  Future<void> _setGain(double v) async {
    setState(() => _gain = v);
    if (_isMacOS && _listening) {
      try {
        await _macChannel.invokeMethod('surroundSetGain', {'gain': v});
      } catch (_) {}
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('surround.title'.tr)),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('surround.intro'.tr, style: TextStyle(color: Colors.grey[600])),
                      const SizedBox(height: 8),
                      Text('surround.warning'.tr, style: TextStyle(color: Colors.orange[700], fontSize: 12)),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 24),
              ElevatedButton.icon(
                onPressed: _toggleListening,
                icon: Icon(_listening ? Icons.hearing_disabled : Icons.hearing),
                label: Text(_listening ? 'surround.stop'.tr : 'surround.start'.tr),
                style: ElevatedButton.styleFrom(
                  backgroundColor: _listening ? Colors.redAccent : null,
                  padding: const EdgeInsets.symmetric(vertical: 16),
                  textStyle: const TextStyle(fontSize: 16),
                ),
              ),
              if (_listening) ...[
                const SizedBox(height: 12),
                ElevatedButton.icon(
                  onPressed: _togglePause,
                  icon: Icon(_paused ? Icons.play_arrow : Icons.pause),
                  label: Text(_paused ? 'surround.resume'.tr : 'surround.pause'.tr),
                ),
              ],
              const SizedBox(height: 24),
              Row(
                children: [
                  Text('surround.gain'.tr),
                  Expanded(
                    child: Slider(
                      value: _gain,
                      onChanged: _setGain,
                      min: 0.1,
                      max: 6.0,
                      divisions: 59,
                      label: '${_gain.toStringAsFixed(2)}x',
                    ),
                  ),
                  Text('${_gain.toStringAsFixed(1)}x'),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                _status,
                style: TextStyle(color: _listening ? Colors.green[700] : Colors.grey[600]),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
