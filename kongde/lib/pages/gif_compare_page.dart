import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/utils/gif_decode.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class GifComparePage extends StatefulWidget {
  const GifComparePage({super.key});

  @override
  State<GifComparePage> createState() => _GifComparePageState();
}

class _GifComparePageState extends State<GifComparePage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  String? _filePath;
  int _fileSizeMb = 0;

  // Dart results
  int _dartLoadMs = 0;
  bool _dartLoading = false;

  // Rust results
  int _rustLoadMs = 0;
  bool _rustLoading = false;
  List<_RustFrame> _rustFrames = [];
  int _currentRustFrame = 0;
  Stopwatch? _rustStopwatch;
  StreamSubscription? _rustSubscription;

  static const int _maxFrames = 30;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _rustSubscription?.cancel();
    super.dispose();
  }

  Future<void> _pickFile() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['gif'],
    );
    if (result == null || result.files.isEmpty) return;

    final path = result.files.first.path;
    if (path == null) return;

    final file = File(path);
    final sizeBytes = await file.length();

    _rustSubscription?.cancel();
    setState(() {
      _filePath = path;
      _fileSizeMb = (sizeBytes / (1024 * 1024)).round();
      _rustFrames = [];
      _currentRustFrame = 0;
      _rustLoadMs = 0;
    });
  }

  Future<void> _testDart() async {
    if (_filePath == null) return;
    setState(() => _dartLoading = true);

    final sw = Stopwatch()..start();
    await File(_filePath!).readAsBytes();
    sw.stop();

    if (mounted) {
      setState(() {
        _dartLoadMs = sw.elapsedMilliseconds;
        _dartLoading = false;
      });
    }
  }

  void _testRust() {
    if (_filePath == null) return;
    _rustSubscription?.cancel();

    setState(() {
      _rustLoading = true;
      _rustFrames = [];
      _currentRustFrame = 0;
      _rustLoadMs = 0;
    });

    _rustStopwatch = Stopwatch()..start();

    _rustSubscription = decodeGifStream(
      path: _filePath!,
      maxFrames: _maxFrames,
    ).listen(
      (frame) async {
        // Convert RGBA to ui.Image
        final completer = Completer<ui.Image>();
        ui.decodeImageFromPixels(
          frame.rgba,
          frame.width,
          frame.height,
          ui.PixelFormat.rgba8888,
          (image) => completer.complete(image),
        );
        final image = await completer.future;

        if (!mounted) return;
        setState(() {
          _rustFrames.add(_RustFrame(
            image: image,
            width: frame.width,
            height: frame.height,
            delayMs: frame.delayMs,
          ));
        });
      },
      onDone: () {
        if (!mounted) return;
        _rustStopwatch?.stop();
        setState(() {
          _rustLoadMs = _rustStopwatch?.elapsedMilliseconds ?? 0;
          _rustLoading = false;
        });
      },
      onError: (_) {
        if (!mounted) return;
        setState(() => _rustLoading = false);
      },
    );
  }

  void _playRustAnimation() {
    if (_rustFrames.isEmpty) return;
    _currentRustFrame = 0;
    _advanceFrame();
  }

  void _advanceFrame() {
    if (!mounted || _rustFrames.isEmpty) return;
    final delay = _rustFrames[_currentRustFrame].delayMs;
    Future.delayed(Duration(milliseconds: delay > 0 ? delay : 100), () {
      if (!mounted) return;
      setState(() {
        _currentRustFrame = (_currentRustFrame + 1) % _rustFrames.length;
      });
      _advanceFrame();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'GIF Rust Compare'),
      body: SafeArea(
        child: Column(
          children: [
            // File selector
            Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      _filePath != null
                          ? '${_filePath!.split('/').last} ($_fileSizeMb MB)'
                          : 'No file selected',
                      style: const TextStyle(fontSize: 14),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton.icon(
                    onPressed: _pickFile,
                    icon: const Icon(Icons.folder_open, size: 18),
                    label: const Text('Select GIF'),
                  ),
                ],
              ),
            ),
            // Tabs
            TabBar(
              controller: _tabController,
              tabs: const [
                Tab(text: 'Flutter Image', icon: Icon(Icons.image)),
                Tab(text: 'Rust Decode', icon: Icon(Icons.code)),
              ],
            ),
            // Tab content
            Expanded(
              child: TabBarView(
                controller: _tabController,
                children: [
                  _buildDartTab(),
                  _buildRustTab(),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDartTab() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          ElevatedButton.icon(
            onPressed: _filePath != null && !_dartLoading ? _testDart : null,
            icon: const Icon(Icons.play_arrow),
            label: const Text('Test Flutter Image'),
          ),
          if (_dartLoading) const Padding(
            padding: EdgeInsets.all(16),
            child: CircularProgressIndicator(),
          ),
          if (_dartLoadMs > 0)
            _buildStatCard('Load time', '${_dartLoadMs}ms'),
          const SizedBox(height: 12),
          Expanded(
            child: _filePath != null && _dartLoadMs > 0
                ? Container(
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.grey.shade300),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(8),
                      child: Image.file(
                        File(_filePath!),
                        fit: BoxFit.contain,
                        gaplessPlayback: true,
                      ),
                    ),
                  )
                : const Center(child: Text('Press test to load GIF')),
          ),
        ],
      ),
    );
  }

  Widget _buildRustTab() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton.icon(
                onPressed: _filePath != null && !_rustLoading ? _testRust : null,
                icon: const Icon(Icons.play_arrow),
                label: const Text('Test Rust Decode'),
              ),
              if (_rustFrames.isNotEmpty) ...[
                const SizedBox(width: 12),
                ElevatedButton.icon(
                  onPressed: _playRustAnimation,
                  icon: const Icon(Icons.play_circle),
                  label: const Text('Play'),
                ),
              ],
            ],
          ),
          if (_rustLoading) Padding(
            padding: const EdgeInsets.all(8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2)),
                const SizedBox(width: 8),
                Text('Decoding... ${_rustFrames.length} frames', style: const TextStyle(fontSize: 14)),
              ],
            ),
          ),
          if (_rustLoadMs > 0) ...[
            _buildStatCard('Total time', '${_rustLoadMs}ms'),
            _buildStatCard('Frames', '${_rustFrames.length}'),
          ],
          const SizedBox(height: 12),
          Expanded(
            child: _rustFrames.isNotEmpty
                ? Container(
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.grey.shade300),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(8),
                      child: RawImage(
                        image: _rustFrames[_currentRustFrame].image,
                        fit: BoxFit.contain,
                      ),
                    ),
                  )
                : const Center(child: Text('Press test to decode GIF frames')),
          ),
        ],
      ),
    );
  }

  Widget _buildStatCard(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(label, style: const TextStyle(fontWeight: FontWeight.w500)),
          const SizedBox(width: 8),
          Text(value, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
        ],
      ),
    );
  }
}

class _RustFrame {
  final ui.Image image;
  final int width;
  final int height;
  final int delayMs;

  _RustFrame({
    required this.image,
    required this.width,
    required this.height,
    required this.delayMs,
  });
}
