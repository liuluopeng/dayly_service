import 'dart:typed_data';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/play_online_music_page.dart';
import 'package:kongde/src/rust/api/wifi_api/song.dart';

class OnlineMusicPage extends StatefulWidget {
  const OnlineMusicPage({super.key});

  @override
  State<OnlineMusicPage> createState() => _OnlineMusicPageState();
}

class _OnlineMusicPageState extends State<OnlineMusicPage> {
  List<SongWithUrlForDart> _songs = [];
  bool _isLoading = false;
  bool _isLoadingMore = false;
  String _errorMessage = '';

  // 分页相关变量
  int _currentPage = 1;
  int _pageSize = 20;
  bool _hasMore = true;

  // 滚动控制器
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();

    // 添加滚动监听器
    _scrollController.addListener(() {
      if (_scrollController.position.pixels >=
              _scrollController.position.maxScrollExtent - 200 &&
          !_isLoading &&
          _hasMore) {
        _fetchSongs(isLoadMore: true);
      }
    });

    _fetchSongs();
  }

  Future<void> _fetchSongs({bool isLoadMore = false}) async {
    if (isLoadMore && (!_hasMore || _isLoadingMore)) {
      return;
    }

    setState(() {
      if (isLoadMore) {
        _isLoadingMore = true;
      } else {
        _isLoading = true;
      }
      _errorMessage = '';
    });

    try {
      final page = isLoadMore ? _currentPage + 1 : 1;
      final response = await getAllSongsForDart(
        page: page,
        pageSize: _pageSize,
      );

      final songs = response.data;

      setState(() {
        if (isLoadMore) {
          _songs.addAll(songs);
          _currentPage = page;
          _isLoadingMore = false;
        } else {
          _songs = songs;
          _currentPage = 1;
          _isLoading = false;
        }
        _hasMore = songs.length == _pageSize;
      });
    } catch (e) {
      setState(() {
        _errorMessage = 'onlineMusic.loadFailed'.trParams({'error': '$e'});
        _isLoading = false;
        _isLoadingMore = false;
      });
    }
  }

  Future<void> _playSong(SongWithUrlForDart song, int index) async {
    try {
      if (mounted) {
        setState(() {
          _isLoading = true;
          _errorMessage = '';
        });
      }

      // 立即导航到播放页面，不等待音频加载完成
      Get.to(() => const PlayOnlineMusicPage());

      // 立即设置 _isLoading = false，因为导航已经完成
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }

      // 在后台执行音频加载操作
      if (_songs.isNotEmpty) {
        // 转换 _songs 列表为 playOnlinePlaylist 方法所需的格式
        final songList = _songs
            .map(
              (song) => {
                'songId': song.id.toString(),
                'title': song.title,
                'artist': song.artist,
                'album': song.album,
                'coverUrl': song.coverUrl,
              },
            )
            .toList();

        await Get.find<AudioPlayerHandler>().playOnlinePlaylist(
          songList,
          index,
        );
      }
    } catch (e) {
      print('播放歌曲失败: $e');
      // 不在列表页面显示错误，因为用户已经在播放页面
    } finally {
      // 确保 _isLoading 为 false
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'onlineMusic.title'.tr),
      body: SafeArea(
        child: Column(
          children: [
            if (_errorMessage.isNotEmpty) ...[
              Container(
                padding: const EdgeInsets.all(16),
                color: Colors.red[100],
                child: Text(
                  _errorMessage,
                  style: const TextStyle(color: Colors.red),
                ),
              ),
            ],
            if (_isLoading) ...[
              const Expanded(child: Center(child: CircularProgressIndicator())),
            ] else ...[
              Expanded(
                child: ListView.builder(
                  controller: _scrollController,
                  padding: EdgeInsets.zero,
                  itemCount: _songs.length + (_hasMore || _isLoadingMore ? 1 : 0),
                  itemBuilder: (context, index) {
                    if (index == _songs.length) {
                      // 显示加载指示器
                      return Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Center(child: CircularProgressIndicator()),
                      );
                    }

                    final song = _songs[index];
                    return Container(
                      height: 70,
                      child: Stack(
                        children: [
                          if (song.coverUrl != null &&
                              song.coverUrl!.isNotEmpty)
                            Positioned.fill(
                              child: Container(
                                child: Image.network(
                                  song.coverUrl!,
                                  fit: BoxFit.cover,
                                  errorBuilder: (context, error, stackTrace) {
                                    return Container(color: Colors.grey[100]);
                                  },
                                ),
                              ),
                            ),
                          if (song.coverUrl != null &&
                              song.coverUrl!.isNotEmpty)
                            Positioned.fill(
                              child: Container(
                                decoration: BoxDecoration(
                                  color: Colors.black.withOpacity(0.3),
                                ),
                              ),
                            ),
                          Container(
                            decoration: BoxDecoration(
                              color:
                                  song.coverUrl != null &&
                                      song.coverUrl!.isNotEmpty
                                  ? Colors.transparent
                                  : Colors.grey[100],
                            ),
                            child: ListTile(
                              leading: Container(
                                width: 50,
                                height: 50,
                                decoration: BoxDecoration(
                                  borderRadius: BorderRadius.circular(8),
                                  color: Colors.grey[200],
                                ),
                                child:
                                    song.coverUrl != null &&
                                        song.coverUrl!.isNotEmpty
                                    ? ClipRRect(
                                        borderRadius: BorderRadius.circular(8),
                                        child: Image.network(
                                          song.coverUrl!,
                                          fit: BoxFit.cover,
                                          errorBuilder:
                                              (context, error, stackTrace) {
                                                return Icon(
                                                  Icons.music_note,
                                                  color: Colors.grey,
                                                );
                                              },
                                        ),
                                      )
                                    : Icon(
                                        Icons.music_note,
                                        color: Colors.grey,
                                      ),
                              ),
                              title: Text(
                                song.title,
                                style: TextStyle(
                                  color:
                                      song.coverUrl != null &&
                                          song.coverUrl!.isNotEmpty
                                      ? Colors.white
                                      : Colors.black,
                                  fontWeight: FontWeight.w500,
                                ),
                              ),
                              subtitle:
                                  song.artist != null && song.artist!.isNotEmpty
                                  ? Text(
                                      '${song.artist} - ${song.album ?? ''}',
                                      style: TextStyle(
                                        color:
                                            song.coverUrl != null &&
                                                song.coverUrl!.isNotEmpty
                                            ? Colors.white.withOpacity(0.8)
                                            : Colors.grey,
                                        fontSize: 12,
                                      ),
                                    )
                                  : null,
                              trailing: Icon(
                                Icons.play_circle_outline,
                                color:
                                    song.coverUrl != null &&
                                        song.coverUrl!.isNotEmpty
                                    ? Colors.white
                                    : Colors.grey,
                              ),
                              onTap: () => _playSong(song, index),
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
