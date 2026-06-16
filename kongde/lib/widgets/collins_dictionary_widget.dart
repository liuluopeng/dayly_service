import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/services/widget_channel.dart';

class CollinsDictionaryWidget extends StatefulWidget {
  const CollinsDictionaryWidget({super.key});

  @override
  State<CollinsDictionaryWidget> createState() =>
      _CollinsDictionaryWidgetState();
}

class _CollinsDictionaryWidgetState extends State<CollinsDictionaryWidget> {
  final TextEditingController _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _updateWidget() async {
    final word = _controller.text.trim();
    if (word.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('collinsWidget.enterWord'.tr)));
      }
      return;
    }

    await WidgetChannel.updateWidget(word);

    if (mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('collinsWidget.updated'.tr)));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('collinsWidget.title'.tr)),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: _controller,
              decoration: InputDecoration(
                hintText: 'collinsWidget.inputHint'.tr,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
                filled: true,
                fillColor: Colors.grey[100],
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: _updateWidget,
              icon: const Icon(Icons.widgets),
              label: Text('collinsWidget.updateWidget'.tr),
              style: ElevatedButton.styleFrom(
                padding: const EdgeInsets.symmetric(vertical: 16),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
            ),
            const SizedBox(height: 24),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'collinsWidget.instructions'.tr,
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text('collinsWidget.step1'.tr),
                    Text('collinsWidget.step2'.tr),
                    Text('collinsWidget.step3'.tr),
                    Text('collinsWidget.step4'.tr),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
