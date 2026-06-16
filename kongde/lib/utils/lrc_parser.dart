class LrcLine {
  final Duration time;
  final String text;

  const LrcLine({required this.time, required this.text});
}

/// 解析 LRC 格式歌词文本
/// 支持 [mm:ss.xx] 和 [mm:ss.xxx] 格式
/// 跳过元数据标签如 [ti:标题] [ar:艺术家] 等
List<LrcLine> parseLrc(String lrcText) {
  final lines = <LrcLine>[];
  final timeRegex = RegExp(r'^\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)');

  for (final line in lrcText.split('\n')) {
    final trimmed = line.trim();
    if (trimmed.isEmpty) continue;

    final match = timeRegex.firstMatch(trimmed);
    if (match != null) {
      final minutes = int.parse(match.group(1)!);
      final seconds = int.parse(match.group(2)!);
      final msStr = match.group(3)!;
      final milliseconds = msStr.length == 2
          ? int.parse(msStr) * 10
          : int.parse(msStr);
      final text = match.group(4)!.trim();

      // 跳过空文本行（纯时间戳行用于多时间戳同一歌词的情况）
      if (text.isEmpty) continue;

      lines.add(LrcLine(
        time: Duration(
          minutes: minutes,
          seconds: seconds,
          milliseconds: milliseconds,
        ),
        text: text,
      ));
    }
  }

  lines.sort((a, b) => a.time.compareTo(b.time));
  return lines;
}
