import 'dart:async';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/games.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class GameTetrisPage extends StatefulWidget {
  const GameTetrisPage({super.key});

  @override
  State<GameTetrisPage> createState() => _GameTetrisPageState();
}

class _GameTetrisPageState extends State<GameTetrisPage> {
  TetrisState _state = TetrisState(board: Uint8List(0), score: 0, over: false);
  Timer? _timer;
  static const int _width = 10;
  static const int _height = 20;

  static const _pieceColors = <Color>[
    Colors.cyan,
    Colors.blue,
    Colors.orange,
    Colors.yellow,
    Colors.green,
    Colors.purple,
    Colors.red,
  ];

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    await tetrisNew();
    _refresh();
    _startTimer();
  }

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(milliseconds: 500), (_) async {
      final s = await tetrisTick();
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
    final s = await tetrisGet();
    if (mounted) setState(() => _state = s);
  }

  Future<void> _move(String dir) async {
    if (_state.over) return;
    final s = await tetrisMove(dir: dir);
    if (mounted) setState(() => _state = s);
  }

  Future<void> _restart() async {
    await tetrisNew();
    _refresh();
    _startTimer();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: '俄罗斯方块'),
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
                width: 200,
                height: 400,
                decoration: BoxDecoration(
                  color: Colors.grey.shade100,
                  border: Border.all(color: Colors.grey.shade400),
                ),
                child: _state.board.isEmpty
                    ? const SizedBox()
                    : GridView.builder(
                        physics: const NeverScrollableScrollPhysics(),
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: _width,
                          mainAxisSpacing: 0,
                          crossAxisSpacing: 0,
                        ),
                        itemCount: _state.board.length,
                        itemBuilder: (context, i) {
                          final v = _state.board[i];
                          return Container(
                            color: v == 0
                                ? Colors.transparent
                                : _pieceColors[(v - 1) % _pieceColors.length],
                          );
                        },
                      ),
              ),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_up), onPressed: () => _move('up')),
                ],
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_left), onPressed: () => _move('left')),
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_down), onPressed: () => _move('down')),
                  IconButton(iconSize: 48, icon: const Icon(Icons.keyboard_arrow_right), onPressed: () => _move('right')),
                ],
              ),
              if (_state.over) const Text('游戏结束', style: TextStyle(fontSize: 18, color: Colors.red)),
            ],
          ),
        ),
      ),
    );
  }
}
