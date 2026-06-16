import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/base64.dart';
import 'package:kongde/src/rust/api/utils/calculator.dart';
import 'package:kongde/src/rust/api/utils/password.dart';
import 'package:kongde/src/rust/api/utils/timestamp.dart';
import 'package:kongde/src/rust/api/utils/uuid.dart';

class UtilsPage extends StatefulWidget {
  const UtilsPage({super.key});

  @override
  State<UtilsPage> createState() => _UtilsPageState();
}

class _UtilsPageState extends State<UtilsPage> {
  // Base64 测试
  String base64Input = 'Hello, Flutter Rust Bridge!';
  String base64Output = '';
  String base64DecodeInput = '';
  String base64DecodeOutput = '';

  // 计算器测试
  int calculatorA = 10;
  int calculatorB = 20;
  int addResult = 0;
  int add22Result = 0;
  int multiplyResult = 0;

  // 密码生成测试
  int passwordLength = 12;
  String passwordResult = '';
  String strongPasswordResult = '';

  // 时间戳测试
  String currentTimestamp = '';
  String currentLocalTime = '';
  String currentUTCTime = '';

  // UUID 测试
  String uuidV4 = '';
  String uuidV6 = '';
  String uuidV7 = '';

  @override
  void initState() {
    super.initState();
    // 初始化测试数据
    _testBase64();
    _testCalculator();
    _testPassword();
    _loadAsyncData();
  }

  Future<void> _loadAsyncData() async {
    await _testTimestamp();
    await _testUUID();
  }

  void _testBase64() {
    setState(() {
      base64Output = base64EncodeWasm(input: base64Input);
      base64DecodeInput = base64Output;
      base64DecodeOutput = base64DecodeWasm(input: base64DecodeInput) ?? 'base64.decodeFailed'.tr;
    });
  }

  void _testCalculator() {
    setState(() {
      addResult = add(a: calculatorA, b: calculatorB);
      add22Result = add22(a: calculatorA, b: calculatorB);
      multiplyResult = multiply(a: calculatorA, b: calculatorB);
    });
  }

  void _testPassword() {
    setState(() {
      passwordResult = generatePassword(length: BigInt.from(passwordLength));
      strongPasswordResult = generateStrongPassword(
        length: BigInt.from(passwordLength),
      );
    });
  }

  Future<void> _testTimestamp() async {
    setState(() {
      // 显示加载状态
    });

    try {
      final timestamp = await getCurrentTimestamp();
      final localTime = await getCurrentLocalTime();
      final utcTime = await getCurrentUtcTime();

      setState(() {
        currentTimestamp = timestamp.toString();
        currentLocalTime = localTime;
        currentUTCTime = utcTime;
      });
    } catch (e) {
      print('获取时间失败: $e');
    }
  }

  Future<void> _testUUID() async {
    setState(() {
      // 显示加载状态
    });

    try {
      final v4 = await generateUuidV4();
      final v6 = await generateUuidV6();
      final v7 = await generateUuidV7();

      setState(() {
        uuidV4 = v4;
        uuidV6 = v6;
        uuidV7 = v7;
      });
    } catch (e) {
      print('生成UUID失败: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('utils.title'.tr)),
      body: SafeArea(
        child: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Base64 测试
            _buildSection('utils.base64Section'.tr, [
              TextField(
                controller: TextEditingController(text: base64Input),
                onChanged: (value) {
                  base64Input = value;
                  _testBase64();
                },
                decoration: InputDecoration(labelText: 'utils.inputString'.tr),
              ),
              const SizedBox(height: 8),
              Text('utils.encodeResult'.trParams({'value': base64Output})),
              const SizedBox(height: 16),
              TextField(
                controller: TextEditingController(text: base64DecodeInput),
                onChanged: (value) {
                  base64DecodeInput = value;
                  setState(() {
                    base64DecodeOutput =
                        base64DecodeWasm(input: base64DecodeInput) ?? 'base64.decodeFailed'.tr;
                  });
                },
                decoration: InputDecoration(labelText: 'utils.base64String'.tr),
              ),
              const SizedBox(height: 8),
              Text('utils.decodeResult'.trParams({'value': base64DecodeOutput})),
            ]),

            const SizedBox(height: 24),

            // 计算器测试
            _buildSection('utils.calculatorSection'.tr, [
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: TextEditingController(
                        text: calculatorA.toString(),
                      ),
                      onChanged: (value) {
                        calculatorA = int.tryParse(value) ?? 0;
                        _testCalculator();
                      },
                      keyboardType: TextInputType.number,
                      decoration: InputDecoration(labelText: 'calculator.numberA'.tr),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: TextField(
                      controller: TextEditingController(
                        text: calculatorB.toString(),
                      ),
                      onChanged: (value) {
                        calculatorB = int.tryParse(value) ?? 0;
                        _testCalculator();
                      },
                      keyboardType: TextInputType.number,
                      decoration: InputDecoration(labelText: 'calculator.numberB'.tr),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Text('A + B + 10 = $addResult'),
              Text('A + B + 22 = $add22Result'),
              Text('A * B = $multiplyResult'),
            ]),

            const SizedBox(height: 24),

            // 密码生成测试
            _buildSection('utils.passwordSection'.tr, [
              TextField(
                controller: TextEditingController(
                  text: passwordLength.toString(),
                ),
                onChanged: (value) {
                  passwordLength = int.tryParse(value) ?? 12;
                  _testPassword();
                },
                keyboardType: TextInputType.number,
                decoration: InputDecoration(labelText: 'utils.passwordLength'.tr),
              ),
              const SizedBox(height: 16),
              Text('utils.randomPassword'.trParams({'value': passwordResult})),
              Text('utils.strongPassword'.trParams({'value': strongPasswordResult})),
            ]),

            const SizedBox(height: 24),

            // 时间戳测试
            _buildSection('utils.timestampSection'.tr, [
              Text('utils.currentTimestamp'.trParams({'value': currentTimestamp})),
              Text('utils.currentLocalTime'.trParams({'value': currentLocalTime})),
              Text('utils.currentUtcTime'.trParams({'value': currentUTCTime})),
              ElevatedButton(
                onPressed: _testTimestamp,
                child: Text('timestamp.refresh'.tr),
              ),
            ]),

            const SizedBox(height: 24),

            // UUID 测试
            _buildSection('utils.uuidSection'.tr, [
              Text('UUID v4: $uuidV4'),
              Text('UUID v6: $uuidV6'),
              Text('UUID v7: $uuidV7'),
              ElevatedButton(
                onPressed: _testUUID,
                child: Text('uuid.generate'.tr),
              ),
            ]),
          ],
        ),
      ),
    );
  }

  Widget _buildSection(String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 12),
        ...children,
      ],
      ),
    );
  }
}
