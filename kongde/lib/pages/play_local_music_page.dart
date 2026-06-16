import 'dart:async';
import 'dart:io';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:audio_service/audio_service.dart';
import 'package:rxdart/rxdart.dart' as rxdart;
import 'package:kongde/main.dart';
import 'package:kongde/models/media_state.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:kongde/controllers/settings_controller.dart';
import 'package:flutter_audio_visualizer/flutter_audio_visualizer.dart';
import 'package:kongde/widgets/seek_bar.dart';
import 'package:kongde/widgets/spectrum_painter.dart';
import 'package:kongde/widgets/playback_controls.dart';
import 'package:kongde/widgets/album_art_widget.dart';
import 'package:kongde/widgets/song_info_widget.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/widgets/appbar_mini_window.dart';

class PlayLocalMusicPage extends StatefulWidget {
  const PlayLocalMusicPage({super.key});

  @override
  State<PlayLocalMusicPage> createState() => _PlayLocalMusicPageState();
}

class _PlayLocalMusicPageState extends State<PlayLocalMusicPage> {
  final ValueNotifier<List<double>> _fftDataNotifier = ValueNotifier(
    List.filled(64, 0.0),
  );
  int? _audioSessionId;
  bool _visualizerInitialized = false;
  Map<String, Color?> _mainColors = {
    'primaryColor': Colors.grey.shade700,
    'secondaryColor': Colors.blue,
  };
  StreamSubscription? _colorSubscription;
  StreamSubscription? _audioSessionSubscription;
  StreamSubscription? _fftDataSubscription;
  StreamSubscription? _playbackStateSubscription;
  Timer? _fftUpdateTimer;
  List<double> _pendingFftData = [];
  late final SettingsController _settingsController;

  @override
  void initState() {
    super.initState();
    _settingsController = Get.find<SettingsController>();
    _setupColorListener();
    _setupPlaybackStateListener();
    _setupVisualizer();
  }

  void _setupPlaybackStateListener() {
    _playbackStateSubscription = Get.find<AudioPlayerHandler>().playbackState
        .map((state) => state.processingState)
        .distinct()
        .listen((processingState) {
          if (mounted) {
            AppBarMiniWindow.show('播放状态: ${processingState.name}');
          }
        });
  }

  void _setupColorListener() {
    _colorSubscription = Get.find<AudioPlayerHandler>().mainColors.listen((
      colors,
    ) {
      logger.i('收到颜色更新: $colors');
      if (mounted) {
        setState(() {
          _mainColors = colors;
        });
      }
    });
  }

  Future<void> _setupVisualizer() async {
    if (Platform.isAndroid) {
      _audioSessionSubscription = Get.find<AudioPlayerHandler>()
          .player
          .androidAudioSessionIdStream
          .listen((sessionId) {
            if (mounted && sessionId != null && sessionId != _audioSessionId) {
              _audioSessionId = sessionId;
              _initializeVisualizer();
            }
          });
    } else if (Platform.isIOS || Platform.isMacOS) {
      Get.find<AudioPlayerHandler>().mediaItem.listen((mediaItem) {
        if (mounted && mediaItem != null) {
          _initializeVisualizer();
        }
      });
    }

    _fftDataSubscription = FlutterAudioVisualizer.fftDataStream.listen((data) {
      if (mounted) {
        _pendingFftData = data;
        _fftUpdateTimer?.cancel();
        _fftUpdateTimer = Timer(const Duration(milliseconds: 16), () {
          if (mounted) {
            _fftDataNotifier.value = _pendingFftData;
          }
        });
      }
    });
  }

  Future<void> _initializeVisualizer() async {
    try {
      if (Platform.isAndroid && _audioSessionId != null) {
        final hasPermission = await FlutterAudioVisualizer.requestPermission();
        if (hasPermission) {
          await FlutterAudioVisualizer.initialize(_audioSessionId!);
          await FlutterAudioVisualizer.start();
          if (mounted) {
            setState(() {
              _visualizerInitialized = true;
            });
          }
        }
      } else if (Platform.isIOS || Platform.isMacOS) {
        final mediaItem = Get.find<AudioPlayerHandler>().mediaItem.value;
        if (mediaItem != null && mediaItem.id.isNotEmpty) {
          try {
            await FlutterAudioVisualizer.stop();

            String filePath = mediaItem.id;
            if (filePath.startsWith('asset:///')) return;
            if (filePath.startsWith('http://') || filePath.startsWith('https://')) return;
            if (filePath.startsWith('file://')) {
              filePath = Uri.parse(filePath).toFilePath();
            }

            final success = await FlutterAudioVisualizer.initializeWithFile(filePath);
            if (success) {
              await FlutterAudioVisualizer.start();
              if (mounted) {
                setState(() {
                  _visualizerInitialized = true;
                });
              }
            }
          } catch (e) {
            print('初始化频谱失败: $e');
          }
        }
      }
    } catch (e) {
      print('初始化频谱异常: $e');
    }
  }

