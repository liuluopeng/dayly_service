import 'package:flutter/material.dart';

class SeekBar extends StatefulWidget {
  final Duration duration;
  final Duration position;
  final ValueChanged<Duration> onChangeEnd;
  final Color activeColor;
  final Color inactiveColor;
  final Color textColor;

  const SeekBar({
    super.key,
    required this.duration,
    required this.position,
    required this.onChangeEnd,
    required this.activeColor,
    required this.inactiveColor,
    required this.textColor,
  });

  @override
  State<SeekBar> createState() => _SeekBarState();
}

class _SeekBarState extends State<SeekBar> {
  double? _dragValue;

  String _formatDuration(Duration duration) {
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final minutes = twoDigits(duration.inMinutes.remainder(60));
    final seconds = twoDigits(duration.inSeconds.remainder(60));
    return '$minutes:$seconds';
  }

  @override
  Widget build(BuildContext context) {
    final maxDuration = widget.duration.inMilliseconds.toDouble();
    final currentValue = _dragValue ?? widget.position.inMilliseconds.toDouble();
    final clampedValue = currentValue.clamp(0.0, maxDuration > 0 ? maxDuration : 1.0);

    return Column(
      children: [
        SliderTheme(
          data: SliderThemeData(
            activeTrackColor: widget.activeColor,
            inactiveTrackColor: widget.inactiveColor,
            thumbColor: widget.activeColor,
            activeTickMarkColor: widget.activeColor,
            inactiveTickMarkColor: widget.inactiveColor,
          ),
          child: Slider(
            min: 0.0,
            max: maxDuration > 0 ? maxDuration : 1.0,
            value: clampedValue,
            onChanged: (value) {
              setState(() {
                _dragValue = value;
              });
            },
            onChangeEnd: (value) {
              widget.onChangeEnd(Duration(milliseconds: value.toInt()));
              setState(() {
                _dragValue = null;
              });
            },
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                _formatDuration(widget.position),
                style: TextStyle(color: widget.textColor),
              ),
              Text(
                _formatDuration(widget.duration),
                style: TextStyle(color: widget.textColor),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
