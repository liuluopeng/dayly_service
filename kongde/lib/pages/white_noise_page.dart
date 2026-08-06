import 'dart:typed_data';
import 'dart:math' as math;

import 'package:audioplayers/audioplayers.dart';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/noise.dart';
import 'package:kongde/widgets/common_app_bar.dart';

enum NoiseKind { white, pink, brown, rain }

class WhiteNoisePage extends StatefulWidget {
  const WhiteNoisePage({super.key});

  @override
  State<WhiteNoisePage> createState() => _WhiteNoisePageState();
}

class _WhiteNoisePageState extends State<WhiteNoisePage> {
  final AudioPlayer _player = AudioPlayer();
  NoiseKind? _active;
  bool _generating = false;

  static const _sampleRate = 48000;
  static const _durationMs = 10000;

  @override
  void dispose() {
    _player.dispose();
    super.dispose();
  }

  Future<void> _toggle(NoiseKind kind) async {
    if (_generating) return;
    if (_active == kind) {
      await _player.stop();
      setState(() => _active = null);
      return;
    }

    setState(() {
      _generating = true;
      _active = kind;
    });

    try {
      final pcm = await _generate(kind);
      final wav = _f32ToWav(pcm);
      await _player.stop();
      await _player.play(BytesSource(wav));
    } catch (e) {
      setState(() => _active = null);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('生成失败: $e')));
      }
    } finally {
      if (mounted) setState(() => _generating = false);
    }
  }

  Future<Float32List> _generate(NoiseKind kind) async {
    final seed = math.Random().nextInt(0xFFFFFFF);
    switch (kind) {
      case NoiseKind.white:
        return whiteNoisePcm(durationMs: _durationMs, sampleRate: _sampleRate, seed: seed);
      case NoiseKind.pink:
        return pinkNoisePcm(durationMs: _durationMs, sampleRate: _sampleRate, seed: seed);
      case NoiseKind.brown:
        return brownNoisePcm(durationMs: _durationMs, sampleRate: _sampleRate, seed: seed);
      case NoiseKind.rain:
        return rainNoisePcm(durationMs: _durationMs, sampleRate: _sampleRate, seed: seed);
    }
  }

  /// f32 PCM → 16-bit WAV（audioplayers BytesSource 播放，循环无缝）
  Uint8List _f32ToWav(Float32List pcm) {
    final numSamples = pcm.length;
    final dataSize = numSamples * 2;
    final bytes = Uint8List(44 + dataSize);
    final data = ByteData.sublistView(bytes);

    void writeString(int offset, String s) {
      for (var i = 0; i < s.length; i++) {
        data.setUint8(offset + i, s.codeUnitAt(i));
      }
    }

    writeString(0, 'RIFF');
    data.setUint32(4, 36 + dataSize, Endian.little);
    writeString(8, 'WAVE');
    writeString(12, 'fmt ');
    data.setUint32(16, 16, Endian.little);
    data.setUint16(20, 1, Endian.little); // PCM
    data.setUint16(22, 1, Endian.little); // mono
    data.setUint32(24, _sampleRate, Endian.little);
    data.setUint32(28, _sampleRate * 2, Endian.little); // byte rate
    data.setUint16(32, 2, Endian.little); // block align
    data.setUint16(34, 16, Endian.little); // bits
    writeString(36, 'data');
    data.setUint32(40, dataSize, Endian.little);

    for (var i = 0; i < numSamples; i++) {
      final v = (pcm[i].clamp(-1.0, 1.0) * 32767.0).round();
      data.setInt16(44 + i * 2, v, Endian.little);
    }
    return bytes;
  }

  Widget _noiseButton(NoiseKind kind, String label, IconData icon) {
    final active = _active == kind;
    return SizedBox(
      width: 140,
      height: 140,
      child: FilledButton(
        style: FilledButton.styleFrom(
          backgroundColor: active ? Theme.of(context).colorScheme.primary : Colors.grey.shade200,
          foregroundColor: active ? Colors.white : Colors.black87,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
        ),
        onPressed: _generating ? null : () => _toggle(kind),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 40),
            const SizedBox(height: 8),
            Text(label, style: const TextStyle(fontSize: 16)),
            if (_generating && active)
              const Padding(
                padding: EdgeInsets.only(top: 6),
                child: SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)),
              ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '白噪音'),
      body: SafeArea(
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Text('Wasm 实时合成，点击播放/停止（10 秒无缝循环）', style: TextStyle(color: Colors.grey)),
              const SizedBox(height: 32),
              Wrap(
                spacing: 24,
                runSpacing: 24,
                alignment: WrapAlignment.center,
                children: [
                  _noiseButton(NoiseKind.white, '白噪音', Icons.grain),
                  _noiseButton(NoiseKind.pink, '粉红噪音', Icons.waves),
                  _noiseButton(NoiseKind.brown, '棕噪音', Icons.thunderstorm),
                  _noiseButton(NoiseKind.rain, '雨声', Icons.umbrella),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
