import 'package:flutter/material.dart';
import 'package:flutter_metro_ui/flutter_metro_ui.dart';

void main() => runApp(const MetroDemoApp());

class MetroDemoApp extends StatelessWidget {
  const MetroDemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return GetMaterialApp(
      theme: wp10Theme(dark: false),
      darkTheme: wp10Theme(dark: true),
      themeMode: ThemeMode.dark,
      home: const MetroDemoPage(),
    );
  }
}

// 简化的 GetX 替代，避免依赖 get
class _Get {
  static void to(Widget page) {}
}
class GetMaterialApp extends StatelessWidget {
  final ThemeData? theme;
  final ThemeData? darkTheme;
  final ThemeMode? themeMode;
  final Widget? home;
  const GetMaterialApp({super.key, this.theme, this.darkTheme, this.themeMode, this.home});
  @override
  Widget build(BuildContext context) => MaterialApp(theme: theme, darkTheme: darkTheme, themeMode: themeMode, home: home);
}

class MetroDemoPage extends StatefulWidget {
  const MetroDemoPage({super.key});

  @override
  State<MetroDemoPage> createState() => _MetroDemoPageState();
}

class _MetroDemoPageState extends State<MetroDemoPage> {
  int _tab = 0;
  double _slider = 0.5;
  bool _switch = true;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('MetroUI Demo')),
      body: MetroPivotPages(
        labels: const ['Tiles', 'Controls', 'Hub'],
        pages: [
          _buildTilesPage(),
          _buildControlsPage(),
          _buildHubPage(),
        ],
      ),
    );
  }

  Widget _buildTilesPage() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          Wrap(
            runSpacing: 4, spacing: 4,
            children: [
              SizedBox(width: 150, height: 150,
                child: MetroTile(icon: Icons.music_note, label: 'Music', onTap: () {}, size: TileSize.medium)),
              SizedBox(width: 150, height: 150,
                child: MetroTile(icon: Icons.photo, label: 'Photos', color: const Color(0xFF0078D4))),
              SizedBox(width: 304, height: 150,
                child: MetroTile(icon: Icons.video_library, label: 'Videos', size: TileSize.wide, color: const Color(0xFF881798))),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildControlsPage() {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Elevated Button'),
          const SizedBox(height: 8),
          ElevatedButton(onPressed: () {}, child: const Text('Button')),
          const SizedBox(height: 24),
          const Text('Slider'),
          Slider(value: _slider, onChanged: (v) => setState(() => _slider = v)),
          const SizedBox(height: 24),
          const Text('Switch'),
          Switch(value: _switch, onChanged: (v) => setState(() => _switch = v)),
          const SizedBox(height: 24),
          const Text('Checkbox'),
          Checkbox(value: true, onChanged: (_) {}),
          const SizedBox(height: 24),
          const Text('Text Field'),
          const TextField(decoration: InputDecoration(hintText: 'Enter text...')),
        ],
      ),
    );
  }

  Widget _buildHubPage() {
    return MetroHub(
      title: 'Hub',
      sections: [
        MetroHubSection(
          title: 'Section 1',
          tiles: List.generate(6, (i) => SizedBox(
            width: 150, height: 150,
            child: MetroTile(icon: Icons.folder, label: 'Item $i', color: i.isEven ? const Color(0xFF0078D4) : null),
          )),
        ),
        const SizedBox(width: 16),
        MetroHubSection(
          title: 'Section 2',
          tiles: List.generate(4, (i) => SizedBox(
            width: 150, height: 150,
            child: MetroTile(icon: Icons.star, label: 'Star $i', color: const Color(0xFF881798)),
          )),
        ),
      ],
    );
  }
}
