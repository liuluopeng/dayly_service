import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/timestamp.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class TimestampToolPage extends StatefulWidget {
  const TimestampToolPage({super.key});

  @override
  State<TimestampToolPage> createState() => _TimestampToolPageState();
}

class _TimestampToolPageState extends State<TimestampToolPage> {
  String currentTimestamp = '';
  String currentLocalTime = '';
  String currentUTCTime = '';
  bool isLoading = false;

  @override
  void initState() {
    super.initState();
    _loadAsyncData();
  }

  Future<void> _loadAsyncData() async {
    await _testTimestamp();
  }

  Future<void> _testTimestamp() async {
    setState(() {
      isLoading = true;
    });

    try {
      final timestamp = await getCurrentTimestamp();
      final localTime = await getCurrentLocalTime();
      final utcTime = await getCurrentUtcTime();

      setState(() {
        currentTimestamp = timestamp.toString();
        currentLocalTime = localTime;
        currentUTCTime = utcTime;
        isLoading = false;
      });
    } catch (e) {
      print('获取时间失败: $e');
      setState(() {
        isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'timestamp.title'.tr,
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _testTimestamp,
            tooltip: 'timestamp.refresh'.tr,
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
                            'timestamp.currentInfo'.tr,
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          const SizedBox(height: 16.0),
                          Text('timestamp.timestamp'.trParams({'value': currentTimestamp})),
                          const SizedBox(height: 8.0),
                          Text('timestamp.localTime'.trParams({'value': currentLocalTime})),
                          const SizedBox(height: 8.0),
                          Text('timestamp.utcTime'.trParams({'value': currentUTCTime})),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 16.0),
                  ElevatedButton(
                    onPressed: _testTimestamp,
                    child: Text('timestamp.refresh'.tr),
                  ),
                ],
              ),
      ),
      ),
    );
  }
}
