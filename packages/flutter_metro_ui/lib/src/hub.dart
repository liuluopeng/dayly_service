import 'package:flutter/material.dart';

/// WP10 Hub 页面 — 水平滚动的 Section 布局
class MetroHub extends StatelessWidget {
  final String? title;
  final List<Widget> sections;

  const MetroHub({
    super.key,
    this.title,
    required this.sections,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (title != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
            child: Text(title!,
              style: Theme.of(context).textTheme.headlineLarge,
            ),
          ),
        Expanded(
          child: ListView(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.all(16),
            children: sections,
          ),
        ),
      ],
    );
  }
}

/// Hub 中的一节
class MetroHubSection extends StatelessWidget {
  final String? title;
  final List<Widget> tiles;

  const MetroHubSection({
    super.key,
    this.title,
    required this.tiles,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 320,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (title != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(title!,
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
          Expanded(child: SingleChildScrollView(
            child: Wrap(
              runSpacing: 4, spacing: 4,
              children: tiles,
            ),
          )),
        ],
      ),
    );
  }
}
