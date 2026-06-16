import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'base64_tool_page.dart';
import 'calculator_tool_page.dart';
import 'password_tool_page.dart';
import 'timestamp_tool_page.dart';
import 'uuid_tool_page.dart';
import 'gif_compare_page.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ToolsHomePage extends StatelessWidget {
  const ToolsHomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final screenWidth = constraints.maxWidth;

        // 计算理想的列数和间距，确保平滑过渡
        int crossAxisCount;
        double iconSize;
        double containerSize;
        double padding;

        // 使用更智能的算法计算布局参数
        if (screenWidth > 900) {
          // 桌面大屏：根据宽度动态调整列数
          crossAxisCount = (screenWidth / 200).floor().clamp(3, 5);
          containerSize = 80;
          iconSize = 40;
          padding = 12.0;
        } else if (screenWidth > 600) {
          // 桌面小屏或平板
          crossAxisCount = (screenWidth / 180).floor().clamp(2, 4);
          containerSize = 70;
          iconSize = 35;
          padding = 14.0;
        } else if (screenWidth > 400) {
          // 大屏手机
          crossAxisCount = 2;
          containerSize = 60;
          iconSize = 30;
          padding = 16.0;
        } else {
          // 小屏手机
          crossAxisCount = 2;
          containerSize = 50;
          iconSize = 25;
          padding = 16.0;
        }

        return Scaffold(
          appBar: CommonAppBar(title: 'tools.title'.tr),
          body: SafeArea(
            child: Padding(
              padding: EdgeInsets.all(padding),
              child: GridView.builder(
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: crossAxisCount,
                  crossAxisSpacing: 16.0,
                  mainAxisSpacing: 16.0,
                  childAspectRatio: 1.0,
                ),
                itemCount: 6,
                itemBuilder: (context, index) {
                  final tools = [
                    {
                      'title': 'tools.base64'.tr,
                      'icon': Icons.code,
                      'color': Colors.blue,
                      'route': () => Get.to(() => const Base64ToolPage()),
                    },
                    {
                      'title': 'tools.calculator'.tr,
                      'icon': Icons.calculate,
                      'color': Colors.green,
                      'route': () => Get.to(() => const CalculatorToolPage()),
                    },
                    {
                      'title': 'tools.password'.tr,
                      'icon': Icons.lock,
                      'color': Colors.red,
                      'route': () => Get.to(() => const PasswordToolPage()),
                    },
                    {
                      'title': 'tools.timestamp'.tr,
                      'icon': Icons.access_time,
                      'color': Colors.orange,
                      'route': () => Get.to(() => const TimestampToolPage()),
                    },
                    {
                      'title': 'tools.uuid'.tr,
                      'icon': Icons.tag,
                      'color': Colors.purple,
                      'route': () => Get.to(() => const UuidToolPage()),
                    },
                    {
                      'title': 'GIF Rust',
                      'icon': Icons.movie_filter,
                      'color': Colors.teal,
                      'route': () => Get.to(() => const GifComparePage()),
                    },
                  ];

                  final tool = tools[index];
                  return GestureDetector(
                    onTap: tool['route'] as GestureTapCallback,
                    child: Card(
                      elevation: 4.0,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12.0),
                      ),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Container(
                            width: containerSize,
                            height: containerSize,
                            decoration: BoxDecoration(
                              color: tool['color'] as Color,
                              borderRadius: BorderRadius.circular(
                                containerSize / 2,
                              ),
                            ),
                            child: Icon(
                              tool['icon'] as IconData,
                              size: iconSize,
                              color: Colors.white,
                            ),
                          ),
                          const SizedBox(height: 16.0),
                          Text(
                            tool['title'] as String,
                            style: const TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        );
      },
    );
  }
}
