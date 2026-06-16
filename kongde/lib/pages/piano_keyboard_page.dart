import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:audioplayers/audioplayers.dart';
import 'package:path_provider/path_provider.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class PianoKeyboardPage extends StatefulWidget {
  const PianoKeyboardPage({super.key});

  @override
  State<PianoKeyboardPage> createState() => _PianoKeyboardPageState();
}

class _PianoKeyboardPageState extends State<PianoKeyboardPage> {
  static const double _a4Frequency = 440.0;
  static const int _a4MidiNumber = 69;
  static const int _startMidi = 48; // C3
  static const int _endMidi = 72; // C5

  final Set<int> _pressedKeys = {};
  final Map<int, AudioPlayer> _players = {};
  final Map<int, String> _cachedWavPaths = {};

  static const List<String> _noteNames = [
    'C',
    'C#',
    'D',
    'D#',
    'E',
    'F',
    'F#',
    'G',
    'G#',
    'A',
    'A#',
    'B',
  ];

  double _getFrequency(int midiNumber) {
    return _a4Frequency * math.pow(2, (midiNumber - _a4MidiNumber) / 12.0);
  }

  String _getNoteName(int midiNumber) {
    int noteIndex = midiNumber % 12;
    int octave = (midiNumber ~/ 12) - 1;
    return '${_noteNames[noteIndex]}$octave';
  }

  bool _isBlackKey(int noteInOctave) {
    return [1, 3, 6, 8, 10].contains(noteInOctave);
  }

  int _getWhiteKeyIndex(int midiNumber) {
    int count = 0;
    for (int i = _startMidi; i < midiNumber; i++) {
      if (!_isBlackKey(i % 12)) {
        count++;
      }
    }
    return count;
  }

  // 钢琴音色合成：包含基频和泛音
  Uint8List _generatePianoTone(double frequency, {int durationMs = 1000}) {
    const int sampleRate = 44100;
    int numSamples = (sampleRate * durationMs / 1000).round();
    final buffer = Float32List(numSamples);

    // 钢琴音色的泛音结构
    final partials = [
      {'freq': 1.0, 'amp': 1.0, 'decay': 0.8},
      {'freq': 2.0, 'amp': 0.7, 'decay': 0.6},
      {'freq': 3.0, 'amp': 0.5, 'decay': 0.4},
      {'freq': 4.0, 'amp': 0.3, 'decay': 0.3},
      {'freq': 5.0, 'amp': 0.2, 'decay': 0.2},
      {'freq': 6.0, 'amp': 0.1, 'decay': 0.1},
    ];

    for (int i = 0; i < numSamples; i++) {
      double t = i / sampleRate;
      double sample = 0.0;

      // 计算每个泛音
      for (var partial in partials) {
        double partialFreq = frequency * partial['freq']!;
        double partialAmp = partial['amp']!;
        double partialDecay = partial['decay']!;

        // 指数衰减
        double envelope = math.exp(-t * 3 * (1 - partialDecay));

        // 添加一些随机的起始瞬态，模拟钢琴 hammer 敲击
        if (i < sampleRate * 0.01) {
          envelope *= (i / (sampleRate * 0.01));
        }

        sample +=
            math.sin(2 * math.pi * partialFreq * t) * partialAmp * envelope;
      }

      // 主包络
      double mainEnvelope = 1.0;
      if (i < sampleRate * 0.02) {
        mainEnvelope = i / (sampleRate * 0.02);
      } else if (i > sampleRate * 0.3) {
        mainEnvelope = math.exp(-(i - sampleRate * 0.3) / (sampleRate * 0.5));
      }

      buffer[i] = (sample * mainEnvelope * 0.3).clamp(-1.0, 1.0);
    }

    return _createWavFile(buffer, sampleRate);
  }

