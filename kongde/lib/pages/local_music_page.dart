import 'dart:io';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:path_provider/path_provider.dart';
// ignore_for_file: invalid_use_of_internal_member
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:kongde/services/music_service.dart';
import 'package:kongde/services/sqlite_storage.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:kongde/pages/play_local_music_page.dart';

class LocalMusicPage extends StatefulWidget {
  const LocalMusicPage({super.key});

  @override
  State<LocalMusicPage> createState() => _LocalMusicPageState();
}

class _LocalMusicPageState extends State<LocalMusicPage> {
  List<Map<String, dynamic>> _songs = [];
  bool _loading = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadMusic();
  }

  Future<void> _loadMusic() async {
    setState(() { _loading = true; _error = null; });
    try {
      final songs = await MusicService.getMusicFromDevice();
      setState(() { _songs = songs; _loading = false; });
    } catch (e) {
      setState(() { _error = e.toString(); _loading = false; });
    }
  }

  Future<void> _scanMusic() async {
    setState(() { _loading = true; _error = null; });
    try {
      final songs = await MusicService.scanDeviceMusic();
      setState(() { _songs = songs; _loading = false; });
    } catch (e) {
      setState(() { _error = e.toString(); _loading = false; });
    }
  }

  Future<void> _playSong(int index) async {
    if (_songs.isEmpty) return;
    final playlist = _songs
        .map((s) => {
              'url': s['path'] as String,
              'title': s['title'] as String,
              'artist': s['artist'] as String? ?? '',
              'album': s['album'] as String? ?? '',
              'coverPath': s['coverPath'] ?? '',
            })
        .toList();
    Get.find<AudioPlayerHandler>().playFromPlaylist(playlist, index);
    Get.to(() => const PlayLocalMusicPage());
  }

  Widget _buildArtistAlbumRow(Map<String, dynamic> song) {
    final artist = song['artist'] as String? ?? '';
    final album = song['album'] as String? ?? '';
    return Row(children: [
      if (artist.isNotEmpty)
        GestureDetector(
          onTap: () => _showFiltered('artist', artist),
          child: Text(artist, style: TextStyle(fontSize: 12, color: Colors.blue[400], decoration: TextDecoration.underline)),
        ),
      if (artist.isNotEmpty && album.isNotEmpty) Text(' · ', style: TextStyle(fontSize: 12, color: Colors.grey[500])),
      if (album.isNotEmpty)
        GestureDetector(
          onTap: () => _showFiltered('album', album),
          child: Text(album, style: TextStyle(fontSize: 12, color: Colors.blue[400], decoration: TextDecoration.underline)),
        ),
    ]);
  }

  void _showFiltered(String field, String value) {
    final filtered = _songs.where((s) => (s[field] as String? ?? '') == value).toList();
    Get.to(() => _FilteredSongsPage(field: field, value: value, songs: filtered));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('本地音乐', style: TextStyle(fontSize: 16)),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Get.back(),
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () async {
              await MusicService.clearLocalSongs();
              _scanMusic();
            },
            tooltip: '重新扫描',
          ),
          PopupMenuButton<String>(
            tooltip: '清除数据',
            onSelected: (v) async {
              if (v == 'clear_songs') {
                await MusicService.clearLocalSongs();
                final dir = await getApplicationDocumentsDirectory();
                final coversDir = Directory('${dir.path}/covers');
                if (await coversDir.exists()) await coversDir.delete(recursive: true);
                await SqliteStorage().init();
                await RustLib.instance.api.crateApiDbKvDelete(key: '__local_songs__');
                setState(() { _songs = []; });
                showDialog(context: context, builder: (_) => AlertDialog(title: const Text('已清除'), content: const Text('歌单、封面、元数据缓存已全部删除'), actions: [TextButton(onPressed: () => Get.back(), child: const Text('好'))]));
              } else if (v == 'clear_all') {
                await SqliteStorage().init();
                await RustLib.instance.api.crateApiDbKvClear();
                final dir = await getApplicationDocumentsDirectory();
                final coversDir = Directory('${dir.path}/covers');
                if (await coversDir.exists()) await coversDir.delete(recursive: true);
                final cacheDir = Directory('${dir.path}/../cache/kongde_music_cache');
                if (await cacheDir.exists()) await cacheDir.delete(recursive: true);
                setState(() { _songs = []; });
                showDialog(context: context, builder: (_) => AlertDialog(title: const Text('全部清除'), content: const Text('所有本地数据和缓存已删除，请重启应用'), actions: [TextButton(onPressed: () => Get.back(), child: const Text('好'))]));
              }
            },
            itemBuilder: (_) => const [
              PopupMenuItem(value: 'clear_songs', child: Text('清除歌单缓存')),
              PopupMenuItem(value: 'clear_all', child: Text('清除所有本地数据')),
            ],
          ),
        ],
      ),
      body: SafeArea(
        child: _buildBody(),
      ),
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.error_outline, size: 48, color: Colors.grey[600]),
              const SizedBox(height: 12),
              Text(_error!, textAlign: TextAlign.center,
                  style: TextStyle(color: Colors.grey[500])),
              const SizedBox(height: 16),
              ElevatedButton(onPressed: _loadMusic, child: const Text('重试')),
            ],
          ),
        ),
      );
    }
    if (_songs.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.library_music, size: 48, color: Colors.grey[600]),
            const SizedBox(height: 12),
            Text(_loading ? '正在扫描...' : '暂无本地歌曲', style: TextStyle(color: Colors.grey[500])),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: _loading ? null : _scanMusic,
              icon: const Icon(Icons.search),
              label: const Text('扫描本地歌曲'),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: _songs.length,
      itemBuilder: (context, index) {
        final song = _songs[index];
        final coverPath = song['coverPath'] as String? ?? '';
        return ListTile(
          leading: CircleAvatar(
            radius: 24,
            backgroundColor: Colors.blue.withAlpha(30),
            backgroundImage: coverPath.isNotEmpty ? FileImage(File(coverPath)) : null,
            child: coverPath.isEmpty
                ? Text('${index + 1}', style: const TextStyle(color: Colors.blue, fontSize: 12))
                : null,
          ),
          title: Text(
            song['title'] ?? '',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 14),
          ),
          subtitle: _buildArtistAlbumRow(song),
          trailing: Text(
            song['duration'] ?? '',
            style: TextStyle(fontSize: 12, color: Colors.grey[600]),
          ),
          onTap: () => _playSong(index),
        );
      },
    );
  }
}

