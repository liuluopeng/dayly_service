import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/games.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class Game2048Page extends StatefulWidget {
  const Game2048Page({super.key});

  @override
  State<Game2048Page> createState() => _Game2048PageState();
}

class _Game2048PageState extends State<Game2048Page> {
  Game2048State _state = Game2048State(board: Uint32List(0), score: BigInt.zero, over: false);

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    await game2048New();
    _refresh();
  }

  Future<void> _refresh() async {
    final s = await game2048Get();
    if (mounted) setState(() => _state = s);
  }

  Future<void> _move(String dir) async {
    if (_state.over) return;
    final s = await game2048Move(dir: dir);
    if (mounted) setState(() => _state = s);
  }

  Future<void> _restart() async {
    await game2048New();
    _refresh();
  }

  @override
  Widget build(BuildContext context) {
    final colors = <int, Color>{
      0: Colors.grey.shade200,
      2: Colors.amber.shade50,
      4: Colors.amber.shade100,
      8: Colors.orange.shade200,
      16: Colors.orange.shade300,
      32: Colors.deepOrange.shade300,
      64: Colors.deepOrange.shade400,
      128: Colors.red.shade300,
      256: Colors.red.shade400,
      512: Colors.purple.shade300,
      1024: Colors.purple.shade400,
      2048: Colors.indigo.shade400,
    };

    return Scaffold(
      appBar: CommonAppBar(title: '2048'),
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
                  color: Colors.grey.shade300,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: GridView.builder(
                  physics: const NeverScrollableScrollPhysics(),
                  padding: const EdgeInsets.all(4),
                  gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: 4,
                    mainAxisSpacing: 4,
                    crossAxisSpacing: 4,
                  ),
                  itemCount: _state.board.length,
                  itemBuilder: (context, i) {
                    final v = _state.board[i];
                    return Container(
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        color: colors[v.toInt()] ?? Colors.grey.shade400,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: v == 0
                          ? null
                          : Text(
                              '${v.toInt()}',
                              style: TextStyle(
                                fontSize: v >= 100 ? 18 : 24,
                                fontWeight: FontWeight.bold,
                                color: v > 128 ? Colors.white : Colors.black87,
                              ),
                            ),
                    );
                  },
                ),
              ),
              if (_state.over) ...[
                const SizedBox(height: 16),
                const Text('游戏结束', style: TextStyle(fontSize: 22, color: Colors.red)),
              ],
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
              TextButton(
                onPressed: () async {
                  final s = await game2048Undo();
                  if (mounted) setState(() => _state = s);
                },
                child: const Text('撤销'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
