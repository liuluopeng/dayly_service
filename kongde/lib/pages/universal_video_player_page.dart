import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:media_kit/media_kit.dart';
import 'package:media_kit_video/media_kit_video.dart';

enum VideoSourceType { network, asset, memory }

class UUUVideoPlayerPage extends StatefulWidget {
  final String videoUrl;
  final VideoSourceType sourceType;
  final Uint8List? videoBytes;
  final Map<String, String>? headers;

  const UUUVideoPlayerPage({
    super.key,
    required this.videoUrl,
    required this.sourceType,
    this.videoBytes,
    this.headers,
  });

  factory UUUVideoPlayerPage.network(
    String url, {
    Map<String, String>? headers,
  }) {
    return UUUVideoPlayerPage(
      videoUrl: url,
      sourceType: VideoSourceType.network,
      headers: headers,
    );
  }

  factory UUUVideoPlayerPage.asset(String assetPath) {
    return UUUVideoPlayerPage(
      videoUrl: assetPath,
      sourceType: VideoSourceType.asset,
    );
  }

  factory UUUVideoPlayerPage.memory(Uint8List bytes) {
    return UUUVideoPlayerPage(
      videoUrl: '',
      sourceType: VideoSourceType.memory,
      videoBytes: bytes,
    );
  }

  @override
  State<UUUVideoPlayerPage> createState() => _UUUVideoPlayerPageState();
}

class _UUUVideoPlayerPageState extends State<UUUVideoPlayerPage> {
  late final Player _player;
  late final VideoController _controller;
  bool _isLoading = true;
  String? _errorMessage;
  double _currentSpeed = 1.0;
  static const _speeds = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
  StreamSubscription? _errorSub;
  StreamSubscription? _videoParamsSub;
  StreamSubscription? _playingSub;
  String? _lastError;

  // Swipe-to-seek state
  bool _isSeeking = false;
  double _seekStartX = 0;
  Duration _seekBasePosition = Duration.zero;
  Duration _seekOffset = Duration.zero;

  // Long-press 2x speed state
  bool _isLongPressSpeed = false;

  // Fullscreen state
  bool _isFullscreen = false;

  @override
  void initState() {
    super.initState();
    _player = Player();
    _controller = VideoController(_player);

    _errorSub = _player.stream.log.listen((event) {
      debugPrint('media_kit [${event.level}]: ${event.text}');
      if (event.level == 'error') {
        _lastError = event.text;
      }
    });

    _videoParamsSub = _player.stream.videoParams.listen((event) {
      debugPrint('media_kit video params: ${event.dw}x${event.dh}');
    });

    _playingSub = _player.stream.playing.listen((event) {
      debugPrint('media_kit playing: $event');
    });

    _initPlayer();
  }

  Future<void> _initPlayer() async {
    setState(() {
      _errorMessage = null;
      _isLoading = true;
    });

    try {
      _lastError = null;

      switch (widget.sourceType) {
        case VideoSourceType.network:
          await _player
              .open(
                Media(widget.videoUrl, httpHeaders: widget.headers),
                play: true,
              )
              .timeout(const Duration(seconds: 30));
          break;
        case VideoSourceType.asset:
          await _player
              .open(Media('asset://${widget.videoUrl}'), play: true)
              .timeout(const Duration(seconds: 30));
          break;
        case VideoSourceType.memory:
          throw Exception('videoPlayer.memoryNotSupported'.tr);
      }

      // 等待视频流初始化或错误到达
      await Future.delayed(const Duration(seconds: 3));

      if (!mounted) return;

      // 检查视频是否真正加载成功
      final width = _player.state.width;
      final height = _player.state.height;
      if (width == null || width == 0 || height == null || height == 0) {
        setState(() {
          _errorMessage = _lastError ?? 'videoPlayer.loadFailed'.tr;
          _isLoading = false;
        });
        return;
      }

      setState(() {
        _isLoading = false;
      });
    } catch (e) {
      if (mounted) {
        setState(() {
          _errorMessage = 'videoPlayer.playFailed'.trParams({'error': '$e'});
          _isLoading = false;
        });
      }
    }
  }

  void _setSpeed(double speed) {
    setState(() => _currentSpeed = speed);
    _player.setRate(speed);
  }

  void _onSeekStart(DragStartDetails details) {
    _seekStartX = details.globalPosition.dx;
    _seekBasePosition = _player.state.position;
    _seekOffset = Duration.zero;
    setState(() => _isSeeking = true);
  }

  void _onSeekUpdate(DragUpdateDetails details) {
    final dx = details.globalPosition.dx - _seekStartX;
    final screenWidth = MediaQuery.of(context).size.width;
    final totalSeconds = _player.state.duration.inSeconds;

    // Drag ~2/3 screen = seek full duration
    final ratio = (dx * 1.5 / screenWidth).clamp(-1.0, 1.0);
    final offsetSeconds = (ratio * totalSeconds).round();

    setState(() {
      _seekOffset = Duration(seconds: offsetSeconds);
    });
  }

