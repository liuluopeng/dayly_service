import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:kongde/src/rust/api/wifi_api/dict.dart';
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class UnifiedDictPage extends StatefulWidget {
  const UnifiedDictPage({super.key});

  @override
  State<UnifiedDictPage> createState() => _UnifiedDictPageState();
}

class _UnifiedDictPageState extends State<UnifiedDictPage> {
  final _controller = TextEditingController();
  bool _isLoading = false;
  final _webController = WebViewController()..setJavaScriptMode(JavaScriptMode.unrestricted);
  String _currentHtml = '';
  String _currentLabel = '';
  List<Word> _topWords = [];
  List<WordHistory> _recentHistory = [];
  int _searchCount = 0;
  double _lastScale = 1.0;

  @override
  void initState() {
    super.initState();
    _loadStats();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final scale = MediaQuery.textScaleFactorOf(context);
    if ((scale - _lastScale).abs() > 0.01) {
      _lastScale = scale;
      _updateWebViewZoom(scale);
    }
  }

  void _updateWebViewZoom(double scale) {
    _webController.runJavaScript('''
      document.body.style.zoom = '$scale';
      document.querySelectorAll('img, table, pre, code').forEach(el => {
        el.style.maxWidth = '${100 / scale}%';
      });
    ''');
  }

  @override
  void dispose() { _controller.dispose(); super.dispose(); }

  Future<void> _loadStats() async {
    try {
      final [freq, hist] = await Future.wait([
        getTopWordsForDart(),
        getRecentHistoryForDart(limit: PlatformInt64Util.from(20)),
      ]);
      setState(() { _topWords = freq; _recentHistory = hist; });
    } catch (_) {}
  }

  Future<void> _searchAll() async {
    var word = _controller.text.trim();
    if (word.isEmpty) {
      final clip = await Clipboard.getData(Clipboard.kTextPlain);
      if (clip?.text != null && clip!.text!.isNotEmpty) {
        word = clip.text!;
        _controller.text = word;
      } else { return; }
    }

    setState(() => _isLoading = true);

    try {
      final results = await Future.wait([
        searchXianzaihanyuForDart(word: word).then((v) => ('现代汉语', v)),
        searchCollinsForDart(word: word).then((v) => ('Collins', v)),
        searchLdoceForDart(word: word).then((v) => ('LDOCE', v)),
      ]);

      // 选第一个有结果的
      String? html;
      String label = '';
      for (final (l, h) in results) {
        if (h.isNotEmpty && !h.startsWith('error')) {
          html = h; label = l; break;
        }
      }

      if (html != null && html.isNotEmpty) {
        _webController.loadHtmlString('''
<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"></head><body>$html</body></html>''');
        // 等页面加载完再应用缩放
        _webController.setNavigationDelegate(NavigationDelegate(
          onPageFinished: (_) => _updateWebViewZoom(_lastScale),
        ));
        setState(() { _currentHtml = html!; _currentLabel = label; });
      }
    } catch (e) {
      if (mounted) Get.snackbar('错误', '$e', snackPosition: SnackPosition.bottom);
    }

    // 刷新统计
    await _loadStats();
    setState(() {
      _isLoading = false;
      _searchCount = _topWords.where((w) => w.word == word).map((w) => w.hasSearchedTimes).firstOrNull ?? 1;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'unifiedDict.title'.tr),
      body: SafeArea(child: Column(children: [
        // 搜索栏
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 12, 12, 4),
          child: Row(children: [
            Expanded(child: TextField(
              controller: _controller,
              decoration: InputDecoration(
                hintText: 'unifiedDict.hint'.tr,
                isDense: true, contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                border: OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
              ),
              onSubmitted: (_) => _searchAll(),
            )),
            const SizedBox(width: 8),
            ElevatedButton(
              onPressed: _isLoading ? null : _searchAll,
              child: _isLoading
                ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2))
                : Text('common.search'.tr),
            ),
          ]),
        ),

        // 搜索次数
        if (_searchCount > 0 && _currentLabel.isNotEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Text('unifiedDict.searchCount'.trParams({'word': _controller.text, 'count': '$_searchCount'}),
              style: TextStyle(fontSize: 11, color: Colors.grey.shade500)),
          ),

        // 来源标签
        if (_currentLabel.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 4, 12, 0),
            child: Align(alignment: Alignment.centerLeft,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(color: Colors.blue.shade50, borderRadius: BorderRadius.circular(4)),
                child: Text(_currentLabel, style: TextStyle(fontSize: 11, color: Colors.blue.shade700)),
              ),
            ),
          ),

        // 统计面板（初始状态）
        if (_currentLabel.isEmpty && !_isLoading)
          Expanded(child: _buildStatsPane())
        else
          Expanded(child: WebViewWidget(controller: _webController)),
      ])),
    );
  }

  Widget _buildStatsPane() {
    return DefaultTabController(
      length: 2,
      child: Column(children: [
        TabBar(tabs: [
          Tab(text: 'unifiedDict.frequency'.tr),
          Tab(text: 'unifiedDict.history'.tr),
        ]),
        Expanded(child: TabBarView(children: [
          _buildTopWords(),
          _buildRecentHistory(),
        ])),
      ]),
    );
  }

  Widget _buildTopWords() {
    if (_topWords.isEmpty) return Center(child: Text('common.noData'.tr));
    final max = _topWords.map((w) => w.hasSearchedTimes).reduce((a, b) => a > b ? a : b);
    return ListView.builder(
      itemCount: _topWords.length,
      itemBuilder: (_, i) {
        final w = _topWords[i];
        return ListTile(
          dense: true, title: Text(w.word),
          trailing: Text('${w.hasSearchedTimes}', style: TextStyle(fontSize: 12, color: Colors.grey.shade500)),
          onTap: () { _controller.text = w.word; _searchAll(); },
        );
      },
    );
  }

  Widget _buildRecentHistory() {
    if (_recentHistory.isEmpty) return Center(child: Text('common.noData'.tr));
    return ListView.builder(
      itemCount: _recentHistory.length,
      itemBuilder: (_, i) {
        final h = _recentHistory[i];
        return ListTile(
          dense: true,
          title: Text(h.word),
          subtitle: Text('${h.time.year}-${h.time.month.toString().padLeft(2,'0')}-${h.time.day.toString().padLeft(2,'0')}',
            style: TextStyle(fontSize: 11, color: Colors.grey.shade400)),
          onTap: () { _controller.text = h.word; _searchAll(); },
        );
      },
    );
  }
}
