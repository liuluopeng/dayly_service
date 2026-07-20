import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/src/rust/api/wifi_api/clipboard.dart';

class ClipboardHistoryPage extends StatefulWidget {
  const ClipboardHistoryPage({super.key});

  @override
  State<ClipboardHistoryPage> createState() => _ClipboardHistoryPageState();
}

class _ClipboardHistoryPageState extends State<ClipboardHistoryPage> {
  final RxList<ClipboardEntry> _entries = RxList<ClipboardEntry>();
  final RxBool _isLoading = false.obs;
  final RxString _error = ''.obs;
  final _baseUrl = AppConfig.instance.serverUrl;

  final _searchController = TextEditingController();
  String _filterType = 'all';
  final _count = 50.obs;

  @override
  void initState() {
    super.initState();
    _loadHistory();
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _loadHistory() async {
    _isLoading.value = true;
    _error.value = '';
    try {
      final typeFilter = _filterType == 'all' ? null : _filterType;
      final search = _searchController.text.trim().isEmpty
          ? null
          : _searchController.text.trim();
      final result = await getClipboardHistoryForDart(
        count: PlatformInt64Util.from(_count.value),
        typeFilter: typeFilter,
        search: search,
      );
      _entries.value = result;
    } catch (e) {
      _error.value = e.toString();
    } finally {
      _isLoading.value = false;
    }
  }

  Future<void> _copyText(String text) async {
    await Clipboard.setData(ClipboardData(text: text));
    Get.snackbar(
      'common.copied'.tr,
      '',
      snackPosition: SnackPosition.bottom,
      duration: const Duration(seconds: 1),
    );
  }

  String _formatTime(String ts) {
    try {
      final d = DateTime.parse(ts);
      return '${d.year}-${d.month.toString().padLeft(2, '0')}-${d.day.toString().padLeft(2, '0')} '
          '${d.hour.toString().padLeft(2, '0')}:${d.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return ts;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'menu.clipboardHistory'.tr),
      body: Column(
        children: [
          // Filter + Search bar
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 12, 12, 8),
            child: Column(
              children: [
                // Filter buttons
                Row(
                  children: [
                    _buildFilterChip('all', 'common.all'.tr),
                    const SizedBox(width: 8),
                    _buildFilterChip('text', 'menu.clipboardHistoryText'.tr),
                    const SizedBox(width: 8),
                    _buildFilterChip('image', 'menu.clipboardHistoryImage'.tr),
                    const Spacer(),
                    IconButton(
                      icon: const Icon(Icons.refresh),
                      onPressed: _loadHistory,
                      tooltip: 'common.refresh'.tr,
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                // Search field
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _searchController,
                        decoration: InputDecoration(
                          hintText: 'common.search'.tr,
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(
                            horizontal: 12,
                            vertical: 10,
                          ),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(8),
                          ),
                        ),
                        onSubmitted: (_) => _loadHistory(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    ElevatedButton(
                      onPressed: _loadHistory,
                      child: Text('common.search'.tr),
                    ),
                  ],
                ),
              ],
            ),
          ),

          // Content
          Expanded(
            child: Obx(() {
              if (_isLoading.value) {
                return const Center(child: CircularProgressIndicator());
              }
              if (_error.isNotEmpty) {
                return Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(_error.value, style: const TextStyle(color: Colors.red)),
                      const SizedBox(height: 8),
                      ElevatedButton(
                        onPressed: _loadHistory,
                        child: Text('common.retry'.tr),
                      ),
                    ],
                  ),
                );
              }
              if (_entries.isEmpty) {
                return Center(
                  child: Text(
                    'common.emptyState'.tr,
                    style: TextStyle(color: Colors.grey.shade400),
                  ),
                );
              }
              return RefreshIndicator(
                onRefresh: _loadHistory,
                child: ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  itemCount: _entries.length,
                  itemBuilder: (context, index) {
                    final entry = _entries[index];
                    return _buildEntryCard(entry);
                  },
                ),
              );
            }),
          ),
        ],
      ),
    );
  }

  Widget _buildFilterChip(String value, String label) {
    final selected = _filterType == value;
    return GestureDetector(
      onTap: () {
        _filterType = value;
        _loadHistory();
      },
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: selected ? Colors.blue : Colors.grey.shade200,
          borderRadius: BorderRadius.circular(16),
        ),
        child: Text(
          label,
          style: TextStyle(
            color: selected ? Colors.white : Colors.black87,
            fontSize: 13,
          ),
        ),
      ),
    );
  }

  Widget _buildEntryCard(ClipboardEntry entry) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (entry.entryType == 'text' && entry.textContent != null) ...[
              Text(
                entry.textContent!,
                style: const TextStyle(fontSize: 14),
                maxLines: 10,
                overflow: TextOverflow.ellipsis,
              ),
              const SizedBox(height: 8),
              OutlinedButton.icon(
                onPressed: () => _copyText(entry.textContent!),
                icon: const Icon(Icons.copy, size: 16),
                label: Text('common.copy'.tr),
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                  minimumSize: Size.zero,
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
              ),
            ] else if (entry.imageUrl != null) ...[
              ClipRRect(
                borderRadius: BorderRadius.circular(4),
                child: Image.network(
                  '$_baseUrl${entry.imageUrl}',
                  height: 200,
                  width: double.infinity,
                  fit: BoxFit.contain,
                  errorBuilder: (_, __, ___) => const Icon(Icons.broken_image, size: 48),
                ),
              ),
            ] else ...[
              Text(
                'menu.clipboardHistoryNoPreview'.tr,
                style: TextStyle(
                  fontSize: 13,
                  color: Colors.grey.shade500,
                  fontStyle: FontStyle.italic,
                ),
              ),
            ],
            const SizedBox(height: 4),
            Text(
              _formatTime(entry.createdAt),
              style: TextStyle(fontSize: 11, color: Colors.grey.shade400),
            ),
          ],
        ),
      ),
    );
  }
}
