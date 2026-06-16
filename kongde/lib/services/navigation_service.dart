import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/collins_dict_page.dart';

class NavigationService {
  static const MethodChannel _channel =
      MethodChannel('com.example.kongde/navigation');

  static void init() {
    _channel.setMethodCallHandler(_handleMethodCall);
  }

  static Future<dynamic> _handleMethodCall(MethodCall call) async {
    if (call.method == 'navigate') {
      final arguments = call.arguments as Map<dynamic, dynamic>;
      final route = arguments['route'] as String?;
      final word = arguments['word'] as String?;

      if (route == '/collins_dict' && word != null) {
        Get.to(() => CollinsDictPage(initialWord: word));
      }
    }
  }
}
