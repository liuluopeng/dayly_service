import 'package:flutter/services.dart';

class WidgetChannel {
  static const MethodChannel _channel =
      MethodChannel('com.example.kongde/widget');

  static Future<void> updateWidget(String word) async {
    try {
      await _channel.invokeMethod('updateWidget', {'word': word});
    } catch (e) {
      print('Error updating widget: $e');
    }
  }
}
