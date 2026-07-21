import 'package:flutter/material.dart';

/// WP10 Pivot 导航 — 水平滚动的标签栏，无动画切换
class MetroPivot extends StatelessWidget {
  final List<String> labels;
  final int selectedIndex;
  final ValueChanged<int> onChanged;
  final Color? accentColor;

  const MetroPivot({
    super.key,
    required this.labels,
    required this.selectedIndex,
    required this.onChanged,
    this.accentColor,
  });

  @override
  Widget build(BuildContext context) {
    final accent = accentColor ?? Theme.of(context).colorScheme.primary;
    return SizedBox(
      height: 44,
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        children: List.generate(labels.length, (i) {
          final selected = selectedIndex == i;
          return GestureDetector(
            onTap: () => onChanged(i),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              decoration: BoxDecoration(
                border: selected
                    ? Border(bottom: BorderSide(color: accent, width: 2))
                    : null,
              ),
              child: Text(labels[i],
                style: TextStyle(
                  color: selected ? Colors.white : Colors.white54,
                  fontSize: 16, fontWeight: FontWeight.w400,
                ),
              ),
            ),
          );
        }),
      ),
    );
  }
}

/// 多页 Pivot 容器（自动管理页面切换）
class MetroPivotPages extends StatefulWidget {
  final List<String> labels;
  final List<Widget> pages;
  final Color? accentColor;

  const MetroPivotPages({
    super.key,
    required this.labels,
    required this.pages,
    this.accentColor,
  });

  @override
  State<MetroPivotPages> createState() => _MetroPivotPagesState();
}

class _MetroPivotPagesState extends State<MetroPivotPages> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        MetroPivot(
          labels: widget.labels,
          selectedIndex: _index,
          onChanged: (i) => setState(() => _index = i),
          accentColor: widget.accentColor,
        ),
        Expanded(child: widget.pages[_index]),
      ],
    );
  }
}
