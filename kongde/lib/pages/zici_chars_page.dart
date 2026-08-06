import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/zici.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ZiciCharsPage extends StatefulWidget {
  const ZiciCharsPage({super.key});

  @override
  State<ZiciCharsPage> createState() => _ZiciCharsPageState();
}

class _ZiciCharsPageState extends State<ZiciCharsPage> {
  int _grade = 1;
  int _term = 1;
  List<String> _chars = [];
  bool _loading = false;
  bool _showPinyin = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _loading = true);
    final chars = await ziciNewChars(grade: _grade, term: _term);
    if (mounted) setState(() {
      _chars = chars;
      _loading = false;
    });
  }

  void _select(int grade, int term) {
    setState(() {
      _grade = grade;
      _term = term;
    });
    _load();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '生字表'),
      body: SafeArea(
        child: Column(
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
                        onSelected: (_) => _select(g, t),
                      ),
                ],
              ),
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text('共 ${_chars.length} 字', style: const TextStyle(color: Colors.grey)),
                const SizedBox(width: 12),
                const Text('注音', style: TextStyle(color: Colors.grey)),
                Switch(
                  value: _showPinyin,
                  onChanged: (v) => setState(() => _showPinyin = v),
                ),
              ],
            ),
            Expanded(
              child: _loading
                  ? const Center(child: CircularProgressIndicator())
                  : GridView.builder(
                      padding: const EdgeInsets.all(16),
                      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: 5,
                        mainAxisSpacing: 8,
                        crossAxisSpacing: 8,
                      ),
                      itemCount: _chars.length,
                      itemBuilder: (context, i) {
                        final c = _chars[i];
                        return Card(
                          child: Center(
                            child: Text(
                              c,
                              style: const TextStyle(fontSize: 28, fontWeight: FontWeight.bold),
                            ),
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
