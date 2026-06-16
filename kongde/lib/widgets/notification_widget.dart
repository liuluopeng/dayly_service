import 'package:flutter/material.dart';
import 'package:get/get.dart';

class NotificationWidget extends StatelessWidget {
  final String message;

  const NotificationWidget({
    super.key,
    required this.message,
  });

  static void show(String message) {
    Get.snackbar(
      '',
      '',
      backgroundColor: Colors.black,
      colorText: Colors.white,
      snackPosition: SnackPosition.top,
      duration: const Duration(seconds: 2),
      margin: const EdgeInsets.all(8),
      borderRadius: 8,
      titleText: const SizedBox.shrink(),
      messageText: Text(
        message,
        style: const TextStyle(fontSize: 14),
        textAlign: TextAlign.center,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return const SizedBox.shrink();
  }
}
