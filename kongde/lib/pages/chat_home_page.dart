import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;
import 'package:kongde/config/app_config.dart';
import 'package:kongde/pages/chat_page.dart';

class ChatHomePage extends StatefulWidget {
  const ChatHomePage({super.key});

  @override
  State<ChatHomePage> createState() => _ChatHomePageState();
}

class _ChatHomePageState extends State<ChatHomePage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  List<Map<String, dynamic>> _recentContacts = [];
  List<Map<String, dynamic>> _contacts = [];
  bool _loadingRecent = true;
  bool _loadingContacts = true;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _loadRecentContacts();
    _loadContacts();
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _loadRecentContacts() async {
    try {
      final config = AppConfig.instance;
      final res = await http.get(
        Uri.parse('${config.serverUrl}/api/chat/recent-contacts'),
        headers: {'Authorization': 'Bearer ${config.accessToken}'},
      );
      if (res.statusCode == 200) {
        final json = jsonDecode(res.body);
        final data = json['data'] as List? ?? [];
        setState(() {
          _recentContacts = data.cast<Map<String, dynamic>>();
          _loadingRecent = false;
        });
      } else {
        setState(() => _loadingRecent = false);
      }
    } catch (e) {
      debugPrint('Failed to load recent contacts: $e');
      if (mounted) setState(() => _loadingRecent = false);
    }
  }

  Future<void> _loadContacts() async {
    try {
      final config = AppConfig.instance;
      final res = await http.get(
        Uri.parse('${config.serverUrl}/api/chat/contacts'),
        headers: {'Authorization': 'Bearer ${config.accessToken}'},
      );
      if (res.statusCode == 200) {
        final json = jsonDecode(res.body);
        final data = json['data'] as List? ?? [];
        setState(() {
          _contacts = data.cast<Map<String, dynamic>>();
          _loadingContacts = false;
        });
      } else {
        setState(() => _loadingContacts = false);
      }
    } catch (e) {
      debugPrint('Failed to load contacts: $e');
      if (mounted) setState(() => _loadingContacts = false);
    }
  }

  String _formatTime(String ts) {
    try {
      final d = DateTime.parse(ts).toLocal();
      final now = DateTime.now();
      if (d.year == now.year && d.month == now.month && d.day == now.day) {
        return '${d.hour.toString().padLeft(2, '0')}:${d.minute.toString().padLeft(2, '0')}';
      }
      return '${d.month}/${d.day} ${d.hour.toString().padLeft(2, '0')}:${d.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return '';
    }
  }

  Widget _buildRecentTab() {
    if (_loadingRecent) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_recentContacts.isEmpty) {
      return const Center(
        child: Text('暂无聊天记录', style: TextStyle(color: Colors.grey)),
      );
    }
    return RefreshIndicator(
      onRefresh: _loadRecentContacts,
      child: ListView.separated(
        itemCount: _recentContacts.length,
        separatorBuilder: (_, _) => Divider(
          height: 1,
          color: Colors.grey[800],
          indent: 68,
        ),
        itemBuilder: (context, index) {
          final contact = _recentContacts[index];
          final username = contact['username'] ?? '';
          final lastMsg = contact['last_message'] ?? '';
          final lastTime = contact['last_message_at'] ?? '';

          return ListTile(
            leading: CircleAvatar(
              backgroundColor: Colors.blue[700],
              child: Text(
                username.isNotEmpty ? username[0].toUpperCase() : '?',
                style: const TextStyle(color: Colors.white, fontSize: 16),
              ),
            ),
            title: Text(
              username,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 15,
                fontWeight: FontWeight.w500,
              ),
            ),
            subtitle: Text(
              lastMsg,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(color: Colors.grey[500], fontSize: 13),
            ),
            trailing: Text(
              _formatTime(lastTime),
              style: TextStyle(color: Colors.grey[600], fontSize: 11),
            ),
            onTap: () => Get.to(() => const ChatPage()),
          );
        },
      ),
    );
  }

  Widget _buildContactsTab() {
    if (_loadingContacts) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_contacts.isEmpty) {
      return const Center(
        child: Text('暂无联系人', style: TextStyle(color: Colors.grey)),
      );
    }
    return RefreshIndicator(
      onRefresh: _loadContacts,
      child: ListView.separated(
        itemCount: _contacts.length,
        separatorBuilder: (_, _) => Divider(
          height: 1,
          color: Colors.grey[800],
          indent: 68,
        ),
        itemBuilder: (context, index) {
          final user = _contacts[index];
          final username = user['username'] ?? '';

          return ListTile(
            leading: CircleAvatar(
              backgroundColor: Colors.green[700],
              child: Text(
                username.isNotEmpty ? username[0].toUpperCase() : '?',
                style: const TextStyle(color: Colors.white, fontSize: 16),
              ),
            ),
            title: Text(
              username,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 15,
                fontWeight: FontWeight.w500,
              ),
            ),
            onTap: () => Get.to(() => const ChatPage()),
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('聊天', style: TextStyle(fontSize: 16)),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Get.back(),
        ),
        bottom: TabBar(
          controller: _tabController,
          indicatorColor: Colors.blue,
          labelColor: Colors.white,
          unselectedLabelColor: Colors.grey,
          tabs: const [
            Tab(text: '最近聊天'),
            Tab(text: '通讯录'),
          ],
        ),
      ),
      body: SafeArea(
        child: TabBarView(
          controller: _tabController,
          children: [
            _buildRecentTab(),
            _buildContactsTab(),
          ],
        ),
      ),
    );
  }
}
