import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/pages/surround_listen_page.dart';
import 'package:kongde/services/surround_listen_service.dart';

/// 首页环境监听快捷小组件：直接开始/停止/暂停监听、快速调增益，
/// 点击进入完整页面。
class SurroundQuickWidget extends StatelessWidget {
  const SurroundQuickWidget({super.key});

  @override
  Widget build(BuildContext context) {
    final service = SurroundListenService.instance;

    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(16),
        onTap: () => Get.to(() => const SurroundListenPage()),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            gradient: const LinearGradient(
              colors: [Color(0xFF1565C0), Color(0xFF0D47A1)],
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
            ),
            borderRadius: BorderRadius.circular(16),
            boxShadow: [
              BoxShadow(
                color: Colors.blue.withValues(alpha: 0.3),
                blurRadius: 8,
                offset: const Offset(0, 2),
              ),
            ],
          ),
          child: Obx(() {
            final listening = service.listening.value;
            final paused = service.paused.value;
            return Row(
              children: [
                // 图标（监听中呼吸灯效果）
                Container(
                  width: 40,
                  height: 40,
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.15),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    listening ? Icons.hearing : Icons.hearing_disabled,
                    color: Colors.white,
                  ),
                ),
                const SizedBox(width: 12),
                // 状态文案
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'menu.surroundListen'.tr,
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 14,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        listening
                            ? (paused
                                ? 'surround.pausedHint'.tr
                                : 'surround.quickListening'.tr)
                            : 'surround.quickIdle'.tr,
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.8),
                          fontSize: 11,
                        ),
                      ),
                    ],
                  ),
                ),
                // 控制按钮
                IconButton(
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
                  icon: Icon(
                    listening ? Icons.stop_circle : Icons.play_circle_fill,
                    color: Colors.white,
                    size: 32,
                  ),
                  tooltip: listening ? 'surround.stop'.tr : 'surround.start'.tr,
                ),
                if (listening)
                  IconButton(
                    onPressed: service.togglePause,
                    icon: Icon(
                      paused ? Icons.play_arrow : Icons.pause_circle_filled,
                      color: Colors.white,
                      size: 28,
                    ),
                    tooltip:
                        paused ? 'surround.resume'.tr : 'surround.pause'.tr,
                  ),
                // 增益快速调节
                SizedBox(
                  width: 64,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.volume_up, color: Colors.white, size: 14),
                      SliderTheme(
                        data: SliderThemeData(
                          trackHeight: 2,
                          thumbShape: const RoundSliderThumbShape(
                            enabledThumbRadius: 6,
                          ),
                          overlayShape: const RoundSliderOverlayShape(
                            overlayRadius: 10,
                          ),
                        ),
                        child: Slider(
                          value: service.gain.value.clamp(0.1, 6.0),
                          onChanged: (v) async {
                            service.gain.value = v;
                            await service.setGain(v);
                          },
                          min: 0.1,
                          max: 6.0,
                        ),
                      ),
                      Text(
                        '${service.gain.value.toStringAsFixed(1)}x',
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.8),
                          fontSize: 10,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            );
          }),
        ),
      ),
    );
  }
}
