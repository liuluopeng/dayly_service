import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/zici.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ZiciWordsPage extends StatefulWidget {
  const ZiciWordsPage({super.key});

  @override
  State<ZiciWordsPage> createState() => _ZiciWordsPageState();
}

class _ZiciWordsPageState extends State<ZiciWordsPage> {
  List<String> _allWords = [];
  List<String> _filtered = [];
  String _query = ''; // ignore: unused_field
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final words = await ziciNewWords(query: "");
    if (mounted) {
      setState(() {
        _allWords = words;
        _filtered = words;
        _loading = false;
      });
    }
  }

  void _onQuery(String q) {
    setState(() {
      _query = q;
      if (q.isEmpty) {
        _filtered = _allWords;
      } else {
        _filtered = _allWords.where((w) => w.contains(q)).toList();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '生词表'),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(12),
              child: TextField(
                decoration: const InputDecoration(
                  hintText: '搜索生词...',
                  prefixIcon: Icon(Icons.search),
                  border: OutlineInputBorder(),
                ),
                onChanged: _onQuery,
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text('共 ${_filtered.length} 个', style: const TextStyle(color: Colors.grey)),
              ),
            ),
            Expanded(
              child: _loading
                  ? const Center(child: CircularProgressIndicator())
                  : _filtered.isEmpty
                      ? const Center(child: Text('无匹配', style: TextStyle(color: Colors.grey)))
                      : ListView.builder(
                          itemCount: _filtered.length,
                          itemBuilder: (context, i) => ListTile(
                            leading: const Icon(Icons.abc, color: Colors.blueGrey),
                            title: Text(_filtered[i], style: const TextStyle(fontSize: 18)),
                          ),
                        ),
            ),
          ],
        ),
      ),
    );
  }
}
