import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/base64.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class Base64ToolPage extends StatefulWidget {
  const Base64ToolPage({super.key});

  @override
  State<Base64ToolPage> createState() => _Base64ToolPageState();
}

class _Base64ToolPageState extends State<Base64ToolPage> {
  String base64Input = '';
  String base64Output = '';
  bool isEncoding = true;

  void _toggleMode() {
    setState(() {
      isEncoding = !isEncoding;
      base64Input = '';
      base64Output = '';
    });
  }

  void _processBase64() {
    if (base64Input.isEmpty) {
      return;
    }

    setState(() {
      if (isEncoding) {
        base64Output = base64EncodeWasm(input: base64Input);
      } else {
        final decoded = base64DecodeWasm(input: base64Input);
        base64Output = decoded ?? 'base64.decodeFailed'.tr;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'base64.title'.tr,
        actions: [
          IconButton(
            icon: Icon(isEncoding ? Icons.dehaze : Icons.dehaze),
            onPressed: _toggleMode,
            tooltip: isEncoding ? 'base64.switchToDecode'.tr : 'base64.switchToEncode'.tr,
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              isEncoding ? 'base64.encodeInput'.tr : 'base64.decodeInput'.tr,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8.0),
            TextField(
              onChanged: (value) {
                setState(() {
                  base64Input = value;
                });
              },
              decoration: InputDecoration(
                border: const OutlineInputBorder(),
                hintText: isEncoding ? 'base64.encodeHint'.tr : 'base64.decodeHint'.tr,
              ),
              maxLines: 3,
            ),
            const SizedBox(height: 16.0),
            ElevatedButton(
              onPressed: _processBase64,
              child: Text(isEncoding ? 'base64.encode'.tr : 'base64.decode'.tr),
            ),
            const SizedBox(height: 16.0),
            if (base64Output.isNotEmpty) ...[
              Text(
                isEncoding ? 'base64.encodeResult'.tr : 'base64.decodeResult'.tr,
                style: const TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 8.0),
              Container(
                padding: const EdgeInsets.all(12.0),
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.grey),
                  borderRadius: BorderRadius.circular(8.0),
                  color: Colors.grey[50],
                ),
                child: Text(base64Output),
              ),
            ],
          ],
          ),
        ),
      ),
    );
  }
}
