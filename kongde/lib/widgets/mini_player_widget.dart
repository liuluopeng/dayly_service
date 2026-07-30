import 'package:flutter/material.dart';
import 'package:audio_service/audio_service.dart';
import 'package:just_audio/just_audio.dart';
import 'package:get/get.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:kongde/widgets/album_art_widget.dart';

enum PlayMode { sequential, shuffle, loop }

class MiniPlayerWidget extends StatefulWidget {
  const MiniPlayerWidget({super.key});

  @override
  State<MiniPlayerWidget> createState() => _MiniPlayerWidgetState();
}

class _MiniPlayerWidgetState extends State<MiniPlayerWidget> {
  PlayMode _playMode = PlayMode.sequential;

  void _togglePlayMode() {
    setState(() {
      switch (_playMode) {
        case PlayMode.sequential:
          _playMode = PlayMode.shuffle;
          Get.find<AudioPlayerHandler>().player.setShuffleModeEnabled(true);
          Get.find<AudioPlayerHandler>().player.setLoopMode(LoopMode.off);
          break;
        case PlayMode.shuffle:
          _playMode = PlayMode.loop;
          Get.find<AudioPlayerHandler>().player.setShuffleModeEnabled(false);
          Get.find<AudioPlayerHandler>().player.setLoopMode(LoopMode.one);
          break;
        case PlayMode.loop:
          _playMode = PlayMode.sequential;
          Get.find<AudioPlayerHandler>().player.setShuffleModeEnabled(false);
          Get.find<AudioPlayerHandler>().player.setLoopMode(LoopMode.off);
          break;
      }
    });
  }

  IconData _getPlayModeIcon() {
    switch (_playMode) {
      case PlayMode.sequential:
        return Icons.repeat;
      case PlayMode.shuffle:
        return Icons.shuffle;
      case PlayMode.loop:
        return Icons.repeat_one;
    }
  }

  String _getPlayModeTooltip() {
    switch (_playMode) {
      case PlayMode.sequential:
        return '顺序播放';
      case PlayMode.shuffle:
        return '随机播放';
      case PlayMode.loop:
        return '单曲循环';
    }
  }

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<MediaItem?>(
      stream: Get.find<AudioPlayerHandler>().mediaItem,
      builder: (context, mediaSnapshot) {
        final mediaItem = mediaSnapshot.data;
        if (mediaItem == null) return const SizedBox.shrink();

        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Row(
            children: [
              // 封面
              ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: SizedBox(
                  width: 48,
                  height: 48,
                  child: AlbumArtWidget(artUri: mediaItem.artUri),
                ),
              ),
              const SizedBox(width: 12),
              // 歌曲信息
              Expanded(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      mediaItem.title ?? '未知歌曲',
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w500,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    Text(
                      mediaItem.artist ?? '未知艺术家',
                      style: TextStyle(
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
              // 播放模式
              Tooltip(
                message: _getPlayModeTooltip(),
                child: IconButton(
                  icon: Icon(_getPlayModeIcon(), size: 20),
                  onPressed: _togglePlayMode,
                ),
              ),
              // 上一首
              IconButton(
                icon: const Icon(Icons.skip_previous, size: 28),
                onPressed: Get.find<AudioPlayerHandler>().skipToPrevious,
              ),
              // 播放/暂停
              StreamBuilder<bool>(
                stream: Get.find<AudioPlayerHandler>().playingState,
                builder: (context, playingSnapshot) {
                  final playing = playingSnapshot.data ?? false;
                  return IconButton(
                    icon: Icon(
                      playing ? Icons.pause_circle_filled : Icons.play_circle_filled,
                      size: 40,
                    ),
                    onPressed: playing
                        ? Get.find<AudioPlayerHandler>().pauseDirectly
                        : Get.find<AudioPlayerHandler>().playDirectly,
                  );
                },
              ),
              // 下一首
              IconButton(
                icon: const Icon(Icons.skip_next, size: 28),
                onPressed: Get.find<AudioPlayerHandler>().skipToNext,
              ),
            ],
          ),
        );
      },
    );
  }
}
