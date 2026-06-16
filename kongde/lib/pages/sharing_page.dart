import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/src/rust/api/wifi_api/webrtc.dart';

class SharingPage extends StatefulWidget {
  const SharingPage({super.key});

  @override
  State<SharingPage> createState() => _SharingPageState();
}

class _SharingPageState extends State<SharingPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  StreamSubscription<String>? _wsSubscription;
  bool _connected = false;
  String _myName = '';
  final List<Map<String, String>> _peers = [];

  List<Map<String, String>> get _allPeers {
    final list = <Map<String, String>>[];
    if (_myName.isNotEmpty) {
      list.add({'id': '__self', 'name': '$_myName(我)'});
    }
    list.addAll(_peers);
    return list;
  }

  // 剪贴板
  final List<Map<String, dynamic>> _clipboardHistory = [];
  String _lastClipboard = '';
  Timer? _clipboardTimer;
  final TextEditingController _clipboardInputController = TextEditingController();

  // 文件传输
  final List<Map<String, dynamic>> _transfers = [];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _connectWebRTC();
  }

  @override
  void dispose() {
    _wsSubscription?.cancel();
    _clipboardTimer?.cancel();
    _clipboardInputController.dispose();
    _tabController.dispose();
    super.dispose();
  }

  void _connectWebRTC() {
    final config = AppConfig.instance;
    final deviceName = '${config.username}@${config.serverHost}_${DateTime.now().millisecondsSinceEpoch % 10000}';
    _myName = deviceName;

    final stream = connectWebrtc(deviceName: deviceName);
    _wsSubscription = stream.listen(
      (event) {
        try {
          final msg = jsonDecode(event);
          final type = msg['type'] as String?;

          if (type == 'clipboard') {
            final content = msg['content'] as String? ?? '';
            final from = msg['from'] as String? ?? '';
            if (content.isNotEmpty) {
              setState(() {
                _clipboardHistory.insert(0, {
                  'content': content,
                  'from': from,
                  'time': DateTime.now().toIso8601String(),
                });
              });
            }
          } else if (type == 'file_offer') {
            setState(() {
              _transfers.add({
                'file_id': msg['file_id'],
                'name': msg['name'],
                'size': msg['size'],
                'direction': 'receive',
                'progress': 0.0,
                'chunks': <String>[],
                'status': 'receiving',
              });
            });
          } else if (type == 'file_chunk') {
            _handleFileChunk(msg);
          } else if (type == 'file_end') {
            _handleFileEnd(msg);
          } else if (type == 'connected') {
            setState(() => _connected = true);
          } else if (type == 'peer_joined') {
            setState(() {
              final id = msg['id'] as String? ?? '';
              final name = msg['name'] as String? ?? '';
              if (id.isNotEmpty && !_peers.any((p) => p['id'] == id)) {
                _peers.add({'id': id, 'name': name});
              }
            });
          } else if (type == 'peer_left') {
            setState(() {
              _peers.removeWhere((p) => p['id'] == (msg['id'] as String? ?? ''));
            });
          } else if (type == 'peer_list') {
            setState(() {
              final list = msg['peers'] as List<dynamic>? ?? [];
              _peers.clear();
              for (final p in list) {
                _peers.add({
                  'id': p['id'] as String? ?? '',
                  'name': p['name'] as String? ?? '',
                });
              }
            });
          }
        } catch (e) {
          debugPrint('WebRTC parse error: $e');
        }
      },
      onDone: () {
        setState(() => _connected = false);
      },
    );

    // 开始监听剪贴板
    _clipboardTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _checkClipboard(),
    );
  }

  Future<void> _checkClipboard() async {
    try {
      final data = await Clipboard.getData(Clipboard.kTextPlain);
      final text = data?.text ?? '';
      if (text.isNotEmpty && text != _lastClipboard) {
        _lastClipboard = text;
        _sendData({
          'type': 'clipboard',
          'content': text,
          'from': _myName,
        });
        setState(() {
          _clipboardHistory.insert(0, {
            'content': text,
            'from': _myName,
            'time': DateTime.now().toIso8601String(),
          });
        });
      }
    } catch (_) {}
  }

  Future<void> _sendData(Map<String, dynamic> data) async {
    try {
      await sendWebrtcMessage(msg: jsonEncode(data));
    } catch (e) {
      debugPrint('WebRTC send error: $e');
    }
  }

  void _handleFileChunk(Map<String, dynamic> msg) {
    final fileId = msg['file_id'] as String?;
    final data = msg['data'] as String?;
    if (fileId == null || data == null) return;

    final index = _transfers.indexWhere((t) => t['file_id'] == fileId);
    if (index >= 0) {
      setState(() {
        (_transfers[index]['chunks'] as List).add(data);
      });
    }
  }

  void _handleFileEnd(Map<String, dynamic> msg) {
    final fileId = msg['file_id'] as String?;
    if (fileId == null) return;

    final index = _transfers.indexWhere((t) => t['file_id'] == fileId);
    if (index >= 0) {
      setState(() {
        _transfers[index]['status'] = 'completed';
      });
    }
  }

  Future<void> _pickAndSendFile() async {
    final result = await FilePicker.platform.pickFiles(withData: true);
    if (result == null || result.files.isEmpty) return;

    final file = result.files.first;
    final bytes = file.bytes;
    if (bytes == null) return;

    final fileId = DateTime.now().millisecondsSinceEpoch.toString();

    setState(() {
      _transfers.add({
        'file_id': fileId,
        'name': file.name,
        'size': bytes.length,
        'direction': 'send',
        'progress': 0.0,
        'status': 'sending',
      });
    });

    // 发送 file_offer
    _sendData({
      'type': 'file_offer',
      'file_id': fileId,
      'name': file.name,
      'size': bytes.length,
    });

    // 分块发送（64KB）
    const chunkSize = 65536;
    final totalChunks = (bytes.length / chunkSize).ceil();

    for (var i = 0; i < totalChunks; i++) {
      final start = i * chunkSize;
      final end = (start + chunkSize).clamp(0, bytes.length);
      final chunk = bytes.sublist(start, end);

      _sendData({
        'type': 'file_chunk',
        'file_id': fileId,
        'index': i,
        'data': base64Encode(chunk),
      });

      // 更新进度
      if (mounted) {
        setState(() {
          final idx = _transfers.indexWhere((t) => t['file_id'] == fileId);
          if (idx >= 0) {
            _transfers[idx]['progress'] = (i + 1) / totalChunks;
          }
        });
      }
    }

    // 发送 file_end
    _sendData({'type': 'file_end', 'file_id': fileId});

    if (mounted) {
      setState(() {
        final idx = _transfers.indexWhere((t) => t['file_id'] == fileId);
        if (idx >= 0) {
          _transfers[idx]['status'] = 'completed';
          _transfers[idx]['progress'] = 1.0;
        }
      });
    }
  }

  Future<void> _downloadFile(Map<String, dynamic> transfer) async {
    final chunks = transfer['chunks'] as List<dynamic>;
    if (chunks.isEmpty) return;

    final allBytes = <int>[];
    for (final chunk in chunks) {
      allBytes.addAll(base64Decode(chunk as String));
    }

    try {
      final dir = await getTemporaryDirectory();
      final file = File('${dir.path}/${transfer['name']}');
      await file.writeAsBytes(allBytes);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('已保存到: ${file.path}'), duration: const Duration(seconds: 3)),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('保存失败: $e')),
        );
      }
    }
  }

  String _formatTime(String ts) {
    try {
      final d = DateTime.parse(ts).toLocal();
      return '${d.hour.toString().padLeft(2, '0')}:${d.minute.toString().padLeft(2, '0')}:${d.second.toString().padLeft(2, '0')}';
    } catch (_) {
      return '';
    }
  }

  String _formatSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  void _sendManualClipboard() {
    final text = _clipboardInputController.text.trim();
    if (text.isEmpty) return;
    _sendData({
      'type': 'clipboard',
      'content': text,
      'from': _myName,
    });
    setState(() {
      _clipboardHistory.insert(0, {
        'content': text,
        'from': _myName,
        'time': DateTime.now().toIso8601String(),
      });
    });
    _clipboardInputController.clear();
  }

  Widget _buildClipboardTab() {
    return Column(
      children: [
        Expanded(
          child: _clipboardHistory.isEmpty
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.content_paste, size: 48, color: Colors.grey[600]),
                      const SizedBox(height: 16),
                      Text(
                        _connected ? '复制文本即可同步' : '正在连接...',
                        style: TextStyle(color: Colors.grey[500], fontSize: 14),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(12),
                  itemCount: _clipboardHistory.length,
                  itemBuilder: (context, index) {
                    final item = _clipboardHistory[index];
                    return Card(
                      color: Colors.grey[850],
                      margin: const EdgeInsets.only(bottom: 8),
                      child: ListTile(
                        title: Text(
                          item['content'] ?? '',
                          maxLines: 3,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(color: Colors.white, fontSize: 14),
                        ),
                        subtitle: Text(
                          '${item['from'] ?? ''} · ${_formatTime(item['time'] ?? '')}',
                          style: TextStyle(color: Colors.grey[500], fontSize: 11),
                        ),
                        trailing: IconButton(
                          icon: const Icon(Icons.copy, color: Colors.blue, size: 20),
                          onPressed: () {
                            Clipboard.setData(
                                ClipboardData(text: item['content'] ?? ''));
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('已复制'),
                                duration: Duration(seconds: 1),
                              ),
                            );
                          },
                        ),
                      ),
                    );
                  },
                ),
        ),
        // 手动输入框
        Container(
          padding: const EdgeInsets.all(8),
          color: Colors.grey[900],
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _clipboardInputController,
                  style: const TextStyle(color: Colors.white, fontSize: 14),
                  decoration: InputDecoration(
                    hintText: '输入文本发送...',
                    hintStyle: TextStyle(color: Colors.grey[600]),
                    filled: true,
                    fillColor: Colors.grey[850],
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide.none,
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                  ),
                  onSubmitted: (_) => _sendManualClipboard(),
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.send, color: Colors.blue),
                onPressed: _sendManualClipboard,
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildFileTab() {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(12),
          child: SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: _pickAndSendFile,
              icon: const Icon(Icons.upload_file),
              label: const Text('选择文件发送'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.blue[700],
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 12),
              ),
            ),
          ),
        ),
        Expanded(
          child: _transfers.isEmpty
              ? Center(
                  child: Text(
                    '暂无文件传输',
                    style: TextStyle(color: Colors.grey[500], fontSize: 14),
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  itemCount: _transfers.length,
                  itemBuilder: (context, index) {
                    final t = _transfers[index];
                    final isSend = t['direction'] == 'send';
                    final status = t['status'] as String;
                    final progress = (t['progress'] as num?)?.toDouble() ?? 0.0;

                    return Card(
                      color: Colors.grey[850],
                      margin: const EdgeInsets.only(bottom: 8),
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              children: [
                                Icon(
                                  isSend
                                      ? Icons.arrow_upward
                                      : Icons.arrow_downward,
                                  color: isSend ? Colors.blue : Colors.green,
                                  size: 20,
                                ),
                                const SizedBox(width: 8),
                                Expanded(
                                  child: Text(
                                    t['name'] ?? '',
                                    style: const TextStyle(
                                      color: Colors.white,
                                      fontSize: 14,
                                    ),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                Text(
                                  _formatSize(t['size'] ?? 0),
                                  style: TextStyle(
                                    color: Colors.grey[500],
                                    fontSize: 12,
                                  ),
                                ),
                              ],
                            ),
                            const SizedBox(height: 8),
                            LinearProgressIndicator(
                              value: progress,
                              backgroundColor: Colors.grey[800],
                              valueColor: AlwaysStoppedAnimation<Color>(
                                status == 'completed'
                                    ? Colors.green
                                    : Colors.blue,
                              ),
                            ),
                            const SizedBox(height: 4),
                            Row(
                              children: [
                                Text(
                                  status == 'completed'
                                      ? '完成'
                                      : '${(progress * 100).toStringAsFixed(0)}%',
                                  style: TextStyle(
                                    color: Colors.grey[500],
                                    fontSize: 11,
                                  ),
                                ),
                                const Spacer(),
                                if (status == 'completed' && !isSend && (t['chunks'] as List?)?.isNotEmpty == true)
                                  TextButton.icon(
                                    onPressed: () => _downloadFile(t),
                                    icon: const Icon(Icons.download, size: 14, color: Colors.blue),
                                    label: const Text('下载', style: TextStyle(fontSize: 11, color: Colors.blue)),
                                    style: TextButton.styleFrom(
                                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 0),
                                      minimumSize: Size.zero,
                                      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                                    ),
                                  ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            const Text('局域网共享', style: TextStyle(fontSize: 16)),
            const SizedBox(width: 8),
            Container(
              width: 8,
              height: 8,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: _connected ? Colors.green : Colors.grey,
              ),
            ),
          ],
        ),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Get.back(),
        ),
        bottom: PreferredSize(
          preferredSize: Size.fromHeight(80),
          child: Column(
            children: [
              SizedBox(
                height: 32,
                child: ListView.separated(
                  scrollDirection: Axis.horizontal,
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  itemCount: _allPeers.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 8),
                  itemBuilder: (_, i) {
                    final p = _allPeers[i];
                    final isSelf = p['id'] == '__self';
                    return Chip(
                      avatar: Icon(Icons.devices, size: 14, color: isSelf ? Colors.blue : Colors.green),
                      label: Text(p['name'] ?? '', style: const TextStyle(fontSize: 11)),
                      backgroundColor: isSelf ? Colors.blue.withAlpha(30) : Colors.grey[850],
                      labelStyle: TextStyle(color: isSelf ? Colors.blue[200] : Colors.white70),
                      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      visualDensity: VisualDensity.compact,
                    );
                  },
                ),
              ),
              TabBar(
                controller: _tabController,
                indicatorColor: Colors.blue,
                labelColor: Colors.white,
                unselectedLabelColor: Colors.grey,
                tabs: const [
                  Tab(text: '剪贴板'),
                  Tab(text: '文件传输'),
                ],
              ),
            ],
          ),
        ),
      ),
      body: SafeArea(
        child: TabBarView(
          controller: _tabController,
          children: [
            _buildClipboardTab(),
            _buildFileTab(),
          ],
        ),
      ),
    );
  }
}
