import 'package:flutter/material.dart';
import 'package:get/get.dart';

// 基础 ANSI 前景色
const _ansiColors = <int, Color>{
  30: Colors.black,   31: Colors.red,      32: Colors.green,
  33: Colors.yellow,  34: Colors.blue,      35: Colors.magenta,
  36: Colors.cyan,    37: Colors.white,
  90: Colors.grey,    91: Color(0xFFFF6B6B), 92: Color(0xFF69DB7C),
  93: Color(0xFFFFD93D), 94: Color(0xFF74C0FC), 95: Color(0xFFDA77F2),
  96: Color(0xFF66D9E8), 97: Color(0xFFFFFFFF),
};

Color _lookupColor(int code) {
  final c = _ansiColors[code];
  if (c != null) return c;
  return Colors.green; // fallback
}

List<TextSpan> _parseAnsi(String text) {
  if (!text.contains('\x1b[')) {
    return [TextSpan(text: text)];
  }

  final spans = <TextSpan>[];
  final buf = StringBuffer();
  Color fg = Colors.green;
  bool bold = false;

  void flush() {
    if (buf.isNotEmpty) {
      spans.add(TextSpan(
        text: buf.toString(),
        style: TextStyle(color: fg, fontWeight: bold ? FontWeight.bold : FontWeight.normal),
      ));
      buf.clear();
    }
  }

  int i = 0;
  while (i < text.length) {
    if (text.codeUnitAt(i) == 0x1B && i + 1 < text.length && text[i + 1] == '[') {
      flush();
      final end = text.indexOf('m', i + 2);
      if (end == -1) break;
      final params = text.substring(i + 2, end);
      i = end + 1;

      if (params.isEmpty || params == '0') {
        fg = Colors.green; bold = false;
        continue;
      }

      final parts = params.split(';');
      int pi = 0;
      while (pi < parts.length) {
        final code = int.tryParse(parts[pi]);
        if (code == null) { pi++; continue; }
        if (code == 0) { fg = Colors.green; bold = false; }
        else if (code == 1) bold = true;
        else if (code == 22) bold = false;
        else if (code == 38) {
          if (pi + 2 < parts.length && parts[pi + 1] == '5') {
            // 256色
            pi += 2;
            final idx = int.tryParse(parts[pi]) ?? 0;
            fg = _lookupColor(idx);
          } else if (pi + 4 < parts.length && parts[pi + 1] == '2') {
            // 真彩色
            pi += 2;
            final r = int.tryParse(parts[pi]) ?? 0;
            final g = int.tryParse(parts[pi + 1]) ?? 0;
            final b = int.tryParse(parts[pi + 2]) ?? 0;
            fg = Color.fromRGBO(r, g, b, 1);
            pi += 2;
          }
        } else if (code >= 30 && code <= 37) {
          fg = _ansiColors[code]!;
        } else if (code >= 90 && code <= 97) {
          fg = _ansiColors[code]!;
        }
        pi++;
      }
    } else {
      buf.writeCharCode(text.codeUnitAt(i));
    }
    i++;
  }
  flush();
  return spans;
}

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
    final defaultStyle = TextStyle(
      color: Colors.green, fontSize: 9, fontFamily: 'monospace',
    );
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
            decoration: BoxDecoration(color: Colors.grey.shade900),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '${'miniWindow.log'.tr} (${messages.length})',
                  style: const TextStyle(
                    color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold,
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
                final spans = _parseAnsi(msg);
                if (spans.length == 1 && spans[0].text == msg && spans[0].style == null) {
                  return Padding(
                    padding: const EdgeInsets.symmetric(vertical: 1),
                    child: Text(msg, style: defaultStyle),
                  );
                }
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 1),
                  child: RichText(
                    text: TextSpan(style: defaultStyle, children: spans),
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
