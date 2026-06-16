import 'dart:math' as math;
import 'package:flutter/material.dart';

class SpectrumPainter extends CustomPainter {
  final List<double> fftData;
  final Color color;

  SpectrumPainter(this.fftData, this.color);

  double _hybridScale(double freq, double minFreq, double maxFreq) {
    final crossoverFreq = 200.0;

    if (freq < crossoverFreq) {
      final linearRange = crossoverFreq - minFreq;
      return (freq - minFreq) / linearRange * 0.15;
    } else {
      final logMin = math.log(crossoverFreq);
      final logMax = math.log(maxFreq);
      final logValue = math.log(freq);
      return 0.15 + (logValue - logMin) / (logMax - logMin) * 0.85;
    }
  }

  double _enhanceValue(double value, double freq) {
    final bassBoost = freq < 150 ? 1.3 : 1.0;
    final midBoost = freq >= 150 && freq < 3000 ? 1.4 : 1.0;
    final highBoost = freq >= 3000 && freq < 8000 ? 1.2 : 1.0;

    final boostedValue = value * bassBoost * midBoost * highBoost;

    return math.pow(boostedValue, 0.8).toDouble();
  }

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.fill;

    final maxHeight = size.height - 20;
    final minFreq = 20.0;
    final maxFreq = 20000.0;
    final sampleRate = 44100.0;
    final fftSize = fftData.length;

    for (int i = 0; i < fftData.length; i++) {
      final freq = (sampleRate * i) / (2 * fftSize);
      final displayFreq = math.max(minFreq, math.min(freq, maxFreq));

      final normalizedValue = fftData[i] / 100.0;
      final enhancedValue = _enhanceValue(normalizedValue, displayFreq);
      final barHeight = enhancedValue * maxHeight;

      final xPos = _hybridScale(displayFreq, minFreq, maxFreq) * size.width;

      final nextFreq = (sampleRate * (i + 1)) / (2 * fftSize);
      final nextDisplayFreq = math.max(minFreq, math.min(nextFreq, maxFreq));
      final nextXPos =
          _hybridScale(nextDisplayFreq, minFreq, maxFreq) * size.width;

      final barWidth = (nextXPos - xPos).clamp(0.5, size.width);

      final rect = Rect.fromLTWH(
        xPos,
        size.height - 20 - barHeight,
        barWidth - 0.3,
        barHeight,
      );

      canvas.drawRect(rect, paint);
    }

    final labelPaint = TextPainter(
      text: const TextSpan(
        text: '20Hz',
        style: TextStyle(color: Colors.grey, fontSize: 10),
      ),
      textDirection: TextDirection.ltr,
    );
    labelPaint.layout();
    labelPaint.paint(canvas, Offset(0, size.height - 18));

    final labelPaint2 = TextPainter(
      text: const TextSpan(
        text: '200Hz',
        style: TextStyle(color: Colors.grey, fontSize: 10),
      ),
      textDirection: TextDirection.ltr,
    );
    labelPaint2.layout();
    final midPos = _hybridScale(200, minFreq, maxFreq) * size.width;
    labelPaint2.paint(canvas, Offset(midPos - 15, size.height - 18));

    final labelPaint3 = TextPainter(
      text: const TextSpan(
        text: '2kHz',
        style: TextStyle(color: Colors.grey, fontSize: 10),
      ),
      textDirection: TextDirection.ltr,
    );
    labelPaint3.layout();
    final midPos2 = _hybridScale(2000, minFreq, maxFreq) * size.width;
    labelPaint3.paint(canvas, Offset(midPos2 - 12, size.height - 18));

    final labelPaint4 = TextPainter(
      text: const TextSpan(
        text: '20kHz',
        style: TextStyle(color: Colors.grey, fontSize: 10),
      ),
      textDirection: TextDirection.ltr,
    );
    labelPaint4.layout();
    labelPaint4.paint(canvas, Offset(size.width - 30, size.height - 18));
  }

  @override
  bool shouldRepaint(covariant SpectrumPainter oldDelegate) {
    return oldDelegate.fftData != fftData || oldDelegate.color != color;
  }
}
