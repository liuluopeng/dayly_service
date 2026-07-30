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
        child: Column(
          children: [
            const NotificationBar(),
            Expanded(
              child: Column(
                children: [
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
                  // 迷你播放器
                  const Padding(
                    padding: EdgeInsets.symmetric(horizontal: 24),
                    child: MiniPlayerWidget(),
                  ),
                  const SizedBox(height: 16),
                  Expanded(
                    child: Center(
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
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