  void _onSeekEnd(DragEndDetails details) {
    final target = _seekBasePosition + _seekOffset;
    final clamped = target.isNegative ? Duration.zero : target;
    _player.seek(clamped);
    setState(() {
      _isSeeking = false;
      _seekOffset = Duration.zero;
    });
  }

  void _onLongPressStart(LongPressStartDetails details) {
    _isLongPressSpeed = true;
    _player.setRate(2.0);
    setState(() {});
  }

  void _onLongPressEnd(LongPressEndDetails details) {
    _isLongPressSpeed = false;
    _player.setRate(_currentSpeed);
    setState(() {});
  }

  void _toggleFullscreen() {
    setState(() => _isFullscreen = !_isFullscreen);
    if (_isFullscreen) {
      SystemChrome.setPreferredOrientations([
        DeviceOrientation.landscapeLeft,
        DeviceOrientation.landscapeRight,
      ]);
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
    } else {
      SystemChrome.setPreferredOrientations(DeviceOrientation.values);
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    }
  }

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60);
    final s = d.inSeconds.remainder(60);
    if (h > 0) return '${h}:${m.toString().padLeft(2, '0')}:${s.toString().padLeft(2, '0')}';
    return '${m}:${s.toString().padLeft(2, '0')}';
  }

  @override
  void dispose() {
    _errorSub?.cancel();
    _videoParamsSub?.cancel();
    _playingSub?.cancel();
    _player.dispose();
    SystemChrome.setPreferredOrientations(DeviceOrientation.values);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final videoWidget = MaterialVideoControlsTheme(
      normal: MaterialVideoControlsThemeData(
        seekBarThumbColor: Colors.blue,
        seekBarPositionColor: Colors.blue,
      ),
      fullscreen: MaterialVideoControlsThemeData(
        seekBarThumbColor: Colors.blue,
        seekBarPositionColor: Colors.blue,
      ),
      child: Video(controller: _controller),
    );

    final gestureLayer = Positioned.fill(
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onHorizontalDragStart: _onSeekStart,
        onHorizontalDragUpdate: _onSeekUpdate,
        onHorizontalDragEnd: _onSeekEnd,
        onLongPressStart: _onLongPressStart,
        onLongPressEnd: _onLongPressEnd,
      ),
    );

    final overlays = <Widget>[
      if (_isSeeking)
        Positioned.fill(
          child: Center(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
              decoration: BoxDecoration(
                color: Colors.black87,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    _seekOffset.isNegative ? Icons.fast_rewind : Icons.fast_forward,
                    color: Colors.white,
                    size: 32,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    '${_seekOffset.isNegative ? '' : '+'}${_formatDuration(_seekOffset)}',
                    style: const TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    '${_formatDuration(_seekBasePosition + _seekOffset)}',
                    style: const TextStyle(color: Colors.white70, fontSize: 14),
                  ),
                ],
              ),
            ),
          ),
        ),
      if (_isLongPressSpeed)
        Positioned(
          top: 60,
          left: 0,
          right: 0,
          child: Center(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.orange.withOpacity(0.9),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Text(
                '2X',
                style: TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold),
              ),
            ),
          ),
        ),
      // Fullscreen toggle button
      Positioned(
        bottom: 8,
        right: 8,
        child: GestureDetector(
          onTap: _toggleFullscreen,
          child: Container(
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              color: Colors.black54,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Icon(
              _isFullscreen ? Icons.fullscreen_exit : Icons.fullscreen,
              color: Colors.white,
              size: 24,
            ),
          ),
        ),
      ),
    ];

    final speedButtons = Positioned(
      top: 8,
      right: 8,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
        decoration: BoxDecoration(
          color: Colors.black54,
          borderRadius: BorderRadius.circular(6),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: _speeds.map((s) {
            final isActive = _currentSpeed == s;
            return GestureDetector(
              onTap: () => _setSpeed(s),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                margin: const EdgeInsets.symmetric(horizontal: 2),
                decoration: BoxDecoration(
                  color: isActive ? Colors.blue : Colors.transparent,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '${s}x',
                  style: TextStyle(
                    color: isActive ? Colors.white : Colors.white70,
                    fontSize: 12,
                    fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ),
    );

    final videoContent = Stack(
      children: [videoWidget, gestureLayer, ...overlays, speedButtons],
    );

    if (_isFullscreen) {
      return Scaffold(
        backgroundColor: Colors.black,
        body: videoContent,
      );
    }

    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        title: Text(
          widget.videoUrl.split('/').last,
          overflow: TextOverflow.ellipsis,
        ),
        backgroundColor: Colors.black87,
        foregroundColor: Colors.white,
      ),
      body: SafeArea(
        child: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _errorMessage != null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline, size: 48, color: Colors.red),
                      const SizedBox(height: 16),
                      Text(_errorMessage!, style: const TextStyle(color: Colors.white)),
                      const SizedBox(height: 16),
                      ElevatedButton(
                          onPressed: _initPlayer, child: Text('common.retry'.tr)),
                    ],
                  ),
                )
              : videoContent,
      ),
    );
  }
}
