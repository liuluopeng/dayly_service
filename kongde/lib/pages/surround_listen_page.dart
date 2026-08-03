import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/services/surround_listen_service.dart';

/// 环境监听页：戴耳机时把麦克风采集的外界声音实时回放，
/// 避免错过环境动静（如有人叫你）。
/// 控制逻辑由 [SurroundListenService] 承载（与首页快捷小组件共享状态）。
class SurroundListenPage extends StatelessWidget {
  const SurroundListenPage({super.key});

  @override
  Widget build(BuildContext context) {
    final service = SurroundListenService.instance;

    return Scaffold(
      appBar: AppBar(title: Text('surround.title'.tr)),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Obx(() {
            final listening = service.listening.value;
            final paused = service.paused.value;
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('surround.intro'.tr,
                            style: TextStyle(color: Colors.grey[600])),
                        const SizedBox(height: 8),
                        Text(
                          'surround.warning'.tr,
                          style: TextStyle(
                              color: Colors.orange[700], fontSize: 12),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 24),
                ElevatedButton.icon(
                  onPressed: () async {
                    try {
                      if (listening) {
                        await service.stop();
                      } else {
                        await service.start();
                      }
                    } catch (e) {
                      service.status.value =
                          'surround.startFailed'.trParams({'error': '$e'});
                    }
                  },
                  icon: Icon(listening ? Icons.hearing_disabled : Icons.hearing),
                  label: Text(listening ? 'surround.stop'.tr : 'surround.start'.tr),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: listening ? Colors.redAccent : null,
                    padding: const EdgeInsets.symmetric(vertical: 16),
                    textStyle: const TextStyle(fontSize: 16),
                  ),
                ),
                if (listening) ...[
                  const SizedBox(height: 12),
                  ElevatedButton.icon(
                    onPressed: service.togglePause,
                    icon: Icon(paused ? Icons.play_arrow : Icons.pause),
                    label: Text(paused ? 'surround.resume'.tr : 'surround.pause'.tr),
                  ),
                ],
                const SizedBox(height: 24),
                Row(
                  children: [
                    Text('surround.gain'.tr),
                    Expanded(
                      child: Slider(
                        value: service.gain.value,
                        onChanged: (v) async {
                          service.gain.value = v;
                          await service.setGain(v);
                        },
                        min: 0.1,
                        max: 6.0,
                        divisions: 59,
                        label: '${service.gain.value.toStringAsFixed(2)}x',
                      ),
                    ),
                    Text('${service.gain.value.toStringAsFixed(1)}x'),
                  ],
                ),
                const SizedBox(height: 8),
                Text(
                  service.status.value,
                  style: TextStyle(
                    color: listening ? Colors.green[700] : Colors.grey[600],
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
            );
          }),
        ),
      ),
    );
  }
}
