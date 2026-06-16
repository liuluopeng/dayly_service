import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/pages/melatonin_detail_page.dart';
import 'package:kongde/src/rust/api/wifi_api/melatonin.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ActorMoviesPage extends StatefulWidget {
  final String actorName;

  const ActorMoviesPage({super.key, required this.actorName});

  @override
  State<ActorMoviesPage> createState() => _ActorMoviesPageState();
}

class _ActorMoviesPageState extends State<ActorMoviesPage>
    with AutomaticKeepAliveClientMixin {
  List<MelatoninMovieListForDart> _movies = [];
  bool _isLoading = false;
  bool _isLoadingMore = false;
  String? _errorMessage;
  int _pageSize = 20;
  int _currentPage = 0;
  bool _hasMore = true;
  final ScrollController _scrollController = ScrollController();

  @override
  bool get wantKeepAlive => true;

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
    if (_scrollController.position.pixels >=
            _scrollController.position.maxScrollExtent - 500 &&
        !_isLoadingMore &&
        _hasMore) {
      _loadMore();
    }
  }

  Future<PaginatedResponseForDart> _fetchActorMovies(
    String actor,
    int page,
    int pageSize,
  ) async {
    return await getMoviesByActorForDart(
      actor: actor,
      page: page,
      pageSize: pageSize,
    );
  }

  void _onMovieTap(MelatoninMovieListForDart movie) {
    Get.to(() => MelatoninDetailPage(movie: movie));
  }

  Future<void> _loadPage(int page) async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final response = await _fetchActorMovies(
        widget.actorName,
        page,
        _pageSize,
      );
      final movies = response.data;
      final total = response.total.toInt();

      setState(() {
        _movies = movies;
        _currentPage = page;
        _isLoading = false;
        _hasMore = (page + 1) * _pageSize < total;
      });
    } catch (e) {
      setState(() {
        _errorMessage = 'actorMovies.loadFailed'.trParams({'error': '$e'});
        _isLoading = false;
      });
    }
  }

  Future<void> _loadMore() async {
    if (_isLoadingMore || !_hasMore) return;

    setState(() {
      _isLoadingMore = true;
    });

    try {
      final nextPage = _currentPage + 1;
      final response = await _fetchActorMovies(
        widget.actorName,
        nextPage,
        _pageSize,
      );
      final movies = response.data;
      final total = response.total.toInt();

      setState(() {
        _movies.addAll(movies);
        _currentPage = nextPage;
        _isLoadingMore = false;
        _hasMore = (nextPage + 1) * _pageSize < total;
      });
    } catch (e) {
      setState(() {
        _isLoadingMore = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'actorMovies.title'.trParams({'name': widget.actorName})),
      body: SafeArea(
        child: _isLoading
            ? const Center(child: CircularProgressIndicator())
            : _errorMessage != null
            ? Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(
                      Icons.error_outline,
                      size: 48,
                      color: Colors.red,
                    ),
                    const SizedBox(height: 16),
                    Text(_errorMessage!),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: () => _loadPage(_currentPage),
                      child: Text('common.retry'.tr),
                    ),
                  ],
                ),
              )
            : _movies.isEmpty
            ? Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(
                      Icons.movie_outlined,
                      size: 64,
                      color: Colors.grey,
                    ),
                    const SizedBox(height: 16),
                    Text('actorMovies.noMovies'.trParams({'name': widget.actorName})),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: () => _loadPage(0),
                      child: Text('common.refresh'.tr),
                    ),
                  ],
                ),
              )
            : GridView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.all(8),
                gridDelegate:
                    const SliverGridDelegateWithFixedCrossAxisCount(
                      crossAxisCount: 3,
                      childAspectRatio: 0.7,
                      crossAxisSpacing: 8,
                      mainAxisSpacing: 8,
                    ),
                itemCount: _movies.length + (_isLoadingMore ? 1 : 0),
                itemBuilder: (context, index) {
                  if (index >= _movies.length) {
                    return const Center(
                      child: Padding(
                        padding: EdgeInsets.all(16),
                        child: CircularProgressIndicator(),
                      ),
                    );
                  }
                  return _buildMovieCard(_movies[index]);
                },
              ),
      ),
    );
  }

  Widget _buildMovieCard(MelatoninMovieListForDart movie) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: () => _onMovieTap(movie),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(child: _buildCoverImage(movie)),
            Padding(
              padding: const EdgeInsets.all(8.0),
              child: Text(
                movie.title,
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                textAlign: TextAlign.center,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCoverImage(MelatoninMovieListForDart movie) {
    final coverUrl = movie.coverUrl;
    if (coverUrl != null && coverUrl.isNotEmpty) {
      final fullUrl = coverUrl.startsWith('http')
          ? coverUrl
          : '${AppConfig.instance.serverUrl}$coverUrl';
      return Image.network(
        fullUrl,
        fit: BoxFit.cover,
        headers: AppConfig.instance.getApiHeaders(),
        errorBuilder: (context, error, stackTrace) {
          return _buildPlaceholder();
        },
      );
    }
    return _buildPlaceholder();
  }

  Widget _buildPlaceholder() {
    return Container(
      color: Colors.grey[300],
      child: const Center(
        child: Icon(Icons.movie, size: 48, color: Colors.grey),
      ),
    );
  }
}