class _FilteredSongsPage extends StatelessWidget {
  final String field;
  final String value;
  final List<Map<String, dynamic>> songs;

  const _FilteredSongsPage({required this.field, required this.value, required this.songs});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(value, style: const TextStyle(fontSize: 16)),
        leading: IconButton(icon: const Icon(Icons.arrow_back), onPressed: () => Get.back()),
      ),
      body: SafeArea(
        child: ListView.builder(
          itemCount: songs.length,
          itemBuilder: (context, index) {
            final song = songs[index];
            final coverPath = song['coverPath'] as String? ?? '';
            return ListTile(
              leading: CircleAvatar(
                radius: 24, backgroundColor: Colors.blue.withAlpha(30),
                backgroundImage: coverPath.isNotEmpty ? FileImage(File(coverPath)) : null,
                child: coverPath.isEmpty
                    ? Text('${index + 1}', style: const TextStyle(color: Colors.blue, fontSize: 12)) : null,
              ),
              title: Text(song['title'] ?? '', maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 14)),
              subtitle: Text('${song['artist'] ?? ''} · ${song['album'] ?? ''}', style: TextStyle(fontSize: 12, color: Colors.grey[500])),
              trailing: Text(song['duration'] ?? '', style: TextStyle(fontSize: 12, color: Colors.grey[600])),
              onTap: () {
                final playlist = songs.map((s) => {
                  'url': s['path'] as String, 'title': s['title'] as String,
                  'artist': s['artist'] as String? ?? '', 'album': s['album'] as String? ?? '',
                  'coverPath': s['coverPath'] ?? '',
                }).toList();
                Get.find<AudioPlayerHandler>().playFromPlaylist(playlist, index);
                Get.to(() => const PlayLocalMusicPage());
              },
            );
          },
        ),
      ),
    );
  }
}
