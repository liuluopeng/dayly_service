import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/collins_dict_page.dart';
import 'package:kongde/pages/play_online_music_page.dart';
import 'package:kongde/widgets/notification_bar.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/widgets/mini_player_widget.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final _dictController = TextEditingController();
  final _dictFocusNode = FocusNode();

  @override
  void dispose() {
    _dictController.dispose();
    _dictFocusNode.dispose();
    super.dispose();
  }

  void _searchWord() {
    final word = _dictController.text.trim();
    if (word.isEmpty) return;
    Get.to(() => CollinsDictPage(initialWord: word));
    _dictController.clear();
  }

  @override
  Widget build(context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'home.title'.tr),
      body: SafeArea(
        child: LayoutBuilder(
          builder: (context, constraints) {
            final isLandscape = constraints.maxWidth > constraints.maxHeight;

            if (isLandscape) {
              return _buildLandscapeLayout();
            } else {
              return _buildPortraitLayout();
            }
          },
        ),
      ),
    );
  }

  Widget _buildPortraitLayout() {
    return SingleChildScrollView(
      padding: EdgeInsets.only(
        bottom: MediaQuery.of(context).viewInsets.bottom,
      ),
      child: Column(
        children: [
          const NotificationBar(),
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
            child: TextField(
              controller: _dictController,
              focusNode: _dictFocusNode,
              autofocus: true,
              decoration: InputDecoration(
                hintText: '查单词...',
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
                contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 14),
              ),
              textInputAction: TextInputAction.search,
              onSubmitted: (_) => _searchWord(),
            ),
          ),
          const SizedBox(height: 16),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 24),
            child: MiniPlayerWidget(),
          ),
          const SizedBox(height: 16),
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(
                  Icons.music_note,
                  size: 100,
                  color: Colors.blue,
                ),
                const SizedBox(height: 32),
                ElevatedButton(
                  onPressed: () =>
                      Get.to(() => const PlayOnlineMusicPage()),
                  style: ElevatedButton.styleFrom(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 32,
                      vertical: 16,
                    ),
                  ),
                  child: Text(
                    'home.listenMusic'.tr,
                    style: TextStyle(fontSize: 20),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLandscapeLayout() {
    return Row(
      children: [
        // 左侧：搜索 + 播放器
        Expanded(
          flex: 1,
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 16, 8, 0),
                child: TextField(
                  controller: _dictController,
                  focusNode: _dictFocusNode,
                  autofocus: true,
                  decoration: InputDecoration(
                    hintText: '查单词...',
                    prefixIcon: Icon(Icons.search),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                  ),
                  textInputAction: TextInputAction.search,
                  onSubmitted: (_) => _searchWord(),
                ),
              ),
              const SizedBox(height: 12),
              const Expanded(
                child: Padding(
                  padding: EdgeInsets.symmetric(horizontal: 16),
                  child: MiniPlayerWidget(),
                ),
              ),
            ],
          ),
        ),
        const VerticalDivider(width: 1),
        // 右侧：音乐入口
        Expanded(
          flex: 1,
          child: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(
                  Icons.music_note,
                  size: 80,
                  color: Colors.blue,
                ),
                const SizedBox(height: 24),
                ElevatedButton(
                  onPressed: () =>
                      Get.to(() => const PlayOnlineMusicPage()),
                  style: ElevatedButton.styleFrom(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 12,
                    ),
                  ),
                  child: Text(
                    'home.listenMusic'.tr,
                    style: TextStyle(fontSize: 18),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
