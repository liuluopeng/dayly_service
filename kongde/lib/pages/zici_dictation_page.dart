import 'package:http/http.dart' as http;
import 'package:flutter/material.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/src/rust/api/zici.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ZiciDictationPage extends StatefulWidget {
  const ZiciDictationPage({super.key});

  @override
  State<ZiciDictationPage> createState() => _ZiciDictationPageState();
}

class _ZiciDictationPageState extends State<ZiciDictationPage> {
  int _grade = 1;
  int _term = 1;
  List<String> _chars = [];
  int _index = 0;
  final TextEditingController _input = TextEditingController();
  bool _checked = false;
  bool _correct = false;
  String _pinyin = '';
  bool _loading = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    final chars = await ziciNewChars(grade: _grade, term: _term);
    if (mounted) {
      setState(() {
        _chars = chars;
        _index = 0;
        _pinyin = '';
        _checked = false;
        _loading = false;
      });
      if (chars.isNotEmpty) await _fetchPinyin();
    }
  }

  /// 后端拼音接口（替代 cnchar）
  Future<void> _fetchPinyin() async {
    if (_chars.isEmpty) return;
    final char = _chars[_index];
    try {
      final url = '${AppConfig.instance.serverUrl}/api/pinyin/get-by-ori?ori=$char';
      final res = await httpGetJson(url);
      if (mounted) setState(() => _pinyin = res ?? '');
    } catch (_) {}
  }

  Future<String?> httpGetJson(String url) async {
    final res = await http.get(Uri.parse(url)).timeout(const Duration(seconds: 5));
    if (res.statusCode != 200) return null;
    final list = (res.body.replaceAll('[', '').replaceAll(']', '').replaceAll('"', '').split(','))
        .where((s) => s.trim().isNotEmpty)
        .toList();
    return list.isEmpty ? null : list.join(' ');
  }

  void _check() {
    final input = _input.text.trim();
    setState(() {
      _checked = true;
      _correct = input == _chars[_index];
    });
  }

  void _next() {
    setState(() {
      _checked = false;
      _input.clear();
      _index = (_index + 1) % _chars.length;
      _pinyin = '';
    });
    _fetchPinyin();
  }

  @override
  void dispose() {
    _input.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '听写'),
      body: SafeArea(
        child: _loading
            ? const Center(child: CircularProgressIndicator())
            : _chars.isEmpty
                ? const Center(child: Text('无数据', style: TextStyle(color: Colors.grey)))
                : Column(
                    children: [
                      Padding(
                        padding: const EdgeInsets.all(12),
                        child: Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          alignment: WrapAlignment.center,
                          children: [
                            for (var g = 1; g <= 6; g++)
                              for (var t = 1; t <= 2; t++)
                                ChoiceChip(
                                  label: Text('${g}年级${t == 1 ? '上' : '下'}'),
                                  selected: _grade == g && _term == t,
                                  onSelected: (_) {
                                    _grade = g;
                                    _term = t;
                                    _load();
                                  },
                                ),
                          ],
                        ),
                      ),
                      Expanded(
                        child: Center(
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Text('第 ${_index + 1}/${_chars.length} 字', style: const TextStyle(color: Colors.grey)),
                              const SizedBox(height: 16),
                              Text(
                                _pinyin.isEmpty ? _chars[_index] : _pinyin,
                                style: TextStyle(
                                  fontSize: _pinyin.isEmpty ? 72 : 40,
                                  fontWeight: FontWeight.bold,
                                  color: _pinyin.isEmpty ? Colors.black : Colors.blueGrey,
                                ),
                              ),
                              if (_pinyin.isEmpty) ...[
                                const SizedBox(height: 8),
                                const Text('（点击下方显示拼音）', style: TextStyle(color: Colors.grey, fontSize: 12)),
                              ],
                              const SizedBox(height: 24),
                              SizedBox(
                                width: 200,
                                child: TextField(
                                  controller: _input,
                                  textAlign: TextAlign.center,
                                  style: const TextStyle(fontSize: 24),
                                  decoration: const InputDecoration(
                                    hintText: '输入汉字',
                                    border: OutlineInputBorder(),
                                  ),
                                  onSubmitted: (_) => _check(),
                                ),
                              ),
                              const SizedBox(height: 16),
                              if (_checked)
                                Text(
                                  _correct ? '✓ 正确' : '✗ 错误，答案是 ${_chars[_index]}',
                                  style: TextStyle(
                                    fontSize: 18,
                                    color: _correct ? Colors.green : Colors.red,
                                  ),
                                ),
                              const SizedBox(height: 16),
                              Row(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  OutlinedButton(
                                    onPressed: () {
                                      setState(() => _pinyin = _pinyin.isEmpty ? _chars[_index] : '');
                                    },
                                    child: Text(_pinyin.isEmpty ? '显示拼音' : '隐藏拼音'),
                                  ),
                                  const SizedBox(width: 12),
                                  FilledButton(onPressed: _checked ? _next : _check, child: Text(_checked ? '下一个' : '检查')),
                                ],
                              ),
                            ],
                          ),
                        ),
                      ),
                    ],
                  ),
      ),
    );
  }
}

