import 'package:flutter/material.dart';

const _ansiColors = <int, Color>{
  30: Colors.black,   31: Colors.red,      32: Colors.green,
  33: Colors.yellow,  34: Colors.blue,      35: Colors.magenta,
  36: Colors.cyan,    37: Colors.white,
  90: Colors.grey,    91: Color(0xFFFF6B6B), 92: Color(0xFF69DB7C),
  93: Color(0xFFFFD93D), 94: Color(0xFF74C0FC), 95: Color(0xFFDA77F2),
  96: Color(0xFF66D9E8), 97: Color(0xFFFFFFFF),
};

Color _lookupColor(int code) => _ansiColors[code] ?? Colors.green;

/// 将含 ANSI 转义码的文本转为 RichText widget。
/// 无 ANSI 时返回普通 Text（快速路径）。
Widget ansiRichText(String text, {TextStyle? style}) {
  if (!text.contains('\x1b[')) {
    return Text(text, style: style);
  }

  final spans = <TextSpan>[];
  final buf = StringBuffer();
  Color fg = style?.color ?? Colors.green;
  bool bold = (style?.fontWeight == FontWeight.bold);

  void flush() {
    if (buf.isNotEmpty) {
      spans.add(TextSpan(
        text: buf.toString(),
        style: TextStyle(color: fg, fontWeight: bold ? FontWeight.bold : null),
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

      if (params.isEmpty || params == '0') { fg = Colors.green; bold = false; continue; }

      final parts = params.split(';');
      for (int pi = 0; pi < parts.length; pi++) {
        final code = int.tryParse(parts[pi]);
        if (code == null) continue;
        if (code == 0) { fg = Colors.green; bold = false; }
        else if (code == 1) bold = true;
        else if (code == 22) bold = false;
        else if (code == 38) {
          if (pi + 2 < parts.length && parts[pi + 1] == '5') {
            pi += 2; fg = _lookupColor(int.tryParse(parts[pi]) ?? 0);
          } else if (pi + 4 < parts.length && parts[pi + 1] == '2') {
            pi += 2;
            fg = Color.fromRGBO(int.tryParse(parts[pi]) ?? 0, int.tryParse(parts[pi + 1]) ?? 0, int.tryParse(parts[pi + 2]) ?? 0, 1);
            pi += 2;
          }
        } else if ((code >= 30 && code <= 37) || (code >= 90 && code <= 97)) {
          fg = _ansiColors[code]!;
        }
      }
    } else {
      buf.writeCharCode(text.codeUnitAt(i));
    }
    i++;
  }
  flush();

  return RichText(text: TextSpan(style: style, children: spans));
}
