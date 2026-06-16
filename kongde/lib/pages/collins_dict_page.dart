import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/wifi_api/dict.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class CollinsDictPage extends StatefulWidget {
  final String? initialWord;

  const CollinsDictPage({super.key, this.initialWord});

  @override
  State<CollinsDictPage> createState() => _CollinsDictPageState();
}

class _CollinsDictPageState extends State<CollinsDictPage> {
  final TextEditingController _controller = TextEditingController();
  bool _isLoading = false;
  late final WebViewController _webViewController;
  List<Word> _topWords = [];
  List<WordHistory> _recentHistory = [];
  bool _showHistory = false;

  @override
  void initState() {
    super.initState();
    _webViewController = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setNavigationDelegate(
        NavigationDelegate(
          onProgress: (int progress) {
            debugPrint('WebView loading: $progress%');
          },
          onPageStarted: (String url) {
            debugPrint('Page started loading: $url');
          },
          onPageFinished: (String url) {
            debugPrint('Page finished loading: $url');
          },
          onWebResourceError: (WebResourceError error) {
            debugPrint(
              'WebView resource error: ${error.description} - ${error.errorCode} - URL: ${error.url}',
            );
          },
          onNavigationRequest: (NavigationRequest request) {
            debugPrint('Navigation request: ${request.url}');
            return NavigationDecision.navigate;
          },
        ),
      )
      ..enableZoom(true);

    if (widget.initialWord != null && widget.initialWord!.isNotEmpty) {
      _controller.text = widget.initialWord!;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _searchWord();
      });
    }

    _loadDictData();
  }

  Future<void> _loadDictData() async {
    try {
      await Future.wait([_loadTopWords(), _loadRecentHistory()]);
    } catch (e) {
      debugPrint('加载字典数据失败: $e');
    }
  }

  Future<void> _loadTopWords() async {
    try {
      final words = await getTopWordsForDart();
      setState(() {
        _topWords = words;
      });
    } catch (e) {
      debugPrint('加载高频单词失败: $e');
    }
  }

  Future<void> _loadRecentHistory() async {
    try {
      final history = await getRecentHistoryForDart(
        limit: PlatformInt64Util.from(2),
      );
      setState(() {
        _recentHistory = history;
      });
    } catch (e) {
      debugPrint('加载最近查询失败: $e');
    }
  }

  Future<void> _searchWord() async {
    var word = _controller.text.trim();

    if (word.isEmpty) {
      final clipboardData = await Clipboard.getData(Clipboard.kTextPlain);
      if (clipboardData != null &&
          clipboardData.text != null &&
          clipboardData.text!.isNotEmpty) {
        word = clipboardData.text!;
        _controller.text = word;
      } else {
        return;
      }
    }

    setState(() {
      _isLoading = true;
    });

    try {
      final htmlContent = await searchCollinsForDart(word: word);
      _webViewController.loadHtmlString('''
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
  $htmlContent
</body>
</html>
        ''');
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('common.searchError'.trParams({'error': '$e'}))));
      }
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'collins.title'.tr),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(16.0),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      decoration: InputDecoration(
                        hintText: 'collins.inputHint'.tr,
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(8),
                        ),
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                          vertical: 12,
                        ),
                      ),
                      onSubmitted: (_) => _searchWord(),
                    ),
                  ),
                  const SizedBox(width: 12),
                  ElevatedButton(
                    onPressed: _isLoading ? null : _searchWord,
                    child: _isLoading
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : Text('common.search'.tr),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton(
                    onPressed: () {
                      setState(() {
                        _showHistory = !_showHistory;
                      });
                    },
                    child: Text(_showHistory ? 'common.hide'.tr : 'common.history'.tr),
                  ),
                ],
              ),
            ),
            if (_showHistory)
              Container(
                height: 200,
                padding: const EdgeInsets.all(16),
                child: DefaultTabController(
                  length: 2,
                  child: Column(
                    children: [
                      TabBar(
                        tabs: [
                          Tab(text: 'common.highFrequency'.tr),
                          Tab(text: 'common.recentSearch'.tr),
                        ],
                      ),
                      Expanded(
                        child: TabBarView(
                          children: [
                            _buildTopWordsTab(),
                            _buildRecentHistoryTab(),
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            Expanded(child: WebViewWidget(controller: _webViewController)),
          ],
        ),
      ),
    );
  }

  Widget _buildTopWordsTab() {
    if (_topWords.isEmpty) {
      return Center(child: Text('common.noHighFrequencyData'.tr));
    }

    return ListView.builder(
      itemCount: _topWords.length,
      itemBuilder: (context, index) {
        final word = _topWords[index];
        return ListTile(
          title: Text(word.word),
          subtitle: Text('common.searchCount'.trParams({'count': '${word.hasSearchedTimes}'})),
          onTap: () {
            _controller.text = word.word;
            _searchWord();
          },
        );
      },
    );
  }

  Widget _buildRecentHistoryTab() {
    if (_recentHistory.isEmpty) {
      return Center(child: Text('common.noRecentData'.tr));
    }

    return ListView.builder(
      itemCount: _recentHistory.length,
      itemBuilder: (context, index) {
        final history = _recentHistory[index];
        return ListTile(
          title: Text(history.word),
          subtitle: Text(history.time.toString()),
          onTap: () {
            _controller.text = history.word;
            _searchWord();
          },
        );
      },
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
