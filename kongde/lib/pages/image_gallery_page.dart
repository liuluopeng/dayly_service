import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ImageGalleryPage extends StatefulWidget {
  const ImageGalleryPage({super.key});

  @override
  State<ImageGalleryPage> createState() => _ImageGalleryPageState();
}

class _ImageGalleryPageState extends State<ImageGalleryPage> {
  List<String> _folders = [];
  List<ImageItem> _images = [];
  bool _isLoading = true;
  bool _isScanning = false;
  String? _errorMessage;
  String? _selectedFolder;
  int _total = 0;
  int _currentPage = 1;
  bool _hasMore = true;
  bool _isLoadingMore = false;
  static const int _pageSize = 100;
  int _columnCount = 6;
  double _scaleStart = 1.0;
  final ScrollController _scrollController = ScrollController();
  double _scrollProgress = 0.0;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    _loadFolders();
  }

  @override
  void dispose() {
    _scrollController.removeListener(_onScroll);
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (!_scrollController.hasClients) return;
    final maxScroll = _scrollController.position.maxScrollExtent;
    if (maxScroll <= 0) return;
    final progress = (_scrollController.offset / maxScroll).clamp(0.0, 1.0);
    if (progress != _scrollProgress) {
      setState(() => _scrollProgress = progress);
    }
  }

  Future<void> _loadFolders() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
      _selectedFolder = null;
    });

    try {
      final headers = AppConfig.instance.getApiHeaders();
      final response = await http.get(
        Uri.parse(AppConfig.instance.imagesFoldersUrl),
        headers: headers,
      );

      if (response.statusCode == 200) {
        final data = json.decode(utf8.decode(response.bodyBytes));
        final folders = (data['data'] as List?)?.cast<String>() ?? [];
        setState(() {
          _folders = folders;
          _isLoading = false;
        });
      } else {
        setState(() {
          _errorMessage = 'common.loadFailedWith'.trParams({'error': '${response.statusCode}'});
          _isLoading = false;
        });
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'common.loadFailedWith'.trParams({'error': '$e'});
        _isLoading = false;
      });
    }
  }

  Future<void> _loadImages(String folder, {bool reset = true}) async {
    if (reset) {
      setState(() {
        _isLoading = true;
        _images = [];
        _currentPage = 1;
        _hasMore = true;
        _selectedFolder = folder;
      });
    } else {
      if (!_hasMore || _isLoadingMore) return;
      setState(() {
        _isLoadingMore = true;
        _currentPage++;
      });
    }

    try {
      final headers = AppConfig.instance.getApiHeaders();
      final uri = Uri.parse(
        '${AppConfig.instance.imagesListUrl}?folder=${Uri.encodeComponent(folder)}&page=$_currentPage&page_size=$_pageSize',
      );
      final response = await http.get(uri, headers: headers);

      if (response.statusCode == 200) {
        final data = json.decode(utf8.decode(response.bodyBytes));
        final paginated = data['data'];
        final imagesData = (paginated['data'] as List?)
                ?.map((item) => ImageItem.fromJson(item))
                .toList() ??
            [];
        final total = paginated['total'] ?? 0;
        final page = paginated['page'] ?? 1;
        final totalPages = paginated['total_pages'] ?? 1;

        setState(() {
          if (reset) {
            _images = imagesData;
          } else {
            _images.addAll(imagesData);
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
        if (!reset) _currentPage--;
      });
    }
  }

  Future<void> _scanImages() async {
    setState(() => _isScanning = true);
    try {
      final headers = AppConfig.instance.getApiHeaders();
      await http.post(
        Uri.parse(AppConfig.instance.imagesScanUrl),
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

  String _getThumbnailUrl(String imageId) {
    return '${AppConfig.instance.imagesThumbnailUrl}/$imageId';
  }

  String _getFolderName(String folder) {
    final parts = folder.split('/');
    return parts.last.isNotEmpty ? parts.last : folder;
  }

  String _formatSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }

  /// 跳转到指定索引的图片，自动加载缺失的页
  Future<void> _jumpToImage(int targetIndex) async {
    if (targetIndex < 0 || targetIndex >= _total) return;

    // 如果目标图片还没加载，先加载到那一页
    while (targetIndex >= _images.length && _hasMore && !_isLoadingMore) {
      await _loadImages(_selectedFolder!, reset: false);
    }

    if (!_scrollController.hasClients) return;

    // 根据网格布局计算滚动偏移
    final screenWidth = MediaQuery.of(context).size.width;
    final itemExtent = (screenWidth - 16) / _columnCount + 8; // item + spacing
    final row = targetIndex ~/ _columnCount;
    final targetOffset = (row * itemExtent).clamp(0.0, _scrollController.position.maxScrollExtent);

    _scrollController.jumpTo(targetOffset);
  }

  void _onImageTap(ImageItem image) {
    final index = _images.indexOf(image);
    Get.to(() => ImageViewerPage(
      images: _images,
      initialIndex: index >= 0 ? index : 0,
      headers: AppConfig.instance.getApiHeaders(),
      serverUrl: AppConfig.instance.serverUrl,
      totalInGallery: _total,
      onLoadMore: _hasMore ? () => _loadImages(_selectedFolder!, reset: false) : null,
    ));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: _selectedFolder != null
            ? _getFolderName(_selectedFolder!)
            : 'imageGallery.title'.tr,
        actions: [
          if (_selectedFolder != null) ...[
            LabeledIconButton(
              icon: Icons.arrow_back,
              label: 'common.back'.tr,
              onPressed: _loadFolders,
            ),
            Center(
              child: Text('imageGallery.columns'.trParams({'count': '$_columnCount'}), style: const TextStyle(fontSize: 13)),
            ),
          ],
          LabeledIconButton(
            icon: Icons.refresh,
            label: 'common.refresh'.tr,
            onPressed: _isScanning
                ? () {}
                : _selectedFolder != null
                    ? () => _loadImages(_selectedFolder!)
                    : _loadFolders,
          ),
          LabeledIconButton(
            icon: Icons.document_scanner,
            label: 'common.scan'.tr,
            onPressed: _isScanning ? () {} : _scanImages,
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
                        onPressed: _selectedFolder != null
                            ? () => _loadImages(_selectedFolder!)
                            : _loadFolders,
                        child: Text('common.retry'.tr),
                      ),
                    ],
                  ),
                )
              : _selectedFolder == null
                  ? _buildFolderList()
                  : _buildImageGrid(),
      ),
    );
  }

  Widget _buildFolderList() {
    if (_folders.isEmpty) {
      return Center(
        child: Text('imageGallery.noFolders'.tr),
      );
    }

    return ListView.builder(
      itemCount: _folders.length,
      itemBuilder: (context, index) {
        final folder = _folders[index];
        return ListTile(
          leading: const Icon(Icons.folder, color: Colors.amber),
          title: Text(_getFolderName(folder)),
          subtitle: Text(folder, maxLines: 1, overflow: TextOverflow.ellipsis),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _loadImages(folder),
        );
      },
    );
  }

  Widget _buildImageGrid() {
    if (_images.isEmpty) {
      return Center(child: Text('imageGallery.noImages'.tr));
    }

    return Row(
      children: [
        Expanded(
          child: GestureDetector(
            onScaleStart: (details) {
              _scaleStart = 1.0;
            },
            onScaleUpdate: (details) {
              if (details.scale == 1.0) return;
              final newCount = (2.0 / details.scale).round().clamp(2, 6);
              if (newCount != _columnCount) {
                setState(() => _columnCount = newCount);
              }
            },
            child: NotificationListener<ScrollNotification>(
              onNotification: (notification) {
                if (notification is ScrollEndNotification &&
                    notification.metrics.extentAfter < 200) {
                  _loadImages(_selectedFolder!, reset: false);
                }
                return false;
              },
              child: GridView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.all(8),
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: _columnCount,
                  crossAxisSpacing: 8,
                  mainAxisSpacing: 8,
                  childAspectRatio: 1,
                ),
                itemCount: _images.length + (_isLoadingMore ? 1 : 0),
                itemBuilder: (context, index) {
                  if (index >= _images.length) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  final image = _images[index];
                  final thumbUrl = _getThumbnailUrl(image.id);

                  return GestureDetector(
                    onTap: () => _onImageTap(image),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(0),
                      child: Image.network(
                        thumbUrl,
                        fit: BoxFit.cover,
                        headers: AppConfig.instance.getApiHeaders(),
                        errorBuilder: (context, error, stackTrace) {
                          return Container(
                            color: Colors.grey[200],
                            child: const Center(
                              child: Icon(Icons.broken_image,
                                  size: 40, color: Colors.grey),
                            ),
                          );
                        },
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        ),
        // 右侧竖向滚动进度条
        if (_total > 1)
          _GridScrollBar(
            progress: _scrollProgress,
            loadedCount: _images.length,
            totalCount: _total,
            onChanged: (value) {
              final maxScroll = _scrollController.position.maxScrollExtent;
              _scrollController.jumpTo(value * maxScroll);
            },
            onJumpToIndex: (index) => _jumpToImage(index),
          ),
      ],
    );
  }
}

class ImageItem {
  final String id;
  final String name;
  final String path;
  final String serveUrl;
  final String folderPath;
  final int size;
  final int? width;
  final int? height;
  final String? format;

  ImageItem({
    required this.id,
    required this.name,
    required this.path,
    required this.serveUrl,
    required this.folderPath,
    required this.size,
    this.width,
    this.height,
    this.format,
  });

  factory ImageItem.fromJson(Map<String, dynamic> json) {
    return ImageItem(
      id: json['id'] ?? '',
      name: json['name'] ?? '',
      path: json['path'] ?? '',
      serveUrl: json['serve_url'] ?? '',
      folderPath: json['folder_path'] ?? '',
      size: json['size'] ?? 0,
      width: json['width'],
      height: json['height'],
      format: json['format'],
    );
  }
}

class ImageViewerPage extends StatefulWidget {
  final List<ImageItem> images;
  final int initialIndex;
  final Map<String, String> headers;
  final String serverUrl;
  final int totalInGallery;
  final Future<void> Function()? onLoadMore;

  const ImageViewerPage({
    super.key,
    required this.images,
    required this.initialIndex,
    required this.headers,
    required this.serverUrl,
    required this.totalInGallery,
    this.onLoadMore,
  });

  @override
  State<ImageViewerPage> createState() => _ImageViewerPageState();
}

class _ImageViewerPageState extends State<ImageViewerPage> {
  late PageController _pageController;
  late int _currentIndex;
  bool _isFullscreen = false;

  @override
  void initState() {
    super.initState();
    _currentIndex = widget.initialIndex;
    _pageController = PageController(initialPage: _currentIndex);
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  String _getFullUrl(ImageItem image) {
    return image.serveUrl.startsWith('http')
        ? image.serveUrl
        : '${widget.serverUrl}${image.serveUrl}';
  }

  void _toggleFullscreen() {
    setState(() => _isFullscreen = !_isFullscreen);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: _isFullscreen
          ? null
          : AppBar(
              backgroundColor: Colors.black87,
              foregroundColor: Colors.white,
              title: Text(
                '${_currentIndex + 1} / ${widget.images.length}',
                style: const TextStyle(fontSize: 14),
              ),
            ),
      body: GestureDetector(
        onTap: _toggleFullscreen,
        child: Row(
          children: [
            Expanded(
              child: PageView.builder(
                controller: _pageController,
                itemCount: widget.images.length,
                onPageChanged: (index) {
                  setState(() => _currentIndex = index);
                  if (widget.onLoadMore != null && index >= widget.images.length - 5) {
                    widget.onLoadMore!();
                  }
                },
                itemBuilder: (context, index) {
                  final image = widget.images[index];
                  final url = _getFullUrl(image);
                  return InteractiveViewer(
                    minScale: 0.5,
                    maxScale: 5.0,
                    child: Image.network(
                      url,
                      headers: widget.headers,
                      fit: BoxFit.contain,
                      errorBuilder: (context, error, stackTrace) {
                        return const Center(
                          child: Icon(Icons.broken_image,
                              size: 64, color: Colors.grey),
                        );
                      },
                    ),
                  );
                },
              ),
            ),
            if (!_isFullscreen && widget.totalInGallery > 1)
              _VerticalSlider(
                current: _currentIndex,
                total: widget.totalInGallery,
                loadedCount: widget.images.length,
                onChanged: (index) {
                  if (index < widget.images.length) {
                    _pageController.jumpToPage(index);
                  }
                },
              ),
          ],
        ),
      ),
    );
  }
}

class _VerticalSlider extends StatelessWidget {
  final int current;
  final int total;
  final int loadedCount;
  final ValueChanged<int> onChanged;

  const _VerticalSlider({
    required this.current,
    required this.total,
    required this.loadedCount,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onVerticalDragUpdate: (details) {
        final box = context.findRenderObject() as RenderBox;
        final dy = details.localPosition.dy.clamp(0.0, box.size.height);
        final ratio = dy / box.size.height;
        // 拖拽基于已加载范围
        final index = (ratio * (loadedCount - 1)).round().clamp(0, loadedCount - 1);
        if (index != current) onChanged(index);
      },
      child: Container(
        width: 44,
        color: Colors.black54,
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                '${current + 1}/$total',
                style: const TextStyle(color: Colors.white70, fontSize: 10),
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final trackHeight = constraints.maxHeight - 8;
                  final loadedFraction = loadedCount / total;
                  final currentFraction = total > 1 ? current / (total - 1) : 0.0;
                  return Stack(
                    alignment: Alignment.topCenter,
                    children: [
                      // 全量轨道（暗色）
                      Positioned(
                        top: 4,
                        child: Container(
                          width: 3,
                          height: trackHeight,
                          decoration: BoxDecoration(
                            color: Colors.white24,
                            borderRadius: BorderRadius.circular(1.5),
                          ),
                        ),
                      ),
                      // 已加载部分（亮色）
                      Positioned(
                        top: 4,
                        child: Container(
                          width: 3,
                          height: trackHeight * loadedFraction,
                          decoration: BoxDecoration(
                            color: Colors.white54,
                            borderRadius: BorderRadius.circular(1.5),
                          ),
                        ),
                      ),
                      // 滑块
                      Positioned(
                        top: 4 + currentFraction * (trackHeight - 12),
                        child: Container(
                          width: 22,
                          height: 12,
                          decoration: BoxDecoration(
                            color: Colors.white,
                            borderRadius: BorderRadius.circular(3),
                          ),
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _GridScrollBar extends StatelessWidget {
  final double progress;
  final int loadedCount;
  final int totalCount;
  final ValueChanged<double> onChanged;
  final ValueChanged<int> onJumpToIndex;

  const _GridScrollBar({
    required this.progress,
    required this.loadedCount,
    required this.totalCount,
    required this.onChanged,
    required this.onJumpToIndex,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onVerticalDragUpdate: (details) {
        final box = context.findRenderObject() as RenderBox;
        final dy = details.localPosition.dy.clamp(0.0, box.size.height);
        final ratio = dy / box.size.height;
        onChanged(ratio);
      },
      onVerticalDragEnd: (details) {
        // 拖拽结束时，根据当前位置计算目标图片索引
        final box = context.findRenderObject() as RenderBox;
        // 使用最后一次 onChanged 的 progress 值来计算
        final targetIndex = (progress * (totalCount - 1)).round().clamp(0, totalCount - 1);
        onJumpToIndex(targetIndex);
      },
      child: Container(
        width: 44,
        color: Colors.black54,
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                '${(progress * totalCount).round()}/$totalCount',
                style: const TextStyle(color: Colors.white70, fontSize: 10),
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final trackHeight = constraints.maxHeight - 8;
                  final loadedFraction = loadedCount / totalCount;
                  // 滑块位置基于当前 progress（可超出已加载范围）
                  final thumbTop = 4 + progress * (trackHeight - 12);
                  return Stack(
                    alignment: Alignment.topCenter,
                    children: [
                      // 全量轨道（暗色）
                      Positioned(
                        top: 4,
                        child: Container(
                          width: 3,
                          height: trackHeight,
                          decoration: BoxDecoration(
                            color: Colors.white24,
                            borderRadius: BorderRadius.circular(1.5),
                          ),
                        ),
                      ),
                      // 已加载部分（亮色）
                      Positioned(
                        top: 4,
                        child: Container(
                          width: 3,
                          height: trackHeight * loadedFraction,
                          decoration: BoxDecoration(
                            color: Colors.white54,
                            borderRadius: BorderRadius.circular(1.5),
                          ),
                        ),
                      ),
                      // 滑块 — 可在全轨道范围内拖动
                      Positioned(
                        top: thumbTop,
                        child: Container(
                          width: 22,
                          height: 12,
                          decoration: BoxDecoration(
                            color: Colors.white,
                            borderRadius: BorderRadius.circular(3),
                          ),
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
