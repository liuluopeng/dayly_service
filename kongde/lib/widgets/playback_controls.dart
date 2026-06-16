import 'package:flutter/material.dart';
import 'package:audio_service/audio_service.dart';
import 'package:get/get.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:flutter/foundation.dart';

class PlaybackControls extends StatefulWidget {
  final bool isPlaying;
  final Color iconColor;
  final VoidCallback onLikeChanged;

  const PlaybackControls({
    super.key,
    required this.isPlaying,
    required this.iconColor,
    required this.onLikeChanged,
  });

  @override
  State<PlaybackControls> createState() => _PlaybackControlsState();
}

class _PlaybackControlsState extends State<PlaybackControls> {
  bool _showVolumeSlider = false;

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<MediaItem?>(
      stream: Get.find<AudioPlayerHandler>().mediaItem,
      builder: (context, mediaSnapshot) {
        final mediaItem = mediaSnapshot.data;
        final extras = mediaItem?.extras ?? {};
        final musicId = extras['musicId'] as int?;

        return Column(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
                _button(
                  Icons.skip_previous,
                  Get.find<AudioPlayerHandler>().skipToPrevious,
                  widget.iconColor,
                  48,
                ),
                if (widget.isPlaying)
                  _button(
                    Icons.pause,
                    Get.find<AudioPlayerHandler>().pauseDirectly,
                    widget.iconColor,
                    56,
                  )
                else
                  _button(
                    Icons.play_arrow,
                    Get.find<AudioPlayerHandler>().playDirectly,
                    widget.iconColor,
                    56,
                  ),
                _button(
                  Icons.skip_next,
                  Get.find<AudioPlayerHandler>().skipToNext,
                  widget.iconColor,
                  48,
                ),
                const SizedBox(width: 8),
                _VolumeControlButton(
                  iconColor: widget.iconColor,
                  showSlider: _showVolumeSlider,
                  onToggleSlider: () {
                    setState(() {
                      _showVolumeSlider = !_showVolumeSlider;
                    });
                  },
                ),
              ],
            ),
            if (_showVolumeSlider) _VolumeSlider(iconColor: widget.iconColor),
          ],
        );
      },
    );
  }

  IconButton _button(
    IconData iconData,
    VoidCallback onPressed,
    Color color, [
    double? size,
  ]) => IconButton(
    icon: Icon(iconData, color: color),
    iconSize: size ?? 64.0,
    onPressed: onPressed,
  );
}

class _VolumeControlButton extends StatelessWidget {
  final Color iconColor;
  final bool showSlider;
  final VoidCallback onToggleSlider;

  const _VolumeControlButton({
    required this.iconColor,
    required this.showSlider,
    required this.onToggleSlider,
  });

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<double>(
      stream: Get.find<AudioPlayerHandler>().player.volumeStream,
      builder: (context, snapshot) {
        final volume = snapshot.data ?? 1.0;
        IconData icon;

        if (volume == 0) {
          icon = Icons.volume_off;
        } else if (volume < 0.5) {
          icon = Icons.volume_down;
        } else {
          icon = Icons.volume_up;
        }

        return IconButton(
          icon: Icon(icon, color: iconColor),
          iconSize: 48,
          onPressed: onToggleSlider,
        );
      },
    );
  }
}

class _VolumeSlider extends StatelessWidget {
  final Color iconColor;

  const _VolumeSlider({required this.iconColor});

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<double>(
      stream: Get.find<AudioPlayerHandler>().player.volumeStream,
      builder: (context, snapshot) {
        final volume = snapshot.data ?? 1.0;

        return Container(
          width: 200,
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: Slider(
            value: volume,
            min: 0.0,
            max: 1.0,
            divisions: 10,
            activeColor: iconColor,
            inactiveColor: iconColor.withOpacity(0.3),
            onChanged: (value) {
              Get.find<AudioPlayerHandler>().player.setVolume(value);
            },
          ),
        );
      },
    );
  }
}
