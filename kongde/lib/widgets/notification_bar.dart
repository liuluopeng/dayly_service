import 'package:flutter/material.dart';
import 'package:get/get.dart';

class NotificationBar extends StatelessWidget {
  static final RxString _message = ''.obs;
  static final RxBool _isVisible = false.obs;

  const NotificationBar({super.key});

  static void show(String message) {
    _message.value = message;
    _isVisible.value = true;
    Future.delayed(const Duration(seconds: 2), () {
      _isVisible.value = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      if (!_isVisible.value) {
        return const SizedBox.shrink();
      }

      return AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeInOut,
        height: 40,
        margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        decoration: BoxDecoration(
          color: Colors.black,
          borderRadius: BorderRadius.circular(8),
        ),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: AnimatedOpacity(
            duration: const Duration(milliseconds: 300),
            opacity: _isVisible.value ? 1.0 : 0.0,
            child: AnimatedSlide(
              duration: const Duration(milliseconds: 300),
              offset: Offset(0, _isVisible.value ? 0.0 : 1.0),
              child: Center(
                child: Text(
                  _message.value,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 14,
                  ),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
          ),
        ),
      );
    });
  }
}
