import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/pages/actor_movies_page.dart';
import 'package:kongde/pages/genre_movies_page.dart';
import 'package:kongde/pages/universal_video_player_page.dart';
import 'package:kongde/src/rust/api/wifi_api/melatonin.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class MelatoninDetailPage extends StatefulWidget {
  final MelatoninMovieListForDart movie;

  const MelatoninDetailPage({super.key, required this.movie});

  @override
  State<MelatoninDetailPage> createState() => _MelatoninDetailPageState();
}

class _MelatoninDetailPageState extends State<MelatoninDetailPage> {
  bool _isLoading = true;
  Map<String, dynamic>? _nfoData;
  List<String> _previewUrls = [];
  List<Map<String, String>> _btList = [];
  String? _currentMovieId;

  @override
  void initState() {
    super.initState();
    _currentMovieId = widget.movie.id.toString();
    _loadData();
  }

  @override
  void didUpdateWidget(MelatoninDetailPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.movie.id != widget.movie.id) {
      _currentMovieId = widget.movie.id.toString();
      setState(() {
        _isLoading = true;
        _nfoData = null;
        _previewUrls = [];
      });
      _loadData();
    }
  }

  Future<void> _loadData() async {
    try {
      final detail = await getMelatoninMovieByIdForDart(id: widget.movie.id);
      final nfoStr = detail.nfoJson;
      Map<String, dynamic>? parsed;
      if (nfoStr.isNotEmpty) {
        try { parsed = jsonDecode(nfoStr) as Map<String, dynamic>; } catch (_) {}
      }
      // 加载 BT 列表
      List<Map<String, String>> btList = [];
      try {
        final btItems = await getBtListForDart(id: widget.movie.id);
        for (final item in btItems) {
          try {
            final map = jsonDecode(item) as Map<String, dynamic>;
            btList.add({
              'name': (map['name'] ?? '').toString(),
              'tags': (map['tags'] ?? '').toString(),
              'size': (map['size'] ?? '').toString(),
              'magnet': (map['magnet'] ?? '').toString(),
            });
          } catch (_) {}
        }
      } catch (_) {}
      if (!mounted) return;
      setState(() {
        _nfoData = parsed;
        _previewUrls = detail.previewUrls;
        _btList = btList;
        _isLoading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() { _isLoading = false; });
    }
  }

  void _playMovie() {
    final urls = widget.movie.videoUrls;
    if (urls.isEmpty) return;
    _playVideoUrl(urls.first);
  }

  void _playVideoUrl(String url) {
    final fullUrl = url.startsWith('http') ? url : '${AppConfig.instance.serverUrl}$url';
    Get.to(() => UUUVideoPlayerPage.network(fullUrl, headers: AppConfig.instance.getApiHeaders()));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: widget.movie.title),
      body: SafeArea(
        child: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : SingleChildScrollView(
              child: Column(children: [
                // Cover — fills screen width, 3:4 aspect
                _buildCover(),
                // Title — compact
                _buildTitle(),
                // 预览图
                if (_previewUrls.isNotEmpty) _buildPreviews(),
                // Info
                Padding(padding: const EdgeInsets.symmetric(horizontal: 16), child: _buildMelatoninInfo()),
                // BT 列表
                if (_btList.isNotEmpty) Padding(padding: const EdgeInsets.symmetric(horizontal: 16), child: _buildBtList()),
                // Play button
                _buildPlayButton(),
              ]),
            ),
      ),
    );
  }

  Widget _buildCover() {
    final coverUrl = widget.movie.coverUrl;
    if (coverUrl != null && coverUrl.isNotEmpty) {
      final fullUrl = coverUrl.startsWith('http') ? coverUrl : '${AppConfig.instance.serverUrl}$coverUrl';
      return Image.network(fullUrl, fit: BoxFit.contain, width: double.infinity,
        headers: AppConfig.instance.getApiHeaders(),
        errorBuilder: (_, __, ___) => Container(color: Colors.black, child: const Center(child: Icon(Icons.movie, size: 64, color: Colors.grey))),
      );
    }
    return Container(color: Colors.black, child: const Center(child: Icon(Icons.movie, size: 64, color: Colors.grey)));
  }

  Widget _buildTitle() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Text(widget.movie.title, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600), textAlign: TextAlign.center),
    );
  }

  Widget _buildPreviews() {
    return SizedBox(height: 120, child: ListView.separated(
      scrollDirection: Axis.horizontal, padding: const EdgeInsets.symmetric(horizontal: 12),
      itemCount: _previewUrls.length,
      separatorBuilder: (_, __) => const SizedBox(width: 8),
      itemBuilder: (context, index) {
        final fullUrl = _previewUrls[index].startsWith('http') ? _previewUrls[index] : '${AppConfig.instance.serverUrl}${_previewUrls[index]}';
        return GestureDetector(
          onTap: () => _showFullPreview(index),
          child: ClipRRect(borderRadius: BorderRadius.circular(8),
            child: Image.network(fullUrl, width: 160, fit: BoxFit.cover,
              headers: AppConfig.instance.getApiHeaders(),
              errorBuilder: (_, __, ___) => Container(width: 160, color: Colors.grey[800], child: const Icon(Icons.broken_image, color: Colors.grey)),
            ),
          ),
        );
      },
    ));
  }

  void _showFullPreview(int index) {
    showDialog(context: context, builder: (_) => Dialog(
      backgroundColor: Colors.transparent, insetPadding: EdgeInsets.zero,
      child: Stack(children: [
        PageView.builder(
          controller: PageController(initialPage: index), itemCount: _previewUrls.length,
          itemBuilder: (_, i) {
            final fullUrl = _previewUrls[i].startsWith('http') ? _previewUrls[i] : '${AppConfig.instance.serverUrl}${_previewUrls[i]}';
            return InteractiveViewer(child: Center(child: Image.network(fullUrl, fit: BoxFit.contain, headers: AppConfig.instance.getApiHeaders())));
          },
        ),
        Positioned(top: 40, right: 16, child: IconButton(icon: const Icon(Icons.close, color: Colors.white, size: 32), onPressed: () => Get.back())),
      ]),
    ));
  }

  Widget _buildMelatoninInfo() {
    if (_nfoData == null) return const SizedBox.shrink();
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      // 演员 — 第一栏
      if (_nfoData!['actor'] != null) _buildActorRow(_nfoData!['actor'] as List),
      if (_nfoData!['year'] != null) _buildInfoRow('年份', _nfoData!['year'].toString()),
      if (_nfoData!['rating'] != null) _buildInfoRow('评分', _nfoData!['rating'].toString()),
      if (_nfoData!['runtime'] != null) _buildInfoRow('时长', _nfoData!['runtime'].toString()),
      if (_nfoData!['premiered'] != null) _buildInfoRow('上映', _nfoData!['premiered'].toString()),
      if (_nfoData!['genre'] != null) _buildGenreRow(_nfoData!['genre'] as List),
      if (_nfoData!['director'] != null) _buildDirectorRow(_nfoData!['director'] as List),
      if (_nfoData!['plot'] != null || _nfoData!['outline'] != null) _buildPlotRow(),
    ]);
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(children: [
      SizedBox(width: 48, child: Text(label, style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: Colors.grey))),
      Expanded(child: Text(value, style: const TextStyle(fontSize: 13))),
    ]));
  }

  Widget _buildGenreRow(List genres) {
    return Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      SizedBox(width: 48, child: Text('类型', style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: Colors.grey))),
      Expanded(child: Wrap(spacing: 8, runSpacing: 4,
        children: genres.map<Widget>((genre) {
          final g = genre.toString();
          return ActionChip(
            label: Text(g, style: const TextStyle(fontSize: 12)),
            visualDensity: VisualDensity.compact,
            onPressed: () => Get.to(() => GenreMoviesPage(genre: g)),
          );
        }).toList(),
      )),
    ]));
  }

  Widget _buildDirectorRow(List directors) {
    return Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      SizedBox(width: 48, child: Text('导演', style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: Colors.grey))),
      Expanded(child: Text(directors.join(', '), style: const TextStyle(fontSize: 13))),
    ]));
  }

  Widget _buildActorRow(List actors) {
    return Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('演员', style: TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: Colors.grey[600])),
      const SizedBox(height: 4),
      ...actors.map<Widget>((actor) {
        final actorData = actor as Map<String, dynamic>;
        final name = actorData['name'] ?? '';
        final role = actorData['role'];
        return GestureDetector(
          onTap: () => Get.to(() => ActorMoviesPage(actorName: name)),
          child: Padding(padding: const EdgeInsets.symmetric(vertical: 2),
            child: Text(role != null ? '$name ($role)' : name,
              style: const TextStyle(fontSize: 14, color: Colors.blue, decoration: TextDecoration.underline))),
        );
      }),
    ]));
  }

  Widget _buildPlotRow() {
    final plot = _nfoData!['plot'] ?? _nfoData!['outline'];
    if (plot == null) return const SizedBox.shrink();
    return Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('剧情', style: TextStyle(fontWeight: FontWeight.w500, fontSize: 14, color: Colors.grey[600])),
      const SizedBox(height: 4),
      Text(plot.toString(), style: const TextStyle(fontSize: 13, height: 1.5)),
    ]));
  }

  Widget _buildBtList() {
    return Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('BT 列表', style: TextStyle(fontWeight: FontWeight.w500, fontSize: 14, color: Colors.grey[600])),
      const SizedBox(height: 4),
      ..._btList.map((item) {
        final tags = item['tags']!.split('|').where((t) => t.isNotEmpty).toList();
        return Card(
          margin: const EdgeInsets.only(bottom: 6),
          child: Padding(padding: const EdgeInsets.all(10), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Row(children: [
              Expanded(child: Text(item['name']!, style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w500))),
              Text(item['size']!, style: TextStyle(fontSize: 11, color: Colors.grey[500])),
            ]),
            if (tags.isNotEmpty) Padding(padding: const EdgeInsets.only(top: 4), child: Wrap(spacing: 4, runSpacing: 2,
              children: tags.map((t) => Container(padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                decoration: BoxDecoration(color: Colors.blue.withAlpha(30), borderRadius: BorderRadius.circular(4)),
                child: Text(t, style: const TextStyle(fontSize: 10, color: Colors.blue)))).toList(),
            )),
            if (item['magnet']!.isNotEmpty) Padding(padding: const EdgeInsets.only(top: 6), child: GestureDetector(
              onTap: () {
                Clipboard.setData(ClipboardData(text: item['magnet']!));
                ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('磁力链接已复制'), duration: Duration(seconds: 1)));
              },
              child: Row(children: [
                const Icon(Icons.link, size: 14, color: Colors.orange),
                const SizedBox(width: 4),
                Expanded(child: Text(item['magnet']!, style: const TextStyle(fontSize: 11, color: Colors.orange), maxLines: 1, overflow: TextOverflow.ellipsis)),
                const Icon(Icons.copy, size: 14, color: Colors.grey),
              ]),
            )),
          ])),
        );
      }),
    ]));
  }

  Widget _buildPlayButton() {
    final urls = widget.movie.videoUrls;
    if (urls.isEmpty) return const SizedBox.shrink();
    if (urls.length == 1) {
      return Padding(padding: const EdgeInsets.all(16), child: ElevatedButton.icon(
        onPressed: _playMovie, icon: const Icon(Icons.play_arrow, size: 20),
        label: const Text('播放'), style: ElevatedButton.styleFrom(padding: const EdgeInsets.symmetric(vertical: 12)),
      ));
    }
    // 多个视频 — 列表
    return Padding(padding: const EdgeInsets.all(8), child: Wrap(spacing: 8, runSpacing: 4,
      children: urls.asMap().entries.map((e) {
        final name = e.value.split('/').last;
        return ActionChip(
          avatar: const Icon(Icons.play_arrow, size: 16),
          label: Text(name.length > 30 ? '${name.substring(0, 30)}...' : name, style: const TextStyle(fontSize: 12)),
          onPressed: () => _playVideoUrl(e.value),
        );
      }).toList(),
    ));
  }
}
