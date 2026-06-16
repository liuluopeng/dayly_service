import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../src/rust/api/wifi_api/note.dart';

class NoteEditPage extends StatefulWidget {
  final NoteSummary? note;

  const NoteEditPage({Key? key, this.note}) : super(key: key);

  @override
  _NoteEditPageState createState() => _NoteEditPageState();
}

class _NoteEditPageState extends State<NoteEditPage> {
  late TextEditingController _textController;
  late TextEditingController _titleController;

  @override
  void initState() {
    super.initState();
    _textController = TextEditingController(text: widget.note?.text ?? '');
    _titleController = TextEditingController(text: widget.note?.filename ?? '');
  }

  @override
  void dispose() {
    _textController.dispose();
    _titleController.dispose();
    super.dispose();
  }

  Future<void> _saveNote() async {
    try {
      if (widget.note != null) {
        await saveNoteForDart(
          id: widget.note!.id.toString(),
          text: _textController.text,
          filename: _titleController.text.isNotEmpty
              ? _titleController.text
              : null,
        );
      } else {
        await createNoteForDart(
          text: _textController.text,
          filename: _titleController.text.isNotEmpty
              ? _titleController.text
              : null,
        );
      }
      Get.back(result: true);
    } catch (e) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('note.saveFailed'.trParams({'error': '$e'}))));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.note != null ? 'note.editNote'.tr : 'note.createNote'.tr),
        actions: [
          ElevatedButton(
            onPressed: _saveNote,
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.blue,
              foregroundColor: Colors.white,
            ),
            child: Text('note.save'.tr),
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _titleController,
              decoration: InputDecoration(
                labelText: 'note.title'.tr,
                border: const OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            Expanded(
              child: TextField(
                controller: _textController,
                decoration: InputDecoration(
                  labelText: 'note.content'.tr,
                  border: const OutlineInputBorder(),
                ),
                maxLines: null,
                minLines: 10,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
