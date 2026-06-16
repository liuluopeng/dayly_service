import 'package:flutter/material.dart';
import 'package:get/get.dart';

class AppBarMiniWindow extends StatefulWidget {
  static const int maxLines = 200;
  static final ValueNotifier<int> _count = ValueNotifier(0);
  static final List<String> _messages = [];
  static final RxBool _isVisible = true.obs;
  static ScrollController? _scrollController;

  static RxBool get isVisible => _isVisible;

  const AppBarMiniWindow({super.key});

  static void toggle() {
    _isVisible.value = !_isVisible.value;
  }

  static void show(String message) {
    _messages.add(message);
    if (_messages.length > maxLines) {
      _messages.removeRange(0, _messages.length - maxLines);
    }
    _count.value = _messages.length;
  }

  @override
  State<AppBarMiniWindow> createState() => _AppBarMiniWindowState();
}

class _AppBarMiniWindowState extends State<AppBarMiniWindow> {
  @override
  void initState() {
    super.initState();
    AppBarMiniWindow._scrollController ??= ScrollController();
    AppBarMiniWindow._count.addListener(_onUpdate);
  }

  @override
  void dispose() {
    AppBarMiniWindow._count.removeListener(_onUpdate);
    super.dispose();
  }

  void _onUpdate() {
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final messages = AppBarMiniWindow._messages;
    return Container(
      height: kToolbarHeight,
      decoration: BoxDecoration(
        color: Colors.black,
        border: Border(top: BorderSide(color: Colors.grey.shade700, width: 1)),
      ),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: Colors.grey.shade900,
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '${'miniWindow.log'.tr} (${messages.length})',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: ListView.builder(
              controller: AppBarMiniWindow._scrollController,
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              itemCount: messages.length,
              reverse: true,
              itemBuilder: (context, index) {
                final msg = messages[messages.length - 1 - index];
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 1),
                  child: Text(
                    msg,
                    style: const TextStyle(
                      color: Colors.green,
                      fontSize: 9,
                      fontFamily: 'monospace',
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
