import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/uuid.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class UuidToolPage extends StatefulWidget {
  const UuidToolPage({super.key});

  @override
  State<UuidToolPage> createState() => _UuidToolPageState();
}

class _UuidToolPageState extends State<UuidToolPage> {
  String uuidV4 = '';
  String uuidV6 = '';
  String uuidV7 = '';
  bool isLoading = false;

  @override
  void initState() {
    super.initState();
    _loadAsyncData();
  }

  Future<void> _loadAsyncData() async {
    await _testUUID();
  }

  Future<void> _testUUID() async {
    setState(() {
      isLoading = true;
    });

    try {
      final v4 = await generateUuidV4();
      final v6 = await generateUuidV6();
      final v7 = await generateUuidV7();

      setState(() {
        uuidV4 = v4;
        uuidV6 = v6;
        uuidV7 = v7;
        isLoading = false;
      });
    } catch (e) {
      print('生成UUID失败: $e');
      setState(() {
        isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'uuid.title'.tr,
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _testUUID,
            tooltip: 'uuid.generate'.tr,
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: isLoading
            ? const Center(child: CircularProgressIndicator())
            : Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(16.0),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'uuid.result'.tr,
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          const SizedBox(height: 16.0),
                          Text('uuid.v4'.trParams({'value': uuidV4})),
                          const SizedBox(height: 16.0),
                          Text('uuid.v6'.trParams({'value': uuidV6})),
                          const SizedBox(height: 16.0),
                          Text('uuid.v7'.trParams({'value': uuidV7})),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 16.0),
                  ElevatedButton(
                    onPressed: _testUUID,
                    child: Text('uuid.generate'.tr),
                  ),
                ],
              ),
      ),
      ),
    );
  }
}
