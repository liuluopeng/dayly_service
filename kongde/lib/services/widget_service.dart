import 'dart:io';
import 'package:flutter/foundation.dart';

class WidgetService {
  static const String _prefsFile = 'CollinsWidgetPrefs';
  static const String _wordKey = 'word_';

  static Future<void> updateWidgetWord(String word) async {
    try {
      if (kDebugMode) {
        print('Updating widget with word: $word');
      }

      if (Platform.isAndroid) {
        await _updateAndroidWidget(word);
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error updating widget: $e');
      }
    }
  }

  static Future<void> _updateAndroidWidget(String word) async {
    try {
      final process = await Process.run('adb', [
        'shell',
        'am',
        'broadcast',
        '-a',
        'com.example.kongde.UPDATE_WIDGET',
        '-e',
        'word',
        word,
      ]);

      if (kDebugMode) {
        print('Widget update result: ${process.exitCode}');
        print('Widget update stdout: ${process.stdout}');
        print('Widget update stderr: ${process.stderr}');
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error running adb command: $e');
      }
    }
  }

  static Future<void> updateAllWidgets() async {
    try {
      if (Platform.isAndroid) {
        final process = await Process.run('adb', [
          'shell',
          'am',
          'broadcast',
          '-a',
          'android.appwidget.action.APPWIDGET_UPDATE',
          '-n',
          'com.example.kongde/.CollinsDictionaryWidget',
        ]);

        if (kDebugMode) {
          print('Widget refresh result: ${process.exitCode}');
        }
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error refreshing widgets: $e');
      }
    }
  }
}
