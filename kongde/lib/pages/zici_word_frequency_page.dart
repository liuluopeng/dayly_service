import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/zici.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ZiciWordFrequencyPage extends StatefulWidget {
  const ZiciWordFrequencyPage({super.key});

  @override
  State<ZiciWordFrequencyPage> createState() => _ZiciWordFrequencyPageState();
}

class _ZiciWordFrequencyPageState extends State<ZiciWordFrequencyPage> {
  List<WordFrequencyEntry> _items = [];
  String _query = '';
  bool _loading = false;
  bool _searched = false;

  Future<void> _search([String? q]) async {
    setState(() {
      _loading = true;
      _searched = true;
    });
    final items = await ziciWordFrequencySearch(query: q ?? _query, limit: 200);
    if (mounted) setState(() {
      _items = items;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '词频'),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      decoration: const InputDecoration(
                        hintText: '搜索词语（含解释）...',
                        prefixIcon: Icon(Icons.search),
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (v) => _query = v,
                      onSubmitted: (_) => _search(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(onPressed: _loading ? null : _search, child: const Text('搜索')),
                ],
              ),
            ),
            Expanded(
              child: _loading
                  ? const Center(child: CircularProgressIndicator())
                  : !_searched
                      ? const Center(child: Text('输入词语搜索词频与解释', style: TextStyle(color: Colors.grey)))
                      : _items.isEmpty
                          ? const Center(child: Text('无匹配', style: TextStyle(color: Colors.grey)))
                          : ListView.builder(
                              itemCount: _items.length,
                              itemBuilder: (context, i) {
                                final e = _items[i];
                                final freq = e.frequency;
                                return ListTile(
                                  title: Text('${e.word}  ${e.pinyin}', style: const TextStyle(fontSize: 17)),
                                  subtitle: Text(
                                    e.explanation,
                                    maxLines: 3,
                                    overflow: TextOverflow.ellipsis,
                                    style: const TextStyle(fontSize: 12, color: Colors.black54),
                                  ),
                                  trailing: Text(
                                    freq > 0 ? (1 - freq / 56000).toStringAsFixed(4) : '',
                                    style: const TextStyle(fontSize: 11, color: Colors.grey),
                                  ),
                                );
                              },
                            ),
            ),
          ],
        ),
      ),
    );
  }
}
