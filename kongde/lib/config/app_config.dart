import 'package:flutter/foundation.dart' show kIsWeb, kReleaseMode;
import 'package:get/get.dart';
import 'package:kongde/services/sqlite_storage.dart';
import 'package:kongde/utils.dart';
import 'package:kongde/src/rust/api/wifi_api/init.dart';
import 'package:kongde/src/rust/api/wifi_api/user.dart';

class ServerEntry {
  String name;
  String host;
  int port;
  String username;
  String password;
  String token;
  // 同源模式：url 返回相对路径（''），请求跟随页面 origin（Web 生产部署自动启用）
  bool sameOrigin;

  ServerEntry({
    required this.name,
    required this.host,
    required this.port,
    this.username = '',
    this.password = '',
    this.token = '',
    this.sameOrigin = false,
  });

  String get url => sameOrigin ? _sameOriginUrl : 'http://$host:$port';

  // 同源模式的绝对地址：页面 origin（wasm reqwest 不接受相对 URL）
  static String get _sameOriginUrl {
    if (kIsWeb) {
      final base = Uri.base;
      return '${base.scheme}://${base.host}:${base.port}';
    }
    return '';
  }
  bool get hasCredentials => username.isNotEmpty && password.isNotEmpty;
  bool get isLoggedIn => token.isNotEmpty;

  Map<String, dynamic> toJson() => {
    'name': name,
    'host': host,
    'port': port,
    'username': username,
    'password': password,
    'token': token,
    'sameOrigin': sameOrigin,
  };

  factory ServerEntry.fromJson(Map<String, dynamic> json) => ServerEntry(
    name: json['name'] as String,
    host: json['host'] as String,
    port: json['port'] as int,
    username: (json['username'] as String?) ?? '',
    password: (json['password'] as String?) ?? '',
    token: (json['token'] as String?) ?? '',
    sameOrigin: (json['sameOrigin'] as bool?) ?? false,
  );
}

class AppConfig extends GetxController {
  static AppConfig? _instance;
  static AppConfig get instance => _instance ??= AppConfig._internal();

  AppConfig._internal();

  // 当前页面 origin 的 host/port（同源模式下编辑对话框展示用）
  static String get sameOriginHost => kIsWeb ? Uri.base.host : '';
  static int get sameOriginPort => kIsWeb ? Uri.base.port : 0;

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

    // 原生平台（macOS/Android/iOS）且未配置任何服务器：
    // 自动预填默认服务器，用户可直接使用或在设置页编辑（原生必须绝对地址）
    if (!kIsWeb && this.servers.isEmpty) {
      this.servers.add(ServerEntry(name: '默认服务器', host: '192.168.31.58', port: 23000));
      LOGGER.i("[config] 原生平台，自动预填默认服务器 192.168.31.58:23000");
      await _saveServers();
      _activeIndex = 0;
    }

    // Web 且未配置任何服务器：按构建模式自动设置
    if (kIsWeb && this.servers.isEmpty) {
      if (kReleaseMode) {
        // 生产构建：同源模式（相对路径，端口跟随页面）
        this.servers.add(ServerEntry(name: '同源自动', host: '', port: 0, sameOrigin: true));
        LOGGER.i("[config] Web 生产构建，自动启用同源模式（相对路径）");
      } else {
        // 开发构建（flutter run）：自动指向本地开发服务器（23001）
        final devHost = Uri.base.host.isNotEmpty ? Uri.base.host : 'localhost';
        this.servers.add(ServerEntry(name: '本地开发', host: devHost, port: 23001));
        LOGGER.i("[config] Web 开发构建，自动指向 $devHost:23001");
      }
      await _saveServers();
      _activeIndex = 0;
    }

    if (_activeIndex >= this.servers.length) _activeIndex = 0;

    LOGGER.i("[config] 加载了 ${this.servers.length} 个服务器, 当前索引=$_activeIndex");

    if (this.servers.isNotEmpty) {
      if (activeServer.sameOrigin) {
        // 同源模式：不设端口，base_url 为空串（相对路径 /api）
        await setClientBaseUrl(baseUrl: '');
        LOGGER.i("[config] 使用同源模式（相对路径）");
      } else {
        await initClient(port: activeServer.port);
        await setClientBaseUrl(baseUrl: activeServer.url);
      }
      LOGGER.i("[config] 切换到服务器 ${activeServer.name} (${activeServer.url})");
      if (activeServer.token.isNotEmpty) {
        await setClientToken(token: activeServer.token);
        LOGGER.i("[config] 使用已保存的 token 登录");
      } else if (activeServer.hasCredentials) {
        LOGGER.i("[config] 尝试自动登录...");
        try {
          final res = await userLoginForDart(username: activeServer.username, password: activeServer.password);
          activeServer.token = res.token;
          await _saveServers();
          await setClientToken(token: res.token);
          LOGGER.i("[config] 自动登录成功");
        } catch (e) {
          LOGGER.w("[config] 自动登录失败: $e");
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
    LOGGER.i("[config] 添加服务器: $name ($host:$port)");
    servers.add(ServerEntry(name: name, host: host, port: port, username: username, password: password));
    _activeIndex = servers.length - 1;
    await _saveServers();
    await initClient(port: port);
    await setClientBaseUrl(baseUrl: 'http://$host:$port');
    if (username.isNotEmpty && password.isNotEmpty) {
      try {
        final res = await userLoginForDart(username: username, password: password);
        await setToken(res.token);
        LOGGER.i("[config] 新服务器登录成功");
      } catch (e) {
        LOGGER.w("[config] 新服务器登录失败: $e");
      }
    }
  }

  Future<void> removeServer(int index) async {
    if (servers.length <= 1) return;
    final removed = servers[index].name;
    servers.removeAt(index);
    if (_activeIndex >= servers.length) _activeIndex = servers.length - 1;
    LOGGER.i("[config] 移除服务器: $removed, 剩余 ${servers.length} 个");
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
    LOGGER.i("[config] 切换到服务器: ${activeServer.name}");
    await _saveServers();
    await initClient(port: activeServer.port);
    await setClientBaseUrl(baseUrl: activeServer.url);
    if (activeServer.token.isNotEmpty) {
      await setClientToken(token: activeServer.token);
    } else if (activeServer.hasCredentials) {
      try {
        final res = await userLoginForDart(username: activeServer.username, password: activeServer.password);
        await setToken(res.token);
        LOGGER.i("[config] 切换服务器后自动登录成功");
      } catch (e) {
        LOGGER.w("[config] 切换服务器后自动登录失败: $e");
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
      sameOrigin: old.sameOrigin,
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
