import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/wubi_query_page.dart';
import 'package:kongde/pages/video_library_page.dart';
import 'package:kongde/pages/image_gallery_page.dart';
import 'package:kongde/pages/scan_page.dart';
import 'package:kongde/pages/melatonin_movies_page.dart';
import 'package:kongde/pages/unified_dict_page.dart';
import 'package:kongde/pages/online_music_page.dart';
import 'package:kongde/pages/local_music_page.dart';
import 'package:kongde/pages/file_manager_page.dart';
import 'package:kongde/pages/chat_home_page.dart';
import 'package:kongde/pages/sharing_page.dart';
import 'package:kongde/pages/tools_home_page.dart';
import 'package:kongde/pages/piano_keyboard_page.dart';
import 'package:kongde/views/note_view.dart';
import 'package:kongde/pages/clipboard_history_page.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ContactsPage extends StatefulWidget {
  const ContactsPage({super.key});

  @override
  State<ContactsPage> createState() => _ContactsPageState();
}

class _ContactsPageState extends State<ContactsPage> {
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final screenWidth = constraints.maxWidth;

        // 计算理想的列数和间距，确保平滑过渡
        int crossAxisCount;
        double mainAxisSpacing;
        double crossAxisSpacing;
        double padding;

        // 使用更智能的算法计算布局参数
        if (screenWidth > 900) {
          // 桌面大屏：根据宽度动态调整列数
          crossAxisCount = (screenWidth / 150).floor().clamp(5, 8);
          mainAxisSpacing = 12.0;
          crossAxisSpacing = 12.0;
          padding = 12.0;
        } else if (screenWidth > 600) {
          // 桌面小屏或平板
          crossAxisCount = (screenWidth / 160).floor().clamp(4, 6);
          mainAxisSpacing = 14.0;
          crossAxisSpacing = 14.0;
          padding = 14.0;
        } else if (screenWidth > 400) {
          // 大屏手机
          crossAxisCount = 4;
          mainAxisSpacing = 16.0;
          crossAxisSpacing = 16.0;
          padding = 16.0;
        } else {
          // 小屏手机
          crossAxisCount = 3;
          mainAxisSpacing = 18.0;
          crossAxisSpacing = 16.0;
          padding = 16.0;
        }

        return Scaffold(
          appBar: CommonAppBar(title: 'menu.title'.tr),
          body: SafeArea(
            child: Padding(
              padding: EdgeInsets.all(padding),
              child: GridView.builder(
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: crossAxisCount,
                  mainAxisSpacing: mainAxisSpacing,
                  crossAxisSpacing: crossAxisSpacing,
                  childAspectRatio: 0.85,
                ),
                itemCount: 15,
                itemBuilder: (context, index) {
                  final menuItems = [
                    {
                      'icon': Icons.text_fields,
                      'title': 'menu.wubiQuery'.tr,
                      'onTap': () => Get.to(() => WubiQueryPage()),
                    },
                    {
                      'icon': Icons.video_library,
                      'title': 'menu.videoLibrary'.tr,
                      'onTap': () => Get.to(() => VideoLibraryPage()),
                    },
                    {
                      'icon': Icons.photo_library,
                      'title': 'menu.imageGallery'.tr,
                      'onTap': () => Get.to(() => ImageGalleryPage()),
                      'borderRadius': 0.0,
                    },
                    {
                      'icon': Icons.movie_creation,
                      'title': 'menu.melatoninMovies'.tr,
                      'onTap': () => Get.to(() => MelatoninMoviesPage()),
                      'borderRadius': 0.0,
                    },

                    {
                      'icon': Icons.qr_code_scanner,
                      'title': 'menu.scan'.tr,
                      'onTap': () => Get.to(() => ScanPage()),
                    },
                    {
                      'icon': Icons.book,
                      'title': 'menu.unifiedDict'.tr,
                      'onTap': () => Get.to(() => const UnifiedDictPage()),
                    },
                    {
                      'icon': Icons.headphones,
                      'title': 'menu.onlineMusic'.tr,
                      'onTap': () => Get.to(() => const OnlineMusicPage()),
                    },
                    {
                      'icon': Icons.library_music,
                      'title': '本地音乐',
                      'onTap': () => Get.to(() => const LocalMusicPage()),
                    },
                    {
                      'icon': Icons.note,
                      'title': 'menu.noteManager'.tr,
                      'onTap': () => Get.to(() => const NoteView()),
                    },
                    {
                      'icon': Icons.piano,
                      'title': 'menu.pianoKeyboard'.tr,
                      'onTap': () => Get.to(() => const PianoKeyboardPage()),
                    },
                    {
                      'icon': Icons.build,
                      'title': 'menu.tools'.tr,
                      'onTap': () => Get.to(() => const ToolsHomePage()),
                    },
                    {
                      'icon': Icons.folder,
                      'title': 'menu.fileManager'.tr,
                      'onTap': () => Get.to(() => const FileManagerPage()),
                    },
                    {
                      'icon': Icons.chat,
                      'title': '聊天',
                      'onTap': () => Get.to(() => const ChatHomePage()),
                    },
                    {
                      'icon': Icons.share,
                      'title': '局域网共享',
                      'onTap': () => Get.to(() => const SharingPage()),
                    },
                    {
                      'icon': Icons.content_paste,
                      'title': 'menu.clipboardHistory'.tr,
                      'onTap': () => Get.to(() => const ClipboardHistoryPage()),
                    },
                  ];

                  final menuItem = menuItems[index];
                  return _buildMenuCard(
                    icon: menuItem['icon'] as IconData,
                    title: menuItem['title'] as String,
                    onTap: menuItem['onTap'] as GestureTapCallback?,
                    borderRadius: menuItem['borderRadius'] as double? ?? 12,
                  );
                },
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildMenuCard({
    required IconData icon,
    required String title,
    required GestureTapCallback? onTap,
    double borderRadius = 12,
  }) {
    return GestureDetector(
      onTap: onTap,
      child: Column(
        children: [
          Container(
            width: 60,
            height: 60,
            decoration: BoxDecoration(
              color: Colors.blue.withOpacity(0.1),
              borderRadius: BorderRadius.circular(borderRadius),
            ),
            child: Icon(icon, size: 30, color: Colors.blue),
          ),
          const SizedBox(height: 8),
          Expanded(
            child: Text(
              title,
              textAlign: TextAlign.center,
              style: const TextStyle(fontSize: 12),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}
