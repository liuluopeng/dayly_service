import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/games.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class GameSnakePage extends StatefulWidget {
  const GameSnakePage({super.key});

  @override
  State<GameSnakePage> createState() => _GameSnakePageState();
}

class _GameSnakePageState extends State<GameSnakePage> {
  SnakeState _state = SnakeState(cells: Uint8List(0), score: 0, over: false);
  Timer? _timer;
  static const int _size = 20;

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    await snakeNew();
    _refresh();
    _startTimer();
  }

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(milliseconds: 200), (_) async {
      final s = await snakeTick();
      if (!mounted) return;
      setState(() => _state = s);
      if (_state.over) _timer?.cancel();
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  Future<void> _refresh() async {
    final s = await snakeGet();
    if (mounted) setState(() => _state = s);
  }

  void _setDir(String dir) {
    snakeSetDir(dir: dir);
  }

  Future<void> _restart() async {
    await snakeNew();
    _refresh();
    _startTimer();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '贪吃蛇'),
      body: SafeArea(
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text('得分: ${_state.score}', style: const TextStyle(fontSize: 20, fontWeight: FontWeight.bold)),
                  const SizedBox(width: 24),
                  FilledButton(onPressed: _restart, child: const Text('重新开始')),
                ],
              ),
              const SizedBox(height: 16),
              Container(
                width: 320,
                height: 320,
                decoration: BoxDecoration(
                  color: Colors.grey.shade100,
                  border: Border.all(color: Colors.grey.shade400),
                ),
                child: _state.cells.isEmpty
                    ? const SizedBox()
                    : GridView.builder(
                        physics: const NeverScrollableScrollPhysics(),
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: _size,
                          mainAxisSpacing: 0,
                          crossAxisSpacing: 0,
                        ),
                        itemCount: _state.cells.length,
                        itemBuilder: (context, i) {
                          final v = _state.cells[i];
                          return Container(
                            color: v == 1
                                ? Colors.green.shade600
                                : v == 2
                                    ? Colors.red.shade400
                                    : Colors.transparent,
                          );
                        },
                      ),
              ),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_up), onPressed: () => _setDir('up')),
                ],
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_left), onPressed: () => _setDir('left')),
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_down), onPressed: () => _setDir('down')),
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_right), onPressed: () => _setDir('right')),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
