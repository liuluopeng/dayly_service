import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/wifi_api/files.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/pages/universal_video_player_page.dart';
import 'package:kongde/pages/image_viewer_page.dart';
import 'package:kongde/pages/pdf_viewer_page.dart';
import 'package:kongde/pages/epub_reader_page.dart';

class FileManagerPage extends StatefulWidget {
  const FileManagerPage({super.key});

  @override
  State<FileManagerPage> createState() => _FileManagerPageState();
}

class _FileManagerPageState extends State<FileManagerPage> {
  List<FileEntryForDart> _entries = [];
  String _currentPath = '/';
  bool _isLoading = false;
  String? _error;
  bool _isGrid = false;
  FileEntryForDart? _selectedFile;
  String? _textContent;

  @override
  void initState() {
    super.initState();
    _loadDirectory('/');
  }

  Future<void> _loadDirectory(String path) async {
    setState(() {
      _isLoading = true;
      _error = null;
      _selectedFile = null;
      _textContent = null;
    });
    try {
      final listing = await listFilesForDart(path: path);
      if (!mounted) return;
      setState(() {
        _entries = listing.entries;
        _currentPath = listing.path;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _entries = [];
      });
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  void _goBack() {
    final parts = _currentPath.split('/')..removeWhere((s) => s.isEmpty);
    if (parts.length <= 1) {
      _loadDirectory('/');
    } else {
      _loadDirectory('/${parts.sublist(0, parts.length - 1).join('/')}');
    }
  }

  void _onEntryTap(FileEntryForDart entry) {
    if (entry.isDir) {
      _loadDirectory(entry.path);
      return;
    }
    final type = _getFileType(entry.name);
    if (type == _FileType.video) {
      _openVideo(entry);
    } else if (type == _FileType.pdf) {
      _openPdf(entry);
    } else if (type == _FileType.epub) {
      _openEpub(entry);
    } else {
      setState(() {
        _selectedFile = entry;
        _textContent = null;
      });
      if (type == _FileType.text) {
        _loadText(entry.path);
      }
    }
  }

  Future<void> _loadText(String path) async {
    try {
      final url = await getFileUrlForDart(path: path);
      final request = await HttpClient().getUrl(Uri.parse(url));
      final response = await request.close();
      final body = await response.transform(utf8.decoder).join();
      if (mounted) setState(() => _textContent = body);
    } catch (e) {
      if (mounted) setState(() => _textContent = 'fileManager.cannotLoad'.trParams({'error': '$e'}));
    }
  }

  void _openVideo(FileEntryForDart entry) async {
    final url = await getFileUrlForDart(path: entry.path);
    if (!mounted) return;
    Get.to(() => UUUVideoPlayerPage(
      videoUrl: url,
      sourceType: VideoSourceType.network,
    ));
  }

  void _openImage(FileEntryForDart entry) async {
    final url = await getFileUrlForDart(path: entry.path);
    if (!mounted) return;
    Get.to(() => ImageViewerPage(imageUrl: url, fileName: entry.name));
  }

  void _openPdf(FileEntryForDart entry) async {
    final url = await getFileUrlForDart(path: entry.path);
    if (!mounted) return;
    Get.to(() => PdfViewerPage(url: url, fileName: entry.name));
  }

  void _openEpub(FileEntryForDart entry) async {
    final url = await getFileUrlForDart(path: entry.path);
    if (!mounted) return;
    Get.to(() => EpubReaderPage(url: url, fileName: entry.name));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'fileManager.title'.tr,
        actions: [
          IconButton(
            icon: Icon(_isGrid ? Icons.view_list : Icons.grid_view),
            onPressed: () => setState(() => _isGrid = !_isGrid),
            tooltip: _isGrid ? 'fileManager.listView'.tr : 'fileManager.gridView'.tr,
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
        children: [
          _buildBreadcrumb(),
          if (_error != null)
            Container(
              padding: const EdgeInsets.all(8),
              color: Colors.red.withOpacity(0.1),
              child: Text(_error!, style: const TextStyle(color: Colors.red)),
            ),
          Expanded(
            child: _isLoading
                ? const Center(child: CircularProgressIndicator())
                : _entries.isEmpty
                    ? Center(child: Text('fileManager.emptyDir'.tr, style: const TextStyle(color: Colors.grey)))
                    : _isGrid
                        ? _buildGrid()
                        : _buildList(),
          ),
          if (_selectedFile != null && !_selectedFile!.isDir) _buildPreview(),
        ],
      ),
      ),
    );
  }

  Widget _buildBreadcrumb() {
    final parts = _currentPath.split('/')..removeWhere((s) => s.isEmpty);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        children: [
          IconButton(
            icon: const Icon(Icons.arrow_back, size: 20),
            onPressed: _currentPath != '/' ? _goBack : null,
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  GestureDetector(
                    onTap: () => _loadDirectory('/'),
                    child: Text('/', style: TextStyle(
                      color: Theme.of(context).colorScheme.primary,
                      fontWeight: FontWeight.bold,
                    )),
                  ),
                  for (int i = 0; i < parts.length; i++) ...[
                    const Text(' / '),
                    GestureDetector(
                      onTap: () => _loadDirectory('/${parts.sublist(0, i + 1).join('/')}'),
                      child: Text(parts[i], style: TextStyle(
                        color: Theme.of(context).colorScheme.primary,
                      )),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildList() {
    return ListView.builder(
      itemCount: _entries.length,
      itemBuilder: (context, index) {
        final entry = _entries[index];
        final isSelected = _selectedFile?.path == entry.path;
        return ListTile(
          leading: Text(_getFileIcon(entry.name, entry.isDir), style: const TextStyle(fontSize: 20)),
          title: Text(entry.name, overflow: TextOverflow.ellipsis),
          subtitle: entry.isDir ? null : Text(_formatSize(entry.size), style: const TextStyle(fontSize: 12)),
          trailing: entry.lastModified != null
              ? Text(entry.lastModified!, style: const TextStyle(fontSize: 11, color: Colors.grey))
              : null,
          selected: isSelected,
          selectedTileColor: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3),
          onTap: () => _onEntryTap(entry),
        );
      },
    );
  }

  Widget _buildGrid() {
    return GridView.builder(
      padding: const EdgeInsets.all(12),
      gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 160,
        childAspectRatio: 0.7,
        crossAxisSpacing: 8,
        mainAxisSpacing: 8,
      ),
      itemCount: _entries.length,
      itemBuilder: (context, index) {
        final entry = _entries[index];
        final isSelected = _selectedFile?.path == entry.path;
        final fileType = _getFileType(entry.name);
        return GestureDetector(
          onTap: () => _onEntryTap(entry),
          child: Container(
            decoration: BoxDecoration(
              color: isSelected
                  ? Theme.of(context).colorScheme.primaryContainer.withOpacity(0.5)
                  : Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(8),
              border: isSelected
                  ? Border.all(color: Theme.of(context).colorScheme.primary, width: 2)
                  : null,
            ),
            child: Column(
              children: [
                Expanded(
                  child: ClipRRect(
                    borderRadius: const BorderRadius.vertical(top: Radius.circular(8)),
                    child: Container(
                      color: Theme.of(context).colorScheme.surfaceContainerLow,
                      child: fileType == _FileType.image
                          ? _GridThumb(path: entry.path)
                          : Center(child: Text(_getFileIcon(entry.name, entry.isDir), style: const TextStyle(fontSize: 32))),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(6),
                  child: Column(
                    children: [
                      Text(entry.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 12)),
                      if (!entry.isDir)
                        Text(_formatSize(entry.size), style: const TextStyle(fontSize: 10, color: Colors.grey)),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildPreview() {
    final type = _getFileType(_selectedFile!.name);
    return Container(
      height: 250,
      decoration: BoxDecoration(
        border: Border(top: BorderSide(color: Theme.of(context).dividerColor)),
      ),
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            child: Row(
              children: [
                Expanded(child: Text(_selectedFile!.name, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 13))),
                TextButton(
                  onPressed: () {
                    final type = _getFileType(_selectedFile!.name);
                    if (type == _FileType.image) {
                      _openImage(_selectedFile!);
                    } else {
                      _onEntryTap(_selectedFile!);
                    }
                  },
                  child: Text('fileManager.openFullscreen'.tr, style: const TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
          Expanded(
            child: _buildPreviewContent(type),
          ),
        ],
      ),
    );
  }

  Widget _buildPreviewContent(_FileType type) {
    switch (type) {
      case _FileType.image:
        return FutureBuilder<String>(
          future: getFileUrlForDart(path: _selectedFile!.path),
          builder: (context, snapshot) {
            if (!snapshot.hasData) return const Center(child: CircularProgressIndicator());
            return Image.network(snapshot.data!, fit: BoxFit.contain, errorBuilder: (_, __, ___) => Center(child: Text('fileManager.loadFailed'.tr)));
          },
        );
      case _FileType.text:
        if (_textContent == null) return const Center(child: CircularProgressIndicator());
        return Container(
          color: Colors.black87,
          padding: const EdgeInsets.all(8),
          child: SingleChildScrollView(
            child: Text(_textContent!, style: const TextStyle(fontFamily: 'monospace', fontSize: 12, color: Colors.white70)),
          ),
        );
      case _FileType.audio:
        return Center(child: Text('fileManager.audioPreview'.tr));
      case _FileType.pdf:
        return Center(child: Text('fileManager.pdfPreview'.tr));
      case _FileType.epub:
        return Center(child: Text('fileManager.epubPreview'.tr));
      default:
        return Center(child: Text('fileManager.noPreview'.tr, style: TextStyle(color: Colors.grey[600])));
    }
  }
}

enum _FileType { video, image, audio, pdf, epub, text, other }

_FileType _getFileType(String name) {
  final ext = name.split('.').last.toLowerCase();
  if (['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'm4v', 'webm'].contains(ext)) return _FileType.video;
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].contains(ext)) return _FileType.image;
  if (['mp3', 'flac', 'wav', 'ogg', 'aac', 'm4a'].contains(ext)) return _FileType.audio;
  if (['pdf'].contains(ext)) return _FileType.pdf;
  if (['epub'].contains(ext)) return _FileType.epub;
  if (['txt', 'md', 'log', 'nfo', 'srt', 'ass', 'json', 'xml', 'csv'].contains(ext)) return _FileType.text;
  return _FileType.other;
}

String _getFileIcon(String name, bool isDir) {
  if (isDir) return '📁';
  switch (_getFileType(name)) {
    case _FileType.video: return '🎬';
    case _FileType.image: return '🖼️';
    case _FileType.audio: return '🎵';
    case _FileType.pdf: return '📄';
    case _FileType.epub: return '📖';
    case _FileType.text: return '📝';
    default: return '📄';
  }
}

String _formatSize(BigInt size) {
  final s = size.toInt();
  if (s < 1024) return '$s B';
  if (s < 1024 * 1024) return '${(s / 1024).toStringAsFixed(1)} KB';
  if (s < 1024 * 1024 * 1024) return '${(s / (1024 * 1024)).toStringAsFixed(1)} MB';
  return '${(s / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
}

// Grid thumbnail with lazy loading
class _GridThumb extends StatefulWidget {
  final String path;
  const _GridThumb({required this.path});

  @override
  State<_GridThumb> createState() => _GridThumbState();
}

class _GridThumbState extends State<_GridThumb> {
  String? _url;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadUrl();
  }

  Future<void> _loadUrl() async {
    try {
      final url = await getFileUrlForDart(path: widget.path);
      if (mounted) setState(() { _url = url; _loading = false; });
    } catch (e) {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    if (_url == null) return const Center(child: Icon(Icons.broken_image, color: Colors.grey));
    return Image.network(_url!, fit: BoxFit.cover, errorBuilder: (_, __, ___) => const Center(child: Icon(Icons.broken_image, color: Colors.grey)));
  }
}