  @override
  void dispose() {
    _colorSubscription?.cancel();
    _audioSessionSubscription?.cancel();
    _fftDataSubscription?.cancel();
    _playbackStateSubscription?.cancel();
    _fftUpdateTimer?.cancel();
    _fftDataNotifier.dispose();
    try {
      FlutterAudioVisualizer.stop();
    } catch (e) {
      print('停止频谱失败: $e');
    }
    super.dispose();
  }

  @override
  Widget build(context) {
    return ValueListenableBuilder(
      valueListenable: _fftDataNotifier,
      builder: (context, _, __) {
        final backgroundColor =
            _mainColors['primaryColor'] ?? Colors.grey.shade700;
        final isDarkBackground = _isDarkColor(backgroundColor);
        final iconColor = isDarkBackground ? Colors.white : Colors.black;

        return Scaffold(
          appBar: CommonAppBar(
            backgroundColor:
                _mainColors['primaryColor'] ?? Colors.grey.shade700,
            iconTheme: IconThemeData(color: iconColor),
          ),
          body: SafeArea(
            child: Obx(() {
              final effectiveBackgroundColor = _getEffectiveBackgroundColor();

              return Stack(
                fit: StackFit.expand,
                children: [
                  if (_settingsController.isBlurBackground)
                    _buildBlurBackground(),
                  Container(
                    color: effectiveBackgroundColor,
                    child: _buildBody(
                      effectiveBackgroundColor,
                      _isDarkColor(effectiveBackgroundColor),
                      _isDarkColor(effectiveBackgroundColor)
                          ? Colors.white
                          : Colors.black,
                      _mainColors['secondaryColor'] ?? Colors.blue,
                      _isDarkColor(effectiveBackgroundColor)
                          ? Colors.white.withValues(alpha: 0.3)
                          : Colors.black.withValues(alpha: 0.3),
                    ),
                  ),
                ],
              );
            }),
          ),
        );
      },
    );
  }

  Widget _buildBody(
    Color backgroundColor,
    bool isDarkBackground,
    Color iconColor,
    Color seekBarActiveColor,
    Color seekBarInactiveColor,
  ) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final isLandscape = constraints.maxWidth > constraints.maxHeight;

