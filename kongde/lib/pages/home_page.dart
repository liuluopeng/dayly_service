import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/play_online_music_page.dart';
import 'package:kongde/widgets/notification_bar.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class HomePage extends StatelessWidget {
  const HomePage({super.key});

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
                  Expanded(
                    flex: 1,
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
