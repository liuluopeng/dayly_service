import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;
import 'package:kongde/pages/universal_video_player_page.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class VideoLibraryPage extends StatefulWidget {
  const VideoLibraryPage({super.key});

  @override
  State<VideoLibraryPage> createState() => _VideoLibraryPageState();
}

class _VideoLibraryPageState extends State<VideoLibraryPage> {
  List<VideoItem> _videos = [];
  bool _isLoading = true;
  bool _isScanning = false;
  String? _errorMessage;
  int _total = 0;
  int _currentPage = 1;
  bool _hasMore = true;
  bool _isLoadingMore = false;
  static const int _pageSize = 50;

  @override
  void initState() {
    super.initState();
    _loadVideos(reset: true);
  }

  Future<void> _loadVideos({bool reset = true}) async {
    if (reset) {
      setState(() {
        _isLoading = true;
        _errorMessage = null;
        _videos = [];
        _currentPage = 1;
        _hasMore = true;
      });
    } else {
      if (!_hasMore || _isLoadingMore) return;
      setState(() => _isLoadingMore = true);
    }

    try {
      final headers = AppConfig.instance.getApiHeaders();
      final uri = Uri.parse(
        '${AppConfig.instance.videosListUrl}?page=$_currentPage&page_size=$_pageSize',
      );
      final response = await http.get(uri, headers: headers);

      if (response.statusCode == 200) {
        final data = json.decode(utf8.decode(response.bodyBytes));
        final paginated = data['data'];
        final videosData = (paginated['data'] as List?)
                ?.map((item) => VideoItem.fromJson(item))
                .toList() ??
            [];
        final total = paginated['total'] ?? 0;
        final page = paginated['page'] ?? 1;
        final totalPages = paginated['total_pages'] ?? 1;

        setState(() {
          if (reset) {
            _videos = videosData;
          } else {
            _videos.addAll(videosData);
          }
          _total = total;
          _currentPage = page;
          _hasMore = page < totalPages;
          _isLoading = false;
          _isLoadingMore = false;
        });
      } else {
        setState(() {
          _errorMessage = 'common.loadFailedWith'.trParams({'error': '${response.statusCode}'});
          _isLoading = false;
          _isLoadingMore = false;
        });
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'common.loadFailedWith'.trParams({'error': '$e'});
        _isLoading = false;
        _isLoadingMore = false;
      });
    }
  }

  Future<void> _scanVideos() async {
    setState(() => _isScanning = true);
    try {
      final headers = AppConfig.instance.getApiHeaders();
      await http.post(
        Uri.parse(AppConfig.instance.videosScanUrl),
        headers: headers,
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('imageGallery.scanStarted'.tr)),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('imageGallery.scanFailed'.trParams({'error': '$e'}))),
        );
      }
    } finally {
      setState(() => _isScanning = false);
    }
  }

  String _formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(2)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(2)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
  }

  String _getPreviewUrl(String videoId) {
    return '${AppConfig.instance.serverUrl}/api/videos/preview/$videoId';
  }

  String _formatDuration(int ms) {
    final seconds = ms ~/ 1000;
    final h = seconds ~/ 3600;
    final m = (seconds % 3600) ~/ 60;
    final s = seconds % 60;
    if (h > 0) {
      return '$h:${m.toString().padLeft(2, '0')}:${s.toString().padLeft(2, '0')}';
    }
    return '$m:${s.toString().padLeft(2, '0')}';
  }

  void _onVideoTap(VideoItem video) {
    final serveUrl = video.serveUrl;
    final fullUrl = serveUrl.startsWith('http')
        ? serveUrl
        : '${AppConfig.instance.serverUrl}$serveUrl';
    Get.to(() => UUUVideoPlayerPage.network(
      fullUrl,
      headers: AppConfig.instance.getApiHeaders(),
    ));
  }

  Widget _buildVideoCard(VideoItem video) {
    final previewUrl = _getPreviewUrl(video.id);
    final headers = AppConfig.instance.getApiHeaders();

    return Card(
      clipBehavior: Clip.antiAlias,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      child: InkWell(
        onTap: () => _onVideoTap(video),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: Stack(
                fit: StackFit.expand,
                children: [
                  // Preview image
                  Image.network(
                    previewUrl,
                    fit: BoxFit.cover,
                    headers: headers,
                    errorBuilder: (context, error, stackTrace) {
                      return Container(
                        color: Colors.grey[900],
                        child: const Center(
                          child: Icon(Icons.movie,
                              size: 48, color: Colors.white38),
                        ),
                      );
                    },
                  ),
                  // Play overlay
                  Positioned.fill(
                    child: Container(
                      color: Colors.black26,
                      child: const Center(
                        child: Icon(Icons.play_circle_fill,
                            size: 48, color: Colors.white70),
                      ),
                    ),
                  ),
                  // Duration badge
                  if (video.durationMs != null && video.durationMs! > 0)
                    Positioned(
                      bottom: 6,
                      right: 6,
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: Colors.black.withOpacity(0.75),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          _formatDuration(video.durationMs!),
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 12,
                            fontFeatures: [FontFeature.tabularFigures()],
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
            // Info section
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    video.name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                        fontSize: 14, fontWeight: FontWeight.w500),
                  ),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      if (video.format != null) ...[
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 1),
                          decoration: BoxDecoration(
                            color: Colors.blue[50],
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            video.format!.toUpperCase(),
                            style: TextStyle(
                                fontSize: 11, color: Colors.blue[700]),
                          ),
                        ),
                        const SizedBox(width: 6),
                      ],
                      Text(
                        _formatFileSize(video.size),
                        style:
                            const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'videoLibrary.title'.tr,
        actions: [
          IconButton(
            icon: _isScanning
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.refresh),
            onPressed: _isScanning ? null : () => _loadVideos(reset: true),
          ),
          IconButton(
            icon: const Icon(Icons.document_scanner),
            onPressed: _isScanning ? null : _scanVideos,
            tooltip: 'videoLibrary.scanVideo'.tr,
          ),
        ],
      ),
      body: SafeArea(
        child: _isLoading
            ? const Center(child: CircularProgressIndicator())
            : _errorMessage != null
                ? Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.error_outline,
                            size: 48, color: Colors.red),
                        const SizedBox(height: 16),
                        Text(_errorMessage!),
                        const SizedBox(height: 16),
                        ElevatedButton(
                          onPressed: () => _loadVideos(reset: true),
                          child: Text('common.retry'.tr),
                        ),
                      ],
                    ),
                  )
                : NotificationListener<ScrollNotification>(
                    onNotification: (notification) {
                      if (notification is ScrollEndNotification &&
                          notification.metrics.extentAfter < 300) {
                        _loadVideos(reset: false);
                      }
                      return false;
                    },
                    child: GridView.builder(
                      padding: const EdgeInsets.all(8),
                      gridDelegate:
                          const SliverGridDelegateWithMaxCrossAxisExtent(
                        maxCrossAxisExtent: 280,
                        childAspectRatio: 0.68,
                        crossAxisSpacing: 8,
                        mainAxisSpacing: 8,
                      ),
                      itemCount: _videos.length + (_isLoadingMore ? 1 : 0),
                      itemBuilder: (context, index) {
                        if (index >= _videos.length) {
                          return const Center(
                              child: CircularProgressIndicator());
                        }
                        final video = _videos[index];
                        return _buildVideoCard(video);
                      },
                    ),
                  ),
      ),
    );
  }
}

class VideoItem {
  final String id;
  final String name;
  final String path;
  final String serveUrl;
  final String folderPath;
  final int size;
  final int? durationMs;
  final String? format;

  VideoItem({
    required this.id,
    required this.name,
    required this.path,
    required this.serveUrl,
    required this.folderPath,
    required this.size,
    this.durationMs,
    this.format,
  });

  factory VideoItem.fromJson(Map<String, dynamic> json) {
    return VideoItem(
      id: json['id'] ?? '',
      name: json['name'] ?? '',
      path: json['path'] ?? '',
      serveUrl: json['serve_url'] ?? '',
      folderPath: json['folder_path'] ?? '',
      size: json['size'] ?? 0,
      durationMs: json['duration_ms'],
      format: json['format'],
    );
  }
}
