import 'dart:math';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/password.dart';
import 'dart:math' as math;
import 'package:kongde/widgets/common_app_bar.dart';

class PasswordToolPage extends StatefulWidget {
  const PasswordToolPage({super.key});

  @override
  State<PasswordToolPage> createState() => _PasswordToolPageState();
}

class _PasswordToolPageState extends State<PasswordToolPage> {
  int passwordLength = 12;
  String passwordResult = '';
  String strongPasswordResult = '';

  void _generatePassword() {
    setState(() {
      passwordResult = generatePassword(length: BigInt.from(passwordLength));
    });
  }

  void _generateStrongPassword() {
    setState(() {
      strongPasswordResult = generateStrongPassword(
        length: BigInt.from(passwordLength),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'password.title'.tr),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Text('password.length'.tr, style: const TextStyle(fontSize: 16)),
                const SizedBox(width: 16.0),
                Expanded(
                  child: Slider(
                    value: passwordLength.toDouble(),
                    min: 8,
                    max: 32,
                    onChanged: (value) {
                      setState(() {
                        passwordLength = value.round();
                      });
                    },
                  ),
                ),
                Text('$passwordLength', style: const TextStyle(fontSize: 16)),
                const SizedBox(width: 16.0),
              ],
            ),
            const SizedBox(height: 16.0),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton(
                    onPressed: _generatePassword,
                    child: Text('password.generateRandom'.tr),
                  ),
                ),
                const SizedBox(width: 16.0),
                Expanded(
                  child: ElevatedButton(
                    onPressed: _generateStrongPassword,
                    child: Text('password.generateStrong'.tr),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16.0),
            if (passwordResult.isNotEmpty) ...[
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'password.randomPassword'.tr,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8.0),
                      Text(passwordResult),
                    ],
                  ),
                ),
              ),
            ],
            if (strongPasswordResult.isNotEmpty) ...[
              const SizedBox(height: 16.0),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'password.strongPassword'.tr,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8.0),
                      Text(strongPasswordResult),
                    ],
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
      ),
    );
  }
}
