import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/play_online_music_page.dart';
import 'package:kongde/widgets/appbar_mini_window.dart';

class CommonAppBar extends StatelessWidget implements PreferredSizeWidget {
  final String? title;
  final Widget? titleWidget;
  final List<Widget>? actions;
  final Widget? leading;
  final bool automaticallyImplyLeading;
  final Color? backgroundColor;
  final IconThemeData? iconTheme;

  const CommonAppBar({
    super.key,
    this.title,
    this.titleWidget,
    this.actions,
    this.leading,
    this.automaticallyImplyLeading = true,
    this.backgroundColor,
    this.iconTheme,
  });

  @override
  Size get preferredSize {
    final miniHeight = AppBarMiniWindow.isVisible.value ? kToolbarHeight : 0.0;
    return Size.fromHeight(kToolbarHeight + miniHeight);
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() => AppBar(
      title: titleWidget ?? (title != null ? Text(title!) : null),
      leading: leading,
      automaticallyImplyLeading: automaticallyImplyLeading,
      backgroundColor: backgroundColor,
      iconTheme: iconTheme,
      bottom: AppBarMiniWindow.isVisible.value
          ? PreferredSize(
              preferredSize: const Size.fromHeight(kToolbarHeight),
              child: AppBarMiniWindow(),
            )
          : null,
      actions: [
        LabeledIconButton(
          icon: Icons.terminal,
          label: 'appBar.log'.tr,
          onPressed: AppBarMiniWindow.toggle,
        ),
        LabeledIconButton(
          icon: Icons.music_note,
          label: 'appBar.music'.tr,
          onPressed: () {
            Get.to(() => const PlayOnlineMusicPage());
            AppBarMiniWindow.show('跳转到音乐播放器');
          },
        ),
        ...?actions,
      ],
    ));
  }
}

class LabeledIconButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onPressed;

  const LabeledIconButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 20),
            const SizedBox(height: 2),
            Text(
              label,
              style: const TextStyle(fontSize: 10),
            ),
          ],
        ),
      ),
    );
  }
}
