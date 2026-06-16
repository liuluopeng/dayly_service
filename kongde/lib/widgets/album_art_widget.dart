import 'package:flutter/material.dart';
import 'package:kongde/main.dart';
import 'dart:io';

class AlbumArtWidget extends StatefulWidget {
  final Uri? artUri;

  const AlbumArtWidget({super.key, this.artUri});

  @override
  State<AlbumArtWidget> createState() => _AlbumArtWidgetState();
}

class _AlbumArtWidgetState extends State<AlbumArtWidget> {
  final Map<String, Widget> _imageCache = {};
  Uri? _lastArtUri;

  @override
  Widget build(BuildContext context) {
    final artUri = widget.artUri;

    if (artUri != _lastArtUri) {
      _lastArtUri = artUri;
      logger.i('artUri: $artUri');
    }

    if (artUri != null) {
      final cacheKey = artUri.toString();

      if (!_imageCache.containsKey(cacheKey)) {
        Widget imageWidget;

        if (artUri.isScheme('data')) {
          imageWidget = Image.memory(
            UriData.fromUri(artUri).contentAsBytes(),
            width: 200,
            height: 200,
            fit: BoxFit.cover,
            gaplessPlayback: true,
            errorBuilder: (context, error, stackTrace) {
              logger.i('Image.memory error: $error');
              return _buildPlaceholder();
            },
          );
        } else if (artUri.isScheme('file')) {
          imageWidget = Image.file(
            File(artUri.toFilePath()),
            width: 200,
            height: 200,
            fit: BoxFit.cover,
            gaplessPlayback: true,
            errorBuilder: (context, error, stackTrace) {
              logger.i('Image.file error: $error');
              return _buildPlaceholder();
            },
          );
        } else {
          imageWidget = Image.network(
            artUri.toString(),
            width: 200,
            height: 200,
            fit: BoxFit.cover,
            gaplessPlayback: true,
            errorBuilder: (context, error, stackTrace) {
              logger.i('Image.network error: $error');
              return _buildPlaceholder();
            },
          );
        }

        _imageCache[cacheKey] = ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: imageWidget,
        );
      }

      return _imageCache[cacheKey]!;
    } else {
      return _buildPlaceholder();
    }
  }

  Widget _buildPlaceholder() {
    return Container(
      width: 200,
      height: 200,
      decoration: BoxDecoration(
        color: Colors.grey[300],
        borderRadius: BorderRadius.circular(12),
      ),
      child: const Icon(Icons.music_note, size: 80, color: Colors.blue),
    );
  }
}