  Uint8List _createWavFile(Float32List samples, int sampleRate) {
    final int numSamples = samples.length;
    final int dataSize = numSamples * 2;
    final int fileSize = 44 + dataSize;

    final buffer = ByteData(fileSize);
    int offset = 0;

    void writeString(String s) {
      for (int i = 0; i < s.length; i++) {
        buffer.setUint8(offset++, s.codeUnitAt(i));
      }
    }

    void writeUint32(int value) {
      buffer.setUint32(offset, value, Endian.little);
      offset += 4;
    }

    void writeUint16(int value) {
      buffer.setUint16(offset, value, Endian.little);
      offset += 2;
    }

    writeString('RIFF');
    writeUint32(fileSize - 8);
    writeString('WAVE');
    writeString('fmt ');
    writeUint32(16);
    writeUint16(1);
    writeUint16(1);
    writeUint32(sampleRate);
    writeUint32(sampleRate * 2);
    writeUint16(2);
    writeUint16(16);
    writeString('data');
    writeUint32(dataSize);

    for (int i = 0; i < numSamples; i++) {
      int sample = (samples[i] * 32767).round().clamp(-32768, 32767);
      buffer.setInt16(offset, sample, Endian.little);
      offset += 2;
    }

    return buffer.buffer.asUint8List();
  }

  Future<String> _getWavPath(int midiNumber) async {
    if (_cachedWavPaths.containsKey(midiNumber)) {
      return _cachedWavPaths[midiNumber]!;
    }

    final frequency = _getFrequency(midiNumber);
    final wavData = _generatePianoTone(frequency);

    final tempDir = await getTemporaryDirectory();
    final file = File('${tempDir.path}/piano_$midiNumber.wav');
    await file.writeAsBytes(wavData);

    _cachedWavPaths[midiNumber] = file.path;
    return file.path;
  }

  Future<void> _playTone(int midiNumber) async {
    if (_players.containsKey(midiNumber)) return;

    final player = AudioPlayer();
    _players[midiNumber] = player;

    try {
      final path = await _getWavPath(midiNumber);
      await player.play(DeviceFileSource(path));
    } catch (e) {
      debugPrint('Error playing tone: $e');
      _players.remove(midiNumber);
      player.dispose();
    }
  }

  Future<void> _stopTone(int midiNumber) async {
    final player = _players.remove(midiNumber);
    if (player != null) {
      await player.stop();
      await player.dispose();
    }
  }

  void _onKeyPressed(int midiNumber) {
    setState(() {
      _pressedKeys.add(midiNumber);
    });
    _playTone(midiNumber);
    final frequency = _getFrequency(midiNumber);
    final noteName = _getNoteName(midiNumber);
    Get.snackbar(
      'Key Pressed',
      '$noteName (${frequency.toStringAsFixed(2)} Hz)',
      duration: const Duration(milliseconds: 500),
      snackPosition: SnackPosition.bottom,
      backgroundColor: Colors.black54,
      colorText: Colors.white,
    );
  }

  void _onKeyReleased(int midiNumber) {
    setState(() {
      _pressedKeys.remove(midiNumber);
    });
    _stopTone(midiNumber);
  }

