import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/settings_page.dart';
import 'package:kongde/widgets/notification_bar.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ProfilePage extends StatelessWidget {
  const ProfilePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'profile.title'.tr),
      body: SafeArea(
        child: Column(
          children: [
            const NotificationBar(),
            Expanded(
              child: ListView(
                children: [
                  Card(
                    margin: const EdgeInsets.all(16),
                    child: ListTile(
                      leading: const Icon(Icons.settings),
                      title: Text('profile.settings'.tr),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => Get.to(() => const SettingsPage()),
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
