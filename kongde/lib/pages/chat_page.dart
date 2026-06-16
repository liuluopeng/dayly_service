import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;
import 'package:kongde/config/app_config.dart';
import 'package:kongde/src/rust/api/wifi_api/chat.dart';

class ChatPage extends StatefulWidget {
  const ChatPage({super.key});

  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  final List<Map<String, dynamic>> _messages = [];
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  bool _loading = true;
  bool _sending = false;
  String? _lastMessageTime;
  String? _currentUserId;
  StreamSubscription<String>? _wsSubscription;

  @override
  void initState() {
    super.initState();
    _loadMessages().then((_) => _connectWebSocket());
  }

  @override
  void dispose() {
    _wsSubscription?.cancel();
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _connectWebSocket() {
    final stream = connectChatWs(path: '/api/chat/ws');
    _wsSubscription = stream.listen(
      (event) {
        try {
          final msg = jsonDecode(event) as Map<String, dynamic>;
          final id = msg['id'] as String?;
          if (id != null && !_messages.any((m) => m['id'] == id)) {
            setState(() {
              _messages.add(msg);
              _lastMessageTime = msg['created_at'];
            });
            _scrollToBottom();
          }
        } catch (e) {
          debugPrint('WS parse error: $e');
        }
      },
      onError: (e) {
        debugPrint('WS error: $e');
      },
      onDone: () {
        debugPrint('WS stream closed');
      },
    );
  }

  void _scrollToBottom() {
    if (_scrollController.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      });
    }
  }

  Future<void> _loadMessages() async {
    try {
      final config = AppConfig.instance;
      final baseUrl = config.serverUrl;
      final token = config.accessToken;
      final url = _lastMessageTime != null
          ? '$baseUrl/api/chat/messages?after=${Uri.encodeComponent(_lastMessageTime!)}'
          : '$baseUrl/api/chat/messages';

      final res = await http.get(
        Uri.parse(url),
        headers: {'Authorization': 'Bearer $token'},
      );

      if (res.statusCode != 200) return;

      final json = jsonDecode(res.body);
      final data = json['data'] as List?;
      if (data == null || data.isEmpty) return;

      if (_lastMessageTime != null) {
        setState(() {
          _messages.addAll(data.cast<Map<String, dynamic>>());
          _lastMessageTime = data.last['created_at'];
        });
      } else {
        final reversed = data.reversed.toList().cast<Map<String, dynamic>>();
        setState(() {
          _messages.clear();
          _messages.addAll(reversed);
          _loading = false;
          if (_messages.isNotEmpty) {
            _lastMessageTime = _messages.last['created_at'];
            final username = config.username;
            final match = _messages.firstWhere(
              (m) => m['username'] == username,
              orElse: () => <String, dynamic>{},
            );
            if (match.isNotEmpty) _currentUserId = match['sender_id'];
          }
        });
      }
      _scrollToBottom();
    } catch (e) {
      debugPrint('Failed to load messages: $e');
      if (_loading && mounted) setState(() => _loading = false);
    }
  }

  Future<void> _sendMessage() async {
    final content = _inputController.text.trim();
    if (content.isEmpty || _sending) return;

    setState(() => _sending = true);
    try {
      final config = AppConfig.instance;
      final baseUrl = config.serverUrl;
      final token = config.accessToken;

      final res = await http.post(
        Uri.parse('$baseUrl/api/chat/messages'),
        headers: {
          'Authorization': 'Bearer $token',
          'Content-Type': 'application/json',
        },
        body: jsonEncode({'content': content}),
      );

      if (res.statusCode == 200) {
        final json = jsonDecode(res.body);
        final msg = json['data'];
        if (msg != null) {
          setState(() {
            _messages.add({
              'id': msg['id'],
              'sender_id': msg['sender_id'],
              'username': config.username,
              'content': msg['content'],
              'created_at': msg['created_at'],
            });
            _currentUserId = msg['sender_id'];
            _lastMessageTime = msg['created_at'];
          });
          _inputController.clear();
          _scrollToBottom();
        }
      }
    } catch (e) {
      debugPrint('Failed to send message: $e');
    } finally {
      if (mounted) setState(() => _sending = false);
    }
  }

  String _formatTime(String ts) {
    try {
      final d = DateTime.parse(ts).toLocal();
      return '${d.hour.toString().padLeft(2, '0')}:${d.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return '';
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('聊天', style: TextStyle(fontSize: 15)),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Get.back(),
        ),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _loading
                  ? const Center(child: CircularProgressIndicator())
                  : ListView.builder(
                      controller: _scrollController,
                      padding: const EdgeInsets.all(16),
                      itemCount: _messages.length,
                      itemBuilder: (context, index) {
                        final msg = _messages[index];
                        final isMine = msg['sender_id'] == _currentUserId;
                        return Padding(
                          padding: const EdgeInsets.only(bottom: 12),
                          child: Row(
                            mainAxisAlignment:
                                isMine ? MainAxisAlignment.end : MainAxisAlignment.start,
                            children: [
                              Flexible(
                                child: Column(
                                  crossAxisAlignment:
                                      isMine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      '${msg['username'] ?? ''} · ${_formatTime(msg['created_at'] ?? '')}',
                                      style: TextStyle(fontSize: 11, color: Colors.grey[600]),
                                    ),
                                    const SizedBox(height: 4),
                                    Container(
                                      padding: const EdgeInsets.symmetric(
                                          horizontal: 12, vertical: 8),
                                      decoration: BoxDecoration(
                                        color: isMine
                                            ? Colors.blue[700]
                                            : Colors.grey[800],
                                        borderRadius: BorderRadius.circular(12),
                                      ),
                                      child: Text(
                                        msg['content'] ?? '',
                                        style: const TextStyle(fontSize: 14, color: Colors.white),
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
            ),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                border: Border(top: BorderSide(color: Colors.grey[800]!)),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _inputController,
                      style: const TextStyle(color: Colors.white),
                      decoration: InputDecoration(
                        hintText: '输入消息...',
                        hintStyle: TextStyle(color: Colors.grey[500]),
                        filled: true,
                        fillColor: Colors.grey[850],
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(20),
                          borderSide: BorderSide.none,
                        ),
                        contentPadding: const EdgeInsets.symmetric(
                            horizontal: 16, vertical: 10),
                      ),
                      onSubmitted: (_) => _sendMessage(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton(
                    onPressed: _sending ? null : _sendMessage,
                    icon: _sending
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.send, color: Colors.blue),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
