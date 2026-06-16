import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:get/get.dart';
import '../src/rust/frb_generated.dart';
import '../src/rust/api/wifi_api/note.dart';
import 'note_edit_page.dart';
import 'note_view_page.dart';
import '../widgets/common_app_bar.dart';

class NoteView extends StatefulWidget {
  const NoteView({Key? key}) : super(key: key);

  @override
  _NoteViewState createState() => _NoteViewState();
}

class _NoteViewState extends State<NoteView> {
  List<NoteSummary>? _notes;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadNotes();
  }

  Future<void> _loadNotes() async {
    try {
      setState(() {
        _isLoading = true;
        _error = null;
      });

      final result = await listNotesForDart(page: 1, pageSize: 100);
      setState(() {
        _notes = result;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = 'note.loadFailed'.trParams({'error': '$e'});
        _isLoading = false;
      });
    }
  }

  Future<void> _createNote() async {
    final result = await Get.to(() => const NoteEditPage());

    if (result == true) {
      _loadNotes();
    }
  }

  Future<void> _editNote(NoteSummary note) async {
    final result = await Get.to(() => NoteEditPage(note: note));

    if (result == true) {
      _loadNotes();
    }
  }

  Future<void> _viewNote(NoteSummary note) async {
    await Get.to(() => NoteViewPage(note: note));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'note.manager'.tr,
        actions: [
          IconButton(onPressed: _createNote, icon: const Icon(Icons.add)),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
          ? Center(child: Text(_error!))
          : ListView.builder(
              padding: const EdgeInsets.symmetric(vertical: 8),
              itemCount: _notes?.length ?? 0,
              itemBuilder: (context, index) {
                final note = _notes![index];
                return Card(
                  margin: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 8,
                  ),
                  elevation: 2,
                  child: InkWell(
                    onTap: () => _viewNote(note),
                    child: Padding(
                      padding: const EdgeInsets.all(16.0),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            note.filename ?? 'note.noTitle'.tr,
                            style: const TextStyle(
                              fontSize: 18,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          const SizedBox(height: 8),
                          Text(
                            note.text != null && note.text!.length > 100
                                ? '${note.text!.substring(0, 100)}...'
                                : note.text ?? 'note.noContent'.tr,
                            style: const TextStyle(
                              fontSize: 14,
                              color: Colors.grey,
                            ),
                            maxLines: 3,
                            overflow: TextOverflow.ellipsis,
                          ),
                          const SizedBox(height: 8),
                          Row(
                            mainAxisAlignment: MainAxisAlignment.end,
                            children: [
                              IconButton(
                                onPressed: () => _editNote(note),
                                icon: const Icon(Icons.edit, size: 20),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
    );
  }
}
