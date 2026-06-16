import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:audio_service/audio_service.dart';

class SongInfoWidget extends StatelessWidget {
  final MediaItem? mediaItem;
  final Map<String, Color?> mainColors;

  const SongInfoWidget({super.key, this.mediaItem, required this.mainColors});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          mediaItem?.title ?? 'songInfo.noAudio'.tr,
          style: TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.bold,
            color: mainColors['secondaryColor'] ?? Colors.white,
          ),
          textAlign: TextAlign.center,
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
        ),
        const SizedBox(height: 16),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            Flexible(child: _buildArtistLink()),
            const SizedBox(width: 8),
            Text(
              '-',
              style: TextStyle(
                fontSize: 14,
                color:
                    mainColors['secondaryColor']?.withValues(alpha: 0.7) ??
                    Colors.white70,
              ),
            ),
            const SizedBox(width: 8),
            Flexible(child: _buildAlbumLink()),
          ],
        ),
      ],
    );
  }

  Widget _buildArtistLink() {
    final artist = mediaItem?.artist;
    final shouldShow = artist != null && artist.isNotEmpty && artist != 'music.unknownArtist'.tr;

    return InkWell(
      onTap: null,
      child: Text(
        mediaItem?.artist ?? 'music.unknownArtist'.tr,
        style: TextStyle(
          fontSize: 14,
          color:
              mainColors['secondaryColor']?.withValues(alpha: 0.7) ??
              Colors.white70,
          decoration: shouldShow ? TextDecoration.underline : null,
        ),
        textAlign: TextAlign.center,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }

  Widget _buildAlbumLink() {
    final album = mediaItem?.album;
    final shouldShow = album != null && album.isNotEmpty && album != 'music.unknownAlbum'.tr;

    return InkWell(
      onTap: null,
      child: Text(
        mediaItem?.album ?? 'music.unknownAlbum'.tr,
        style: TextStyle(
          fontSize: 14,
          color:
              mainColors['secondaryColor']?.withValues(alpha: 0.7) ??
              Colors.white70,
          decoration: shouldShow ? TextDecoration.underline : null,
        ),
        textAlign: TextAlign.center,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }
}
