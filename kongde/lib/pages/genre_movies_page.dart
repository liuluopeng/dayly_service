import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/pages/melatonin_detail_page.dart';
import 'package:kongde/src/rust/api/wifi_api/melatonin.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class GenreMoviesPage extends StatefulWidget {
  final String genre;

  const GenreMoviesPage({super.key, required this.genre});

  @override
  State<GenreMoviesPage> createState() => _GenreMoviesPageState();
}

class _GenreMoviesPageState extends State<GenreMoviesPage> {
  List<MelatoninMovieListForDart> _movies = [];
  bool _isLoading = false;
  bool _isLoadingMore = false;
  int _pageSize = 20;
  int _currentPage = 0;
  bool _hasMore = true;
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    _loadPage(0);
  }

  @override
  void dispose() {
    _scrollController.removeListener(_onScroll);
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (_scrollController.position.pixels >= _scrollController.position.maxScrollExtent - 500 && !_isLoadingMore && _hasMore) {
      _loadMore();
    }
  }

  Future<void> _loadPage(int page) async {
    setState(() { _isLoading = true; });
    try {
      final response = await getMoviesByGenreForDart(genre: widget.genre, page: page, pageSize: _pageSize);
      setState(() {
        _movies = response.data;
        _currentPage = page;
        _isLoading = false;
        _hasMore = (page + 1) * _pageSize < response.total;
      });
    } catch (_) { setState(() { _isLoading = false; }); }
  }

  Future<void> _loadMore() async {
    if (_isLoadingMore || !_hasMore) return;
    setState(() { _isLoadingMore = true; });
    try {
      final nextPage = _currentPage + 1;
      final response = await getMoviesByGenreForDart(genre: widget.genre, page: nextPage, pageSize: _pageSize);
      setState(() {
        _movies.addAll(response.data);
        _currentPage = nextPage;
        _isLoadingMore = false;
        _hasMore = (nextPage + 1) * _pageSize < response.total;
      });
    } catch (_) { setState(() { _isLoadingMore = false; }); }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: widget.genre),
      body: SafeArea(child: _isLoading ? const Center(child: CircularProgressIndicator())
        : _movies.isEmpty ? Center(child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
            const Icon(Icons.movie_outlined, size: 64, color: Colors.grey), const SizedBox(height: 16),
            Text('暂无此类型的电影', style: TextStyle(color: Colors.grey[500])),
          ]))
        : GridView.builder(controller: _scrollController, padding: const EdgeInsets.all(8),
            gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: 3, childAspectRatio: 0.7, crossAxisSpacing: 8, mainAxisSpacing: 8),
            itemCount: _movies.length + (_isLoadingMore ? 1 : 0),
            itemBuilder: (context, index) {
              if (index >= _movies.length) return const Center(child: Padding(padding: EdgeInsets.all(16), child: CircularProgressIndicator()));
              return _buildCard(_movies[index]);
            })),
    );
  }

  Widget _buildCard(MelatoninMovieListForDart movie) {
    final coverUrl = movie.coverUrl;
    final hasVideo = movie.videoUrls.isNotEmpty;
    return Card(clipBehavior: Clip.antiAlias, shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
      child: InkWell(onTap: () => Get.to(() => MelatoninDetailPage(movie: movie)), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: Stack(children: [
          if (coverUrl != null && coverUrl.isNotEmpty)
            Image.network(coverUrl.startsWith('http') ? coverUrl : '${AppConfig.instance.serverUrl}$coverUrl',
              fit: BoxFit.contain, width: double.infinity, height: double.infinity,
              headers: AppConfig.instance.getApiHeaders(),
              errorBuilder: (_, __, ___) => Container(color: Colors.grey[300], child: const Center(child: Icon(Icons.movie, size: 48, color: Colors.grey))),
            )
          else Container(color: Colors.grey[300], child: const Center(child: Icon(Icons.movie, size: 48, color: Colors.grey))),
          Positioned(top: 2, right: 2, child: Container(padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(color: hasVideo ? Colors.green.withAlpha(200) : Colors.orange.withAlpha(200), borderRadius: BorderRadius.circular(4)),
            child: Text(hasVideo ? '已下载' : '未下载', style: const TextStyle(color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold)))),
        ])),
        Padding(padding: const EdgeInsets.all(8), child: Text(movie.title, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500), maxLines: 2, overflow: TextOverflow.ellipsis, textAlign: TextAlign.center)),
      ])),
    );
  }
}