        if (isLandscape) {
          return _buildLandscapeLayout(
            backgroundColor,
            isDarkBackground,
            iconColor,
            seekBarActiveColor,
            seekBarInactiveColor,
          );
        } else {
          return _buildPortraitLayout(
            backgroundColor,
            isDarkBackground,
            iconColor,
            seekBarActiveColor,
            seekBarInactiveColor,
          );
        }
      },
    );
  }

  Widget _buildBlurBackground() {
    return StreamBuilder<MediaItem?>(
      stream: Get.find<AudioPlayerHandler>().mediaItem,
      builder: (context, snapshot) {
        final mediaItem = snapshot.data;
        final artUri = mediaItem?.artUri;

        if (artUri != null) {
          Widget imageWidget;

          if (artUri.isScheme('file')) {
            imageWidget = Image.file(
              File(artUri.toFilePath()),
              fit: BoxFit.cover,
              errorBuilder: (context, error, stackTrace) {
                return Container(color: Colors.grey.shade700);
              },
            );
          } else if (artUri.isScheme('http') || artUri.isScheme('https')) {
            imageWidget = Image.network(
              artUri.toString(),
              fit: BoxFit.cover,
              errorBuilder: (context, error, stackTrace) {
                return Container(color: Colors.grey.shade700);
              },
            );
          } else if (artUri.isScheme('data')) {
            imageWidget = Image.memory(
              UriData.fromUri(artUri).contentAsBytes(),
              fit: BoxFit.cover,
              errorBuilder: (context, error, stackTrace) {
                return Container(color: Colors.grey.shade700);
              },
            );
          } else {
            imageWidget = Container(color: Colors.grey.shade700);
          }

          return Stack(
            fit: StackFit.expand,
            children: [
              Positioned.fill(child: imageWidget),
              Positioned.fill(
                child: BackdropFilter(
                  filter: ImageFilter.blur(sigmaX: 20.0, sigmaY: 20.0),
                  child: Container(color: Colors.black.withValues(alpha: 0.5)),
                ),
              ),
            ],
          );
        } else {
          return Container(color: Colors.grey.shade700);
        }
      },
    );
  }

  Widget _buildSpectrumVisualizer() {
    return ValueListenableBuilder<List<double>>(
      valueListenable: _fftDataNotifier,
      builder: (context, fftData, child) {
        return RepaintBoundary(
          child: SizedBox(
            height: 100,
            width: double.infinity,
            child: CustomPaint(
              painter: SpectrumPainter(
                fftData,
                _mainColors['secondaryColor'] ?? Colors.blue,
              ),
            ),
          ),
        );
      },
    );
  }

  Stream<MediaState> get _mediaStateStream =>
      rxdart.Rx.combineLatest2<MediaItem?, Duration, MediaState>(
        Get.find<AudioPlayerHandler>().mediaItem,
        Get.find<AudioPlayerHandler>().player.positionStream,
        (mediaItem, position) => MediaState(mediaItem, position),
      );

  bool _isDarkColor(Color color) {
    final luminance = color.computeLuminance();
    return luminance < 0.5;
  }

  Color _getEffectiveBackgroundColor() {
    if (_settingsController.isBlurBackground) {
      return Colors.transparent;
    } else if (_settingsController.isDefaultColor) {
      return Colors.grey.shade700;
    } else {
      return _mainColors['primaryColor'] ?? Colors.grey.shade700;
    }
  }

  Widget _buildPortraitLayout(
    Color backgroundColor,
    bool isDarkBackground,
    Color iconColor,
    Color seekBarActiveColor,
    Color seekBarInactiveColor,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          // Album art + song info
          Flexible(
            child: StreamBuilder<MediaItem?>(
              stream: Get.find<AudioPlayerHandler>().mediaItem,
              builder: (context, snapshot) {
                final mediaItem = snapshot.data;
                return Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Flexible(child: AlbumArtWidget(artUri: mediaItem?.artUri)),
                    if (_visualizerInitialized) _buildSpectrumVisualizer(),
                    SongInfoWidget(
                      mediaItem: mediaItem,
                      mainColors: _mainColors,
                    ),
                  ],
                );
              },
            ),
          ),
          // Play controls
          StreamBuilder<bool>(
            stream: Get.find<AudioPlayerHandler>().playingState,
            builder: (context, snapshot) {
              final playing = snapshot.data ?? false;
              return PlaybackControls(
                isPlaying: playing,
                iconColor: iconColor,
                onLikeChanged: () {
                  setState(() {});
                },
              );
            },
          ),
          const SizedBox(height: 8),
          // Seek bar
          StreamBuilder<MediaState>(
            stream: _mediaStateStream,
            builder: (context, snapshot) {
              final mediaState = snapshot.data;
              return SeekBar(
                duration: mediaState?.mediaItem?.duration ?? Duration.zero,
                position: mediaState?.position ?? Duration.zero,
                onChangeEnd: (newPosition) {
                  Get.find<AudioPlayerHandler>().seek(newPosition);
                },
                activeColor: seekBarActiveColor,
                inactiveColor: seekBarInactiveColor,
                textColor: iconColor,
              );
            },
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }

  Widget _buildLandscapeLayout(
    Color backgroundColor,
    bool isDarkBackground,
    Color iconColor,
    Color seekBarActiveColor,
    Color seekBarInactiveColor,
  ) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Row(
        children: [
          Expanded(
            flex: 1,
            child: StreamBuilder<MediaItem?>(
              stream: Get.find<AudioPlayerHandler>().mediaItem,
              builder: (context, snapshot) {
                final mediaItem = snapshot.data;
                return Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    AlbumArtWidget(artUri: mediaItem?.artUri),
                    const SizedBox(height: 16),
                    if (_visualizerInitialized) _buildSpectrumVisualizer(),
                    const SizedBox(height: 24),
                    StreamBuilder<bool>(
                      stream: Get.find<AudioPlayerHandler>().playingState,
                      builder: (context, snapshot) {
                        final playing = snapshot.data ?? false;
                        return PlaybackControls(
                          isPlaying: playing,
                          iconColor: iconColor,
                          onLikeChanged: () {
                            setState(() {});
                          },
                        );
                      },
                    ),
                    const SizedBox(height: 24),
                    StreamBuilder<MediaState>(
                      stream: _mediaStateStream,
                      builder: (context, snapshot) {
                        final mediaState = snapshot.data;
                        return SeekBar(
                          duration:
                              mediaState?.mediaItem?.duration ?? Duration.zero,
                          position: mediaState?.position ?? Duration.zero,
                          onChangeEnd: (newPosition) {
                            Get.find<AudioPlayerHandler>().seek(newPosition);
                          },
                          activeColor: seekBarActiveColor,
                          inactiveColor: seekBarInactiveColor,
                          textColor: iconColor,
                        );
                      },
                    ),
                  ],
                );
              },
            ),
          ),
          const SizedBox(width: 32),
          Expanded(
            flex: 1,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                StreamBuilder<MediaItem?>(
                  stream: Get.find<AudioPlayerHandler>().mediaItem,
                  builder: (context, snapshot) {
                    return SongInfoWidget(
                      mediaItem: snapshot.data,
                      mainColors: _mainColors,
                    );
                  },
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