  @override
  void dispose() {
    for (final player in _players.values) {
      player.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'piano.title'.tr),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: EdgeInsets.all(16.0),
              child: Text(
                'piano.standardPitch'.tr,
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final totalWhiteKeys = _countWhiteKeys(_startMidi, _endMidi);
                  final keyWidth = constraints.maxWidth / totalWhiteKeys;
                  final keyHeight = constraints.maxHeight * 0.8;

                  return Stack(
                    children: [
                      Row(
                        children: _buildWhiteKeys(
                          _startMidi,
                          _endMidi,
                          keyWidth,
                          keyHeight,
                        ),
                      ),
                      ..._buildBlackKeys(
                        _startMidi,
                        _endMidi,
                        keyWidth,
                        keyHeight,
                      ),
                    ],
                  );
                },
              ),
            ),
            _buildLegend(),
          ],
        ),
      ),
    );
  }

  int _countWhiteKeys(int startMidi, int endMidi) {
    int count = 0;
    for (int i = startMidi; i <= endMidi; i++) {
      if (!_isBlackKey(i % 12)) count++;
    }
    return count;
  }

  List<Widget> _buildWhiteKeys(
    int startMidi,
    int endMidi,
    double keyWidth,
    double keyHeight,
  ) {
    final List<Widget> keys = [];
    for (int midi = startMidi; midi <= endMidi; midi++) {
      if (!_isBlackKey(midi % 12)) {
        bool isC4 = midi == 60; // 标记 C4 位置
        keys.add(
          Expanded(
            child: GestureDetector(
              onTapDown: (_) => _onKeyPressed(midi),
              onTapUp: (_) => _onKeyReleased(midi),
              onTapCancel: () => _onKeyReleased(midi),
              child: Container(
                height: keyHeight,
                margin: const EdgeInsets.symmetric(horizontal: 1),
                decoration: BoxDecoration(
                  color: _pressedKeys.contains(midi)
                      ? Colors.blue.shade100
                      : isC4
                      ? Colors
                            .yellow
                            .shade100 // C4 用黄色标记
                      : Colors.white,
                  borderRadius: BorderRadius.only(
                    bottomLeft: Radius.circular(4),
                    bottomRight: Radius.circular(4),
                  ),
                  border: Border.all(
                    color: isC4 ? Colors.yellow.shade800 : Colors.black54,
                    width: isC4 ? 2 : 1,
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.2),
                      blurRadius: 2,
                      offset: const Offset(0, 2),
                    ),
                  ],
                ),
                child: Align(
                  alignment: Alignment.bottomCenter,
                  child: Padding(
                    padding: const EdgeInsets.only(bottom: 8.0),
                    child: Text(
                      _getNoteName(midi),
                      style: TextStyle(
                        fontSize: 10,
                        color: isC4 ? Colors.yellow.shade800 : Colors.black54,
                        fontWeight: isC4 ? FontWeight.bold : FontWeight.normal,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      }
    }
    return keys;
  }

  List<Widget> _buildBlackKeys(
    int startMidi,
    int endMidi,
    double keyWidth,
    double keyHeight,
  ) {
    final List<Widget> keys = [];

    for (int midi = startMidi; midi <= endMidi; midi++) {
      int noteInOctave = midi % 12;
      if (_isBlackKey(noteInOctave)) {
        // 计算这个黑键左边的白键索引
        int leftWhiteKeyMidi = midi - 1;
        while (_isBlackKey(leftWhiteKeyMidi % 12)) {
          leftWhiteKeyMidi--;
        }

        // 计算白键在当前键盘中的索引
        int whiteKeyIndex = _getWhiteKeyIndex(leftWhiteKeyMidi);

        // 黑键应该位于两个白键之间
        double leftPosition = (whiteKeyIndex + 0.65) * keyWidth;

        keys.add(
          Positioned(
            left: leftPosition,
            top: 0,
            child: GestureDetector(
              onTapDown: (_) => _onKeyPressed(midi),
              onTapUp: (_) => _onKeyReleased(midi),
              onTapCancel: () => _onKeyReleased(midi),
              child: Container(
                width: keyWidth * 0.7,
                height: keyHeight * 0.6,
                decoration: BoxDecoration(
                  color: _pressedKeys.contains(midi)
                      ? Colors.blue.shade800
                      : Colors.black87,
                  borderRadius: BorderRadius.only(
                    bottomLeft: Radius.circular(4),
                    bottomRight: Radius.circular(4),
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.3),
                      blurRadius: 2,
                      offset: const Offset(0, 2),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      }
    }
    return keys;
  }

  Widget _buildLegend() {
    return Container(
      padding: const EdgeInsets.all(16.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            width: 20,
            height: 20,
            margin: const EdgeInsets.only(right: 8),
            decoration: BoxDecoration(
              color: Colors.white,
              border: Border.all(color: Colors.black54),
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          Text('piano.whiteKey'.tr, style: TextStyle(fontSize: 12)),
          const SizedBox(width: 16),
          Container(
            width: 20,
            height: 20,
            margin: const EdgeInsets.only(right: 8),
            decoration: BoxDecoration(
              color: Colors.black87,
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          Text('piano.blackKey'.tr, style: TextStyle(fontSize: 12)),
          const SizedBox(width: 16),
          Container(
            width: 20,
            height: 20,
            margin: const EdgeInsets.only(right: 8),
            decoration: BoxDecoration(
              color: Colors.yellow.shade100,
              border: Border.all(color: Colors.yellow.shade800, width: 2),
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          Text('piano.middleC'.tr, style: TextStyle(fontSize: 12)),
        ],
      ),
    );
  }
}
