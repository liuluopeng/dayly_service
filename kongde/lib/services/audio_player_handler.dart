import 'dart:io';
import 'dart:typed_data';
import 'package:audio_service/audio_service.dart';
import 'package:flutter/material.dart';
import 'package:just_audio/just_audio.dart';
import 'package:rxdart/rxdart.dart';
import 'dart:async';
import 'package:kongde/src/rust/api/wifi_api/song.dart';
import 'package:get/get.dart';
import 'package:kongde/widgets/appbar_mini_window.dart';
import 'package:path_provider/path_provider.dart';
import 'package:uuid/uuid.dart';

/// An [AudioHandler] for playing a playlist.
class AudioPlayerHandler extends BaseAudioHandler with SeekHandler {
  List<MediaItem> _playlist = [];
  final _player = AudioPlayer();
  final _playingState = BehaviorSubject<bool>.seeded(false);
  final _mainColors = BehaviorSubject<Map<String, Color?>>.seeded({
    'primaryColor': Colors.grey.shade700,
    'secondaryColor': Colors.blue,
  });

  Stream<bool> get playingState => _playingState.stream;

  // 线控耳机点击检测
  int _hookPressedCount = 0;
  Timer? _hookTimer;
  static const int _hookDelayMs = 600;

  // 提供主要颜色流
  Stream<Map<String, Color?>> get mainColors => _mainColors;

  // 暴露音频播放器实例
  AudioPlayer get player => _player;

  bool _listenersSetup = false;
  bool _isSettingUpPlaylist = false;

  // 在线歌曲播放相关
  List<Map<String, dynamic>>? _onlineSongs;
  int _currentOnlineSongIndex = 0;

  Future<void> playSingleAudio(
    String url, {
    Map<String, String>? headers,
    String? title,
  }) async {
    try {
      final mediaItem = MediaItem(
        id: url,
        title: title ?? 'audio.onlineAudio'.tr,
        artist: 'music.unknownArtist'.tr,
        album: 'music.unknownAlbum'.tr,
        duration: const Duration(seconds: 180),
        extras: {
          'primaryColor': Colors.grey.shade700.value,
          'secondaryColor': Colors.blue.value,
        },
      );

      // 清空播放列表，只播放当前音频
      _playlist = [mediaItem];

      // 确保监听器已设置
      if (!_listenersSetup) {
        _setupPlayerListeners();
        _listenersSetup = true;
      }

      // 更新媒体项和队列
      this.mediaItem.add(mediaItem);
      queue.add(_playlist);

      // 直接执行网络操作，不使用 isolate，因为 AudioPlayer 不能在 isolate 之间传递
      // 使用 AudioSource.uri 来设置音频源，这样可以更好地处理网络音频
      final audioSource = AudioSource.uri(Uri.parse(url), headers: headers);

      // 停止当前播放并重置播放器
      await _player.stop();
      await _player.setAudioSource(audioSource);

      // 开始播放
      await _player.play();
    } catch (e) {
      print('播放单个音频失败: $e');
    }
  }

  /// 播放列表中的歌曲
  /// [songs] 是歌曲列表，每个元素是一个包含 url、title 和可选的 coverUrl 的 Map
  /// [index] 是要播放的歌曲在列表中的索引
  Future<void> playFromPlaylist(
    List<Map<String, dynamic>> songs,
    int index,
  ) async {
    try {
      if (songs.isEmpty) {
        print('歌曲列表为空');
        return;
      }

      if (index < 0 || index >= songs.length) {
        print('索引超出范围');
        return;
      }

      // 构建播放列表
      _playlist = [];
      final audioSources = <AudioSource>[];

      for (var song in songs) {
        final url = song['url']!;
        final title = song['title']!;
        final artist = song['artist'] as String? ?? 'music.unknownArtist'.tr;
        final album = song['album'] as String? ?? 'music.unknownAlbum'.tr;
        var coverUrl = song['coverUrl'] as String?;

        // 本地歌曲：用持久化的 coverPath 文件
        final coverPath = song['coverPath'] as String?;
        if (coverUrl == null && coverPath != null && coverPath.isNotEmpty) {
          final f = File(coverPath);
          if (await f.exists()) {
            coverUrl = f.uri.toString();
          }
        }

        final mediaItem = MediaItem(
          id: url,
          title: title,
          artist: artist,
          album: album,
          duration: const Duration(seconds: 180),
          artUri: coverUrl != null && coverUrl.isNotEmpty
              ? Uri.parse(coverUrl)
              : null,
          extras: {
            'primaryColor': (song['primaryColor'] as int?) ?? Colors.grey.shade700.value,
            'secondaryColor': (song['secondaryColor'] as int?) ?? Colors.blue.value,
          },
        );

        _playlist.add(mediaItem);
        audioSources.add(AudioSource.uri(Uri.parse(url)));
      }

      // 确保监听器已设置
      if (!_listenersSetup) {
        _setupPlayerListeners();
        _listenersSetup = true;
      }

      // 更新媒体项和队列
      this.mediaItem.add(_playlist[index]);
      queue.add(_playlist);

      // 标记正在设置播放列表，防止监听器在设置过程中发出错误的媒体项
      _isSettingUpPlaylist = true;

      // 停止当前播放并重置播放器
      await _player.stop();
      await _player.setAudioSource(
        ConcatenatingAudioSource(children: audioSources),
      );

      // 跳转到指定索引的歌曲
      await _player.seek(Duration.zero, index: index);

      // 标记播放列表设置完成
      _isSettingUpPlaylist = false;

      // 开始播放
      await _player.play();
    } catch (e) {
      print('从播放列表播放失败: $e');
    }
  }

