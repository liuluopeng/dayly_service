import 'package:flutter/foundation.dart';
import 'package:logger/logger.dart';
import 'package:kongde/widgets/appbar_mini_window.dart';

class AppBarMiniWindowOutput extends LogOutput {
  @override
  void output(OutputEvent event) {
    for (final line in event.lines) {
      AppBarMiniWindow.show(line);
    }
  }
}

final LOGGER_MEMORY = MemoryOutput(
  secondOutput: kDebugMode ? ConsoleOutput() : null,
);

final LOGGER = Logger(
  filter: ProductionFilter(),
  printer: PrettyPrinter(
    methodCount: 0,
    errorMethodCount: 5,
    lineLength: 120,
    colors: true,
    printEmojis: true,
    printTime: true,
    noBoxingByDefault: true,
  ),
  output: MultiOutput([
    LOGGER_MEMORY,
    AppBarMiniWindowOutput(),
  ]),
  level: Level.all,
);
