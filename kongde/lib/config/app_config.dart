import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/services/sqlite_storage.dart';
import 'package:kongde/src/rust/api/wifi_api/init.dart';
import 'package:kongde/src/rust/api/wifi_api/user.dart';

class ServerEntry {
  String name;
  String host;
  int port;
  String username;
  String password;
  String token;

  ServerEntry({
    required this.name,
    required this.host,
    required this.port,
    this.username = '',
    this.password = '',
    this.token = '',
  });

  String get url => 'http://$host:$port';
  bool get hasCredentials => username.isNotEmpty && password.isNotEmpty;
  bool get isLoggedIn => token.isNotEmpty;

  Map<String, dynamic> toJson() => {
    'name': name,
    'host': host,
    'port': port,
    'username': username,
    'password': password,
    'token': token,
  };

  factory ServerEntry.fromJson(Map<String, dynamic> json) => ServerEntry(
    name: json['name'] as String,
    host: json['host'] as String,
    port: json['port'] as int,
    username: (json['username'] as String?) ?? '',
    password: (json['password'] as String?) ?? '',
    token: (json['token'] as String?) ?? '',
  );
}

class AppConfig extends GetxController {
  static AppConfig? _instance;
  static AppConfig get instance => _instance ??= AppConfig._internal();

  AppConfig._internal();

  final List<ServerEntry> servers = [];
  int _activeIndex = 0;

  static const String _serversKey = 'servers';
  static const String _activeIndexKey = 'active_server_index';

  bool get isConfigured => servers.isNotEmpty;
  ServerEntry get activeServer => servers[_activeIndex];
  int get activeIndex => _activeIndex;
  String get serverHost => activeServer.host;
  int get serverPort => activeServer.port;
  String get serverUrl => activeServer.url;
  String get username => activeServer.username;
  String get password => activeServer.password;
  String get accessToken => activeServer.token;

  String get videosUrl => '$serverUrl/videos';
  String get videoPlayUrl => '$serverUrl/video/play';
  String get wubiSearchUrl => '$serverUrl/search_ggtt';
  String get imagesScanUrl => '$serverUrl/api/images/scan';
  String get imagesFoldersUrl => '$serverUrl/api/images/folders';
  String get imagesListUrl => '$serverUrl/api/images/list';
  String get imagesThumbnailUrl => '$serverUrl/api/images/thumbnail';
  String get videosScanUrl => '$serverUrl/api/videos/scan';
  String get videosListUrl => '$serverUrl/api/videos/list';

  Future<void> loadFromPreferences() async {
    final store = SqliteStorage();
    await store.init();

    final servers = await store.getJsonList(_serversKey, (json) => ServerEntry.fromJson(json));
    if (servers != null) {
      this.servers.clear();
      this.servers.addAll(servers);
    }

    _activeIndex = await store.getInt(_activeIndexKey) ?? 0;
    if (_activeIndex >= this.servers.length) _activeIndex = 0;

    if (this.servers.isNotEmpty) {
      await initClient(port: activeServer.port);
      await setClientBaseUrl(baseUrl: activeServer.url);
      if (activeServer.token.isNotEmpty) {
        await setClientToken(token: activeServer.token);
      } else if (activeServer.hasCredentials) {
        try {
          final res = await userLoginForDart(username: activeServer.username, password: activeServer.password);
          activeServer.token = res.token;
          await _saveServers();
          await setClientToken(token: res.token);
        } catch (e) {
          debugPrint('自动登录失败: $e');
        }
      }
    }
  }

  Future<void> _saveServers() async {
    final store = SqliteStorage();
    await store.setJsonList(_serversKey, servers.map((e) => e.toJson()).toList());
    await store.setInt(_activeIndexKey, _activeIndex);
  }

  Future<void> addServer(String name, String host, int port, {String username = '', String password = ''}) async {
    servers.add(ServerEntry(name: name, host: host, port: port, username: username, password: password));
    _activeIndex = servers.length - 1;
    await _saveServers();
    await initClient(port: port);
    await setClientBaseUrl(baseUrl: 'http://$host:$port');
    if (username.isNotEmpty && password.isNotEmpty) {
      try {
        final res = await userLoginForDart(username: username, password: password);
        await setToken(res.token);
      } catch (e) {
        debugPrint('登录失败: $e');
      }
    }
  }

  Future<void> removeServer(int index) async {
    if (servers.length <= 1) return;
    servers.removeAt(index);
    if (_activeIndex >= servers.length) _activeIndex = servers.length - 1;
    await _saveServers();
    await initClient(port: activeServer.port);
    await setClientBaseUrl(baseUrl: activeServer.url);
    if (activeServer.token.isNotEmpty) {
      await setClientToken(token: activeServer.token);
    }
  }

  Future<void> switchServer(int index) async {
    if (index < 0 || index >= servers.length) return;
    _activeIndex = index;
    await _saveServers();
    await initClient(port: activeServer.port);
    await setClientBaseUrl(baseUrl: activeServer.url);
    if (activeServer.token.isNotEmpty) {
      await setClientToken(token: activeServer.token);
    } else if (activeServer.hasCredentials) {
      try {
        final res = await userLoginForDart(username: activeServer.username, password: activeServer.password);
        await setToken(res.token);
      } catch (e) {
        debugPrint('自动登录失败: $e');
        await clearClientToken();
      }
    } else {
      await clearClientToken();
    }
  }

  Future<void> updateServer(int index, String name, String host, int port, {String? username, String? password}) async {
    if (index < 0 || index >= servers.length) return;
    final old = servers[index];
    servers[index] = ServerEntry(
      name: name, host: host, port: port,
      username: username ?? old.username,
      password: password ?? old.password,
      token: old.token,
    );
    await _saveServers();
    if (index == _activeIndex) {
      await initClient(port: port);
      await setClientBaseUrl(baseUrl: 'http://${servers[index].host}:$port');
      if (old.token.isNotEmpty) {
        await setClientToken(token: old.token);
      }
    }
  }

  Future<void> setToken(String token) async {
    activeServer.token = token;
    await _saveServers();
    await setClientToken(token: token);
  }

  Future<void> clearToken() async {
    activeServer.token = '';
    activeServer.username = '';
    activeServer.password = '';
    await _saveServers();
    await clearClientToken();
  }

  Map<String, String> getApiHeaders() {
    final headers = <String, String>{};
    if (activeServer.token.isNotEmpty) {
      headers['Authorization'] = 'Bearer ${activeServer.token}';
    }
    return headers;
  }
}
