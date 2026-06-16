import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'package:get/get.dart';
import '../src/rust/api/wifi_api/note.dart';

class NoteViewPage extends StatelessWidget {
  final NoteSummary note;

  const NoteViewPage({Key? key, required this.note}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(note.filename ?? 'note.noTitle'.tr)),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Markdown(
          data: note.text ?? 'note.noContent'.tr,
          styleSheet: MarkdownStyleSheet.fromTheme(Theme.of(context)),
        ),
      ),
    );
  }
}