  /// Initialise our audio handler.
  AudioPlayerHandler() {}

  Duration _parseDuration(String durationStr) {
    final parts = durationStr.split(':');
    if (parts.length == 2) {
      final minutes = int.tryParse(parts[0]) ?? 0;
      final seconds = int.tryParse(parts[1]) ?? 0;
      return Duration(minutes: minutes, seconds: seconds);
    } else if (parts.length == 3) {
      final hours = int.tryParse(parts[0]) ?? 0;
      final minutes = int.tryParse(parts[1]) ?? 0;
      final seconds = int.tryParse(parts[2]) ?? 0;
      return Duration(hours: hours, minutes: minutes, seconds: seconds);
    }
    return const Duration(seconds: 180);
  }

  void _setupPlayerListeners() {
    _player.playbackEventStream.listen((event) {
      _playingState.add(_player.playing);

      if (!_isSettingUpPlaylist) {
        // 确定当前应该使用的索引
        int currentIndex;
        if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
          // 在线歌曲模式，使用我们自己的索引
          currentIndex = _currentOnlineSongIndex;
        } else if (event.currentIndex != null) {
          // 本地播放列表模式，使用播放器的索引
          currentIndex = event.currentIndex!;
        } else {
          // 没有有效索引，跳过
          return;
        }

        // 确保索引在有效范围内
        if (currentIndex >= 0 && currentIndex < _playlist.length) {
          if (_player.duration != null) {
            _playlist[currentIndex] = _playlist[currentIndex].copyWith(
              duration: _player.duration,
            );
          }
          mediaItem.add(_playlist[currentIndex]);

          AppBarMiniWindow.show(
            '切歌: ${_playlist[currentIndex].title}, 路径: ${_playlist[currentIndex].id}',
          );

          final currentMediaItem = _playlist[currentIndex];
          final extras = currentMediaItem.extras ?? {};
          final primaryColor =
              extras['primaryColor'] as int? ?? Colors.grey.shade700.value;
          final secondaryColor =
              extras['secondaryColor'] as int? ?? Colors.blue.value;

          _mainColors.add({
            'primaryColor': Color(primaryColor),
            'secondaryColor': Color(secondaryColor),
          });
        }
      }

      if (!_isSettingUpPlaylist && event.duration != null) {
        // 确定当前应该使用的索引
        int currentIndex;
        if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
          // 在线歌曲模式，使用我们自己的索引
          currentIndex = _currentOnlineSongIndex;
        } else if (event.currentIndex != null) {
          // 本地播放列表模式，使用播放器的索引
          currentIndex = event.currentIndex!;
        } else {
          // 没有有效索引，跳过
          return;
        }

        // 确保索引在有效范围内
        if (currentIndex >= 0 &&
            currentIndex < _playlist.length &&
            _playlist[currentIndex].duration != event.duration) {
          _playlist[currentIndex] = _playlist[currentIndex].copyWith(
            duration: event.duration,
          );
          mediaItem.add(_playlist[currentIndex]);
        }
      }

      if (event.processingState == ProcessingState.completed) {
        Future.delayed(Duration.zero, () => _playNext());
      }
    });

    _player.playbackEventStream.map(_transformEvent).pipe(playbackState);

    // 初始化媒体项时，根据当前模式选择正确的索引
    int initialIndex = 0;
    if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
      initialIndex = _currentOnlineSongIndex;
    }
    if (initialIndex >= 0 && initialIndex < _playlist.length) {
      mediaItem.add(_playlist[initialIndex]);
    } else if (_playlist.isNotEmpty) {
      mediaItem.add(_playlist[0]);
    } else {
      mediaItem.add(MediaItem(id: '', title: ''));
    }
    queue.add(_playlist);
  }

  @override
  Future<void> play() async {
    await _player.play();
  }

  @override
  Future<void> pause() async {
    await _player.pause();
  }

  @override
  Future<void> stop() async {
    await _player.stop();
  }

  @override
  Future<void> seek(Duration position) async {
    await _player.seek(position);
  }

  Future<void> _playNext() async {
    if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
      // 在线歌曲列表模式，更新索引并下载下一首
      _currentOnlineSongIndex =
          (_currentOnlineSongIndex + 1) % _onlineSongs!.length;
      await _downloadAndPlayCurrentOnlineSong();
    } else if (_player.hasNext) {
      // 本地播放列表模式
      await _player.seekToNext();
    } else if (_playlist.isNotEmpty) {
      // 循环播放
      await _player.seek(Duration.zero, index: 0);
    }
  }

  @override
  Future<void> skipToNext() async {
    await _playNext();
  }

  @override
  Future<void> skipToPrevious() async {
    if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
      // 在线歌曲列表模式，更新索引并下载上一首
      _currentOnlineSongIndex =
          (_currentOnlineSongIndex - 1) % _onlineSongs!.length;
      if (_currentOnlineSongIndex < 0) {
        _currentOnlineSongIndex += _onlineSongs!.length;
      }
      await _downloadAndPlayCurrentOnlineSong();
    } else if (_player.hasPrevious) {
      // 本地播放列表模式
      await _player.seekToPrevious();
    } else if (_playlist.isNotEmpty) {
      // 循环播放
      await _player.seek(Duration.zero, index: _playlist.length - 1);
    }
  }

  @override
  Future<void> skipToQueueItem(int index) async {
    if (_onlineSongs != null && _onlineSongs!.isNotEmpty) {
      // 在线歌曲列表模式，更新索引并下载指定歌曲
      if (index >= 0 && index < _onlineSongs!.length) {
        _currentOnlineSongIndex = index;
        await _downloadAndPlayCurrentOnlineSong();
      }
    } else {
      // 本地播放列表模式
      await _player.seek(Duration.zero, index: index);
    }
  }

  @override
  Future<void> setRepeatMode(AudioServiceRepeatMode repeatMode) async {
    switch (repeatMode) {
      case AudioServiceRepeatMode.none:
        await _player.setLoopMode(LoopMode.off);
        break;
      case AudioServiceRepeatMode.one:
        await _player.setLoopMode(LoopMode.one);
        break;
      case AudioServiceRepeatMode.all:
        await _player.setLoopMode(LoopMode.all);
        break;
      case AudioServiceRepeatMode.group:
        break;
    }
  }

  @override
  Future<void> setShuffleMode(AudioServiceShuffleMode shuffleMode) async {
    if (shuffleMode == AudioServiceShuffleMode.all) {
      await _player.setShuffleModeEnabled(true);
    } else {
      await _player.setShuffleModeEnabled(false);
    }
  }

  @override
  Future<void> setSpeed(double speed) async {
    await _player.setSpeed(speed);
  }

  @override
  Future<void> click([MediaButton? button]) async {
    final btn = button ?? MediaButton.media;
    switch (btn) {
      case MediaButton.media:
        _hookPressedCount++;
        if (_hookTimer?.isActive ?? false) {
          _hookTimer?.cancel();
        }

        _hookTimer = Timer(Duration(milliseconds: _hookDelayMs), () {
          if (_hookPressedCount == 1) {
            if (_player.playing) {
              pause();
            } else {
              play();
            }
          } else if (_hookPressedCount == 2) {
            skipToNext();
          } else if (_hookPressedCount >= 3) {
            skipToPrevious();
          }
          _hookPressedCount = 0;
        });
        break;
      case MediaButton.next:
        skipToNext();
        break;
      case MediaButton.previous:
        skipToPrevious();
        break;
    }
  }

  PlaybackState _transformEvent(PlaybackEvent event) {
    return PlaybackState(
      controls: [
        MediaControl.skipToPrevious,
        if (_player.playing) MediaControl.pause else MediaControl.play,
        MediaControl.stop,
        MediaControl.skipToNext,
      ],
      systemActions: const {
        MediaAction.seek,
        MediaAction.seekForward,
        MediaAction.seekBackward,
      },
      androidCompactActionIndices: const [0, 1, 3],
      processingState: {
        ProcessingState.idle: AudioProcessingState.idle,
        ProcessingState.loading: AudioProcessingState.loading,
        ProcessingState.buffering: AudioProcessingState.buffering,
        ProcessingState.ready: AudioProcessingState.ready,
        ProcessingState.completed: AudioProcessingState.completed,
      }[_player.processingState]!,
      repeatMode: {
        LoopMode.off: AudioServiceRepeatMode.none,
        LoopMode.one: AudioServiceRepeatMode.one,
        LoopMode.all: AudioServiceRepeatMode.all,
      }[_player.loopMode]!,
      shuffleMode: _player.shuffleModeEnabled
          ? AudioServiceShuffleMode.all
          : AudioServiceShuffleMode.none,
      playing: _player.playing,
      updatePosition: _player.position,
      bufferedPosition: _player.bufferedPosition,
      speed: _player.speed,
      queueIndex: event.currentIndex,
    );
  }

  VoidCallback get pauseDirectly =>
      () => _player.pause();

  Future<void> playDirectly() async {
    await _player.play();
  }

  Future<void> playOnlineSong(
    String songId, {
    String? title,
    String? artist,
    String? album,
    String? coverUrl,
  }) async {
    try {
      print('开始获取音频数据: $songId');

      final songUuid = UuidValue(songId);

      final audioData = await getSongFileForDart(songId: songUuid);

      print('获取到音频数据长度: ${audioData.length}');

      if (audioData.isEmpty) {
        print('音频数据为空');
        return;
      }

      final tempDir = await getTemporaryDirectory();
      final tempFile = File('${tempDir.path}/${songId}_temp.mp3');
      print('创建临时文件: ${tempFile.path}');

      await tempFile.writeAsBytes(audioData);
      print('音频数据写入完成');
      print('临时文件大小: ${await tempFile.length()}');

      if (!await tempFile.exists() || await tempFile.length() == 0) {
        print('临时文件不存在或为空');
        return;
      }

      final mediaItem = MediaItem(
        id: songId,
        title: title ?? 'audio.onlineSong'.tr,
        artist: artist ?? 'music.unknownArtist'.tr,
        album: album ?? 'music.unknownAlbum'.tr,
        duration: const Duration(seconds: 180),
        artUri: coverUrl != null && coverUrl.isNotEmpty
            ? Uri.parse(coverUrl)
            : null,
        extras: {
          'primaryColor': Colors.grey.shade700.value,
          'secondaryColor': Colors.blue.value,
        },
      );

      _playlist = [mediaItem];

      if (!_listenersSetup) {
        _setupPlayerListeners();
        _listenersSetup = true;
      }

      this.mediaItem.add(mediaItem);
      queue.add(_playlist);

      await _player.stop();
      print('开始设置音频源');

      // 使用临时文件路径播放
      print('临时文件URI: ${tempFile.uri}');
      print('临时文件是否存在: ${await tempFile.exists()}');
      print('临时文件大小: ${await tempFile.length()}');

      // 尝试直接使用文件路径创建Uri
      final fileUri = Uri.file(tempFile.path);
      print('文件URI: $fileUri');

      final audioSource = AudioSource.uri(fileUri);
      await _player.setAudioSource(audioSource);
      print('音频源设置完成');

      // 检查播放器状态
      print('播放器状态: ${_player.processingState}');
      print('播放器是否准备就绪: ${_player.processingState == ProcessingState.ready}');

      // 确保音量是开启的
      if (_player.volume == 0) {
        await _player.setVolume(1.0);
        print('音量被设置为 1.0');
      }

      await _player.play();
      print('开始播放');
      print('播放状态: ${_player.playing}');
      print('播放器位置: ${_player.position}');
      print('播放器时长: ${_player.duration}');
      print('播放器音量: ${_player.volume}');
      print('播放器语速: ${_player.speed}');
      print('播放器处理状态: ${_player.processingState}');

      // 播放完成后删除临时文件
      // 注意：不要在每次调用时都添加新的监听器
      // 而是在 _setupPlayerListeners 中处理
      // 这里我们使用一个简单的 Future.delayed 来等待播放完成
      // 这不是最优解，但可以避免添加多个监听器的问题
      Future.delayed(const Duration(seconds: 300), () {
        try {
          if (tempFile.existsSync()) {
            tempFile.deleteSync();
            print('临时文件已删除');
          }
        } catch (e) {
          print('删除临时文件失败: $e');
        }
      });
    } catch (e) {
      print('播放在线歌曲失败: $e');
      // 确保即使发生错误也能继续执行
    }
  }

  Future<void> playOnlinePlaylist(
    List<Map<String, dynamic>> songs,
    int index,
  ) async {
    try {
      if (songs.isEmpty) {
        print('歌曲列表为空');
        return;
      }

      if (index < 0 || index >= songs.length) {
        print('索引超出范围');
        return;
      }

      // 保存在线歌曲列表和 songsClient 实例
      _onlineSongs = songs;
      _currentOnlineSongIndex = index;

      // 构建播放列表
      _playlist = [];
      for (var song in songs) {
        final songId = song['songId'] as String?;
        final title = song['title'] as String? ?? 'audio.unknownSong'.tr;
        final artist = song['artist'] as String?;
        final album = song['album'] as String?;
        final coverUrl = song['coverUrl'] as String?;

        if (songId != null) {
          final mediaItem = MediaItem(
            id: songId,
            title: title,
            artist: artist ?? 'music.unknownArtist'.tr,
            album: album ?? 'music.unknownAlbum'.tr,
            duration: const Duration(seconds: 180),
            artUri: coverUrl != null && coverUrl.isNotEmpty
                ? Uri.parse(coverUrl)
                : null,
            extras: {
              'primaryColor': Colors.grey.shade700.value,
              'secondaryColor': Colors.blue.value,
            },
          );
          _playlist.add(mediaItem);
        }
      }

      if (_playlist.isEmpty) {
        print('没有成功加载任何歌曲');
        return;
      }

      if (!_listenersSetup) {
        _setupPlayerListeners();
        _listenersSetup = true;
      }

      this.mediaItem.add(_playlist[index]);
      queue.add(_playlist);

      // 下载并播放当前索引的歌曲
      await _downloadAndPlayCurrentOnlineSong();
    } catch (e) {
      print('播放在线播放列表失败: $e');
    }
  }

  Future<void> _downloadAndPlayCurrentOnlineSong() async {
    try {
      if (_onlineSongs == null || _onlineSongs!.isEmpty) {
        print('在线歌曲列表或 songsClient 未初始化');
        return;
      }

      if (_currentOnlineSongIndex < 0 ||
          _currentOnlineSongIndex >= _onlineSongs!.length) {
        print('索引超出范围');
        return;
      }

      final currentSong = _onlineSongs![_currentOnlineSongIndex];
      final songId = currentSong['songId'] as String?;
      final title = currentSong['title'] as String? ?? 'audio.unknownSong'.tr;

      if (songId == null) {
        print('歌曲 ID 为空');
        return;
      }

      print('开始下载歌曲: $title');

      // 尝试通过 getAudioFile 获取音频数据
      final songUuid = UuidValue(songId);
      final audioData = await getSongFileForDart(songId: songUuid);

      print('获取到音频数据长度: ${audioData.length}');

      if (audioData.isEmpty) {
        print('音频数据为空');
        return;
      }

      // 尝试使用临时文件而不是内存URI，可能会更稳定
      final tempDir = await getTemporaryDirectory();
      final tempFile = File('${tempDir.path}/${songId}_temp.mp3');
      await tempFile.writeAsBytes(audioData);
      final fileUri = Uri.file(tempFile.path);
      print('创建临时文件: ${tempFile.path}，大小: ${await tempFile.length()}');

      // 更新 mediaItem 和主颜色
      final currentMediaItem = _playlist[_currentOnlineSongIndex];
      mediaItem.add(currentMediaItem);

      final extras = currentMediaItem.extras ?? {};
      final primaryColor =
          extras['primaryColor'] as int? ?? Colors.grey.shade700.value;
      final secondaryColor =
          extras['secondaryColor'] as int? ?? Colors.blue.value;

      _mainColors.add({
        'primaryColor': Color(primaryColor),
        'secondaryColor': Color(secondaryColor),
      });

      AppBarMiniWindow.show(
        '切歌: ${currentMediaItem.title}, 路径: ${currentMediaItem.id}',
      );

      _isSettingUpPlaylist = true;

      await _player.stop();
      await _player.setAudioSource(AudioSource.uri(fileUri));

      _isSettingUpPlaylist = false;

      // 确保音量是开启的
      if (_player.volume == 0) {
        await _player.setVolume(1.0);
        print('音量被设置为 1.0');
      }

      await _player.play();
      print('开始播放: $title');
      print('播放状态: ${_player.playing}');
      print('播放器位置: ${_player.position}');
      print('播放器音量: ${_player.volume}');
      print('播放器处理状态: ${_player.processingState}');
    } catch (e) {
      print('下载并播放在线歌曲失败: $e');
    }
  }
}
