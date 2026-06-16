import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/controllers/settings_controller.dart';

class LanguageSwitcher extends StatelessWidget {
  const LanguageSwitcher({super.key});

  @override
  Widget build(BuildContext context) {
    final settings = Get.find<SettingsController>();

    return Obx(() {
      final isZh = settings.locale.value.languageCode == 'zh';
      return SegmentedButton<String>(
        segments: const [
          ButtonSegment(value: 'zh', label: Text('中文')),
          ButtonSegment(value: 'en', label: Text('English')),
        ],
        selected: {isZh ? 'zh' : 'en'},
        onSelectionChanged: (Set<String> selected) {
          final code = selected.first;
          settings.setLocale(
            code == 'zh'
                ? const Locale('zh', 'CN')
                : const Locale('en', 'US'),
          );
        },
      );
    });
  }
}
