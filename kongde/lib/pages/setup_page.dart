import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/pages/main_tab_page.dart';

class SetupPage extends StatefulWidget {
  const SetupPage({super.key});

  @override
  State<SetupPage> createState() => _SetupPageState();
}

class _SetupPageState extends State<SetupPage> {
  final _nameController = TextEditingController(text: 'setup.defaultServer'.tr);
  final _hostController = TextEditingController(text: '192.168.31.58');
  final _portController = TextEditingController(text: '23000');
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void dispose() {
    _nameController.dispose();
    _hostController.dispose();
    _portController.dispose();
    _usernameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;

    await AppConfig.instance.addServer(
      _nameController.text.trim(),
      _hostController.text.trim(),
      int.parse(_portController.text.trim()),
      username: _usernameController.text.trim(),
      password: _passwordController.text.trim(),
    );
    if (mounted) {
      Get.offAll(() => const MainTabPage());
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(32),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 400),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.dns, size: 64, color: Theme.of(context).colorScheme.primary),
                const SizedBox(height: 16),
                Text('setup.welcome'.tr, style: Theme.of(context).textTheme.headlineSmall),
                const SizedBox(height: 8),
                Text('setup.welcomeDesc'.tr, style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.grey)),
                const SizedBox(height: 32),
                Form(
                  key: _formKey,
                  child: Column(
                    children: [
                      TextFormField(
                        controller: _nameController,
                        decoration: InputDecoration(
                          labelText: 'common.name'.tr,
                          hintText: 'setup.nameHint'.tr,
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.label),
                        ),
                        validator: (v) => v == null || v.trim().isEmpty ? 'setup.nameRequired'.tr : null,
                      ),
                      const SizedBox(height: 16),
                      TextFormField(
                        controller: _hostController,
                        decoration: InputDecoration(
                          labelText: 'setup.serverAddress'.tr,
                          hintText: 'setup.serverAddressHint'.tr,
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.language),
                        ),
                        validator: (v) => v == null || v.trim().isEmpty ? 'setup.serverAddressRequired'.tr : null,
                      ),
                      const SizedBox(height: 16),
                      TextFormField(
                        controller: _portController,
                        keyboardType: TextInputType.number,
                        decoration: InputDecoration(
                          labelText: 'common.port'.tr,
                          hintText: '23000',
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.numbers),
                        ),
                        validator: (v) {
                          if (v == null || v.trim().isEmpty) return 'setup.portRequired'.tr;
                          final p = int.tryParse(v.trim());
                          if (p == null || p < 1 || p > 65535) return 'setup.portInvalid'.tr;
                          return null;
                        },
                      ),
                      const Divider(height: 32),
                      TextFormField(
                        controller: _usernameController,
                        decoration: InputDecoration(
                          labelText: 'common.username'.tr,
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.person),
                        ),
                        validator: (v) => v == null || v.trim().isEmpty ? 'setup.usernameRequired'.tr : null,
                      ),
                      const SizedBox(height: 16),
                      TextFormField(
                        controller: _passwordController,
                        obscureText: true,
                        decoration: InputDecoration(
                          labelText: 'common.password'.tr,
                          border: OutlineInputBorder(),
                          prefixIcon: Icon(Icons.lock),
                        ),
                        validator: (v) => v == null || v.trim().isEmpty ? 'setup.passwordRequired'.tr : null,
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
                SizedBox(
                  width: double.infinity,
                  height: 48,
                  child: FilledButton(
                    onPressed: _submit,
                    child: Text('setup.start'.tr),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
      ),
    );
  }
}
