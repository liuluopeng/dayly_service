import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:kongde/src/rust/api/games.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class GameMinesweeperPage extends StatefulWidget {
  const GameMinesweeperPage({super.key});

  @override
  State<GameMinesweeperPage> createState() => _GameMinesweeperPageState();
}

class _GameMinesweeperPageState extends State<GameMinesweeperPage> {
  MinesweeperState _state = MinesweeperState(
    cells: Uint8List(0), revealed: [], flagged: [], over: false, won: false,
  );
  static const int _size = 20;

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    await minesweeperNew();
    _refresh();
  }

  Future<void> _refresh() async {
    final s = await minesweeperGet();
    if (mounted) setState(() => _state = s);
  }

  Future<void> _restart() async {
    await minesweeperNew();
    _refresh();
  }

  Future<void> _click(int i) async {
    if (_state.over || _state.won) return;
    final s = await minesweeperClick(x: BigInt.from(i % _size), y: BigInt.from(i ~/ _size));
    if (mounted) setState(() => _state = s);
  }

  Future<void> _flag(int i) async {
    if (_state.over || _state.won) return;
    final s = await minesweeperToggleFlag(x: BigInt.from(i % _size), y: BigInt.from(i ~/ _size));
    if (mounted) setState(() => _state = s);
  }

  @override
  Widget build(BuildContext context) {
    final mineCount = _state.cells.where((c) => c == 9).length;
    final flagCount = _state.flagged.where((f) => f).length;

    return Scaffold(
      appBar: CommonAppBar(title: '扫雷'),
      body: SafeArea(
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text('地雷: $mineCount  旗子: $flagCount', style: const TextStyle(fontSize: 16)),
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
                  borderRadius: BorderRadius.circular(4),
                ),
                child: _state.cells.isEmpty
                    ? const SizedBox()
                    : GridView.builder(
                        physics: const NeverScrollableScrollPhysics(),
                        padding: const EdgeInsets.all(1),
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: _size,
                          mainAxisSpacing: 1,
                          crossAxisSpacing: 1,
                        ),
                        itemCount: _state.cells.length,
                        itemBuilder: (context, i) {
                          final isRevealed = _state.revealed[i];
                          final isFlagged = _state.flagged[i];
                          final cell = _state.cells[i];
                          Color bg = isRevealed ? Colors.grey.shade200 : Colors.blueGrey.shade300;
                          Widget? child;
                          if (isFlagged) {
                            child = const Text('🚩', style: TextStyle(fontSize: 10));
                          } else if (isRevealed) {
                            if (cell == 9) {
                              child = const Text('💣', style: TextStyle(fontSize: 10));
                            } else if (cell > 0) {
                              child = Text(
                                '${cell.toInt()}',
                                style: TextStyle(
                                  fontSize: 10,
                                  fontWeight: FontWeight.bold,
                                  color: cell == 1
                                      ? Colors.blue
                                      : cell == 2
                                          ? Colors.green
                                          : cell == 3
                                              ? Colors.red
                                              : Colors.indigo,
                                ),
                              );
                            }
                          }
                          return GestureDetector(
                            onTap: () => _click(i),
                            onLongPress: () => _flag(i),
                            child: Container(
                              alignment: Alignment.center,
                              color: bg,
                              child: child,
                            ),
                          );
                        },
                      ),
              ),
              const SizedBox(height: 12),
              if (_state.won)
                const Text('🎉 胜利！', style: TextStyle(fontSize: 22, color: Colors.green))
              else if (_state.over)
                const Text('💥 踩雷了', style: TextStyle(fontSize: 22, color: Colors.red))
              else
                const Text('点击翻开 · 长按插旗', style: TextStyle(fontSize: 13, color: Colors.grey)),
            ],
          ),
        ),
      ),
    );
  }
}
