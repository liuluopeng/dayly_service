import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/services/sqlite_storage.dart';
import 'package:kongde/controllers/settings_controller.dart';
import 'package:kongde/config/app_config.dart' as config;
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/widgets/language_switcher.dart';
import 'package:kongde/pages/setup_page.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  void _restartApp() => (context as Element).reassemble();
  final appConfig = config.AppConfig.instance;

  @override
  Widget build(BuildContext context) {
    final settingsController = Get.find<SettingsController>();

    return Scaffold(
      appBar: CommonAppBar(title: 'settings.title'.tr),
      body: SafeArea(
        child: ListView(
          children: [
            _buildSection(
              title: 'settings.language'.tr,
              children: [
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('settings.languageDesc'.tr, style: const TextStyle(color: Colors.grey)),
                      const SizedBox(height: 12),
                      const LanguageSwitcher(),
                    ],
                  ),
                ),
              ],
            ),
            _buildSection(
              title: 'settings.theme'.tr,
              children: [_buildThemeModeSelector(settingsController)],
            ),
            _buildSection(
              title: 'settings.uiStyle'.tr,
              children: [_buildUiStyleSelector(settingsController)],
            ),
            _buildSection(
              title: 'settings.player'.tr,
              children: [_buildBackgroundTypeSelector(settingsController)],
            ),
            _buildSection(
              title: 'settings.server'.tr,
              children: _buildServerList(),
            ),
            _buildSection(
              title: 'settings.data'.tr,
              children: [
                ListTile(
                  leading: const Icon(Icons.restore, color: Colors.orange),
                  title: Text('settings.resetAll'.tr),
                  subtitle: Text('settings.resetAllSubtitle'.tr),
                  onTap: _confirmReset,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSection({required String title, required List<Widget> children}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 24, 16, 8),
          child: Text(title, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.grey)),
        ),
        Card(margin: const EdgeInsets.symmetric(horizontal: 16), child: Column(children: children)),
      ],
    );
  }

  Widget _buildThemeModeSelector(SettingsController controller) {
    return Obx(() => Column(children: [
      RadioListTile<AppThemeMode>(title: Text('settings.light'.tr), subtitle: Text('settings.lightDesc'.tr), value: AppThemeMode.light, groupValue: controller.themeMode.value, onChanged: (v) { if (v != null) controller.setThemeMode(v); }),
      RadioListTile<AppThemeMode>(title: Text('settings.dark'.tr), subtitle: Text('settings.darkDesc'.tr), value: AppThemeMode.dark, groupValue: controller.themeMode.value, onChanged: (v) { if (v != null) controller.setThemeMode(v); }),
      RadioListTile<AppThemeMode>(title: Text('settings.system'.tr), subtitle: Text('settings.systemDesc'.tr), value: AppThemeMode.system, groupValue: controller.themeMode.value, onChanged: (v) { if (v != null) controller.setThemeMode(v); }),
    ]));
  }

  Widget _buildUiStyleSelector(SettingsController controller) {
    return Obx(() => Column(children: [
      RadioListTile<UiStyle>(title: Text('settings.uiMaterial'.tr), subtitle: Text('settings.uiMaterialDesc'.tr), value: UiStyle.material, groupValue: controller.uiStyle.value, onChanged: (v) { if (v != null) { controller.setUiStyle(v); _restartApp(); }}),
      RadioListTile<UiStyle>(title: Text('settings.uiWp10'.tr), subtitle: Text('settings.uiWp10Desc'.tr), value: UiStyle.wp10, groupValue: controller.uiStyle.value, onChanged: (v) { if (v != null) { controller.setUiStyle(v); _restartApp(); }}),
    ]));
  }

  Widget _buildBackgroundTypeSelector(SettingsController controller) {
    return Obx(() => Column(children: [
      RadioListTile<BackgroundType>(title: Text('settings.solidBg'.tr), subtitle: Text('settings.solidBgDesc'.tr), value: BackgroundType.solid, groupValue: controller.backgroundType.value, onChanged: (v) { if (v != null) controller.setBackgroundType(v); }),
      RadioListTile<BackgroundType>(title: Text('settings.blurBg'.tr), subtitle: Text('settings.blurBgDesc'.tr), value: BackgroundType.blur, groupValue: controller.backgroundType.value, onChanged: (v) { if (v != null) controller.setBackgroundType(v); }),
      RadioListTile<BackgroundType>(title: Text('settings.defaultColor'.tr), subtitle: Text('settings.defaultColorDesc'.tr), value: BackgroundType.defaultColor, groupValue: controller.backgroundType.value, onChanged: (v) { if (v != null) controller.setBackgroundType(v); }),
    ]));
  }

  List<Widget> _buildServerList() {
    final items = <Widget>[];
    for (int i = 0; i < appConfig.servers.length; i++) {
      final server = appConfig.servers[i];
      final isActive = i == appConfig.activeIndex;
      items.add(ListTile(
        leading: Icon(isActive ? Icons.check_circle : Icons.dns_outlined, color: isActive ? Colors.green : null),
        title: Text(server.name),
        subtitle: Text(
          '${server.url}  ${server.username}${server.token.isNotEmpty ? '  [${server.token.substring(0, server.token.length > 8 ? 8 : server.token.length)}...]' : ''}',
          style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
        ),
        trailing: Row(mainAxisSize: MainAxisSize.min, children: [
          IconButton(icon: const Icon(Icons.edit, size: 18), onPressed: () => _showEditServerDialog(i)),
          if (appConfig.servers.length > 1)
            IconButton(icon: const Icon(Icons.delete_outline, size: 18, color: Colors.red), onPressed: () => _confirmDelete(i)),
        ]),
        selected: isActive,
        onTap: () async { await appConfig.switchServer(i); setState(() {}); },
      ));
    }
    items.add(ListTile(leading: const Icon(Icons.add_circle_outline), title: Text('settings.addServer'.tr), onTap: _showAddServerDialog));
    return items;
  }

  void _showAddServerDialog() {
    _showServerDialog(
      title: 'settings.addServer'.tr,
      name: '', host: '', port: '23000', username: '', password: '',
      onConfirm: (name, host, port, username, password) async {
        await appConfig.addServer(name, host, port, username: username, password: password);
        setState(() {});
      },
    );
  }

  void _showEditServerDialog(int index) {
    final server = appConfig.servers[index];
    _showServerDialog(
      title: 'settings.editServer'.tr,
      name: server.name, host: server.host, port: server.port.toString(),
      username: server.username, password: server.password,
      onConfirm: (name, host, port, username, password) async {
        await appConfig.updateServer(index, name, host, port, username: username, password: password);
        setState(() {});
      },
    );
  }

  void _showServerDialog({
    required String title,
    required String name, required String host, required String port,
    required String username, required String password,
    required Future<void> Function(String name, String host, int port, String username, String password) onConfirm,
  }) {
    final nameCtl = TextEditingController(text: name);
    final hostCtl = TextEditingController(text: host);
    final portCtl = TextEditingController(text: port);
    final usernameCtl = TextEditingController(text: username);
    final passwordCtl = TextEditingController(text: password);
    final formKey = GlobalKey<FormState>();

    Get.dialog(AlertDialog(
      title: Text(title),
      content: SingleChildScrollView(child: Form(
        key: formKey,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          TextFormField(controller: nameCtl, decoration: InputDecoration(labelText: 'common.name'.tr, border: const OutlineInputBorder()), validator: (v) => v == null || v.trim().isEmpty ? 'common.required'.tr : null),
          const SizedBox(height: 12),
          TextFormField(controller: hostCtl, decoration: InputDecoration(labelText: 'common.address'.tr, hintText: '192.168.1.100', border: const OutlineInputBorder()), validator: (v) => v == null || v.trim().isEmpty ? 'common.required'.tr : null),
          const SizedBox(height: 12),
          TextFormField(controller: portCtl, keyboardType: TextInputType.number, decoration: InputDecoration(labelText: 'common.port'.tr, hintText: '23000', border: const OutlineInputBorder()), validator: (v) { if (v == null || v.trim().isEmpty) return 'common.required'.tr; final p = int.tryParse(v.trim()); if (p == null || p < 1 || p > 65535) return 'common.invalid'.tr; return null; }),
          const Divider(height: 24),
          TextFormField(controller: usernameCtl, decoration: InputDecoration(labelText: 'common.username'.tr, border: const OutlineInputBorder()), validator: (v) => v == null || v.trim().isEmpty ? 'common.required'.tr : null),
          const SizedBox(height: 12),
          TextFormField(controller: passwordCtl, obscureText: true, decoration: InputDecoration(labelText: 'common.password'.tr, border: const OutlineInputBorder()), validator: (v) => v == null || v.trim().isEmpty ? 'common.required'.tr : null),
        ]),
      )),
      actions: [
        TextButton(onPressed: () => Get.back(), child: Text('common.cancel'.tr)),
        FilledButton(onPressed: () async {
          if (!formKey.currentState!.validate()) return;
          await onConfirm(nameCtl.text.trim(), hostCtl.text.trim(), int.parse(portCtl.text.trim()), usernameCtl.text.trim(), passwordCtl.text.trim());
          Get.back();
        }, child: Text('common.confirm'.tr)),
      ],
    ));
  }

  void _confirmDelete(int index) {
    final name = appConfig.servers[index].name;
    Get.dialog(AlertDialog(
      title: Text('settings.deleteServer'.tr),
      content: Text('settings.confirmDelete'.trParams({'name': name})),
      actions: [
        TextButton(onPressed: () => Get.back(), child: Text('common.cancel'.tr)),
        FilledButton(onPressed: () async { await appConfig.removeServer(index); Get.back(); setState(() {}); }, style: FilledButton.styleFrom(backgroundColor: Colors.red), child: Text('common.delete'.tr)),
      ],
    ));
  }

  void _confirmReset() {
    Get.dialog(AlertDialog(
      title: Text('settings.resetAll'.tr),
      content: Text('settings.confirmReset'.tr),
      actions: [
        TextButton(onPressed: () => Get.back(), child: Text('common.cancel'.tr)),
        FilledButton(onPressed: () async {
          final store = SqliteStorage();
          await store.clear();
          Get.back();
          Get.offAll(() => const SetupPage());
        }, style: FilledButton.styleFrom(backgroundColor: Colors.orange), child: Text('settings.confirmResetBtn'.tr)),
      ],
    ));
  }
}
