import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:audio_service/audio_service.dart';
import 'package:media_kit/media_kit.dart';
import 'package:kongde/services/audio_player_handler.dart';
import 'package:kongde/pages/main_tab_page.dart';
import 'package:kongde/pages/setup_page.dart';
import 'package:kongde/pages/collins_dict_page.dart';
import 'package:kongde/config/app_config.dart';
import 'package:logger/logger.dart';
import 'package:kongde/controllers/settings_controller.dart';
import 'package:kongde/theme/wp10_theme.dart';
import 'package:kongde/locales/messages.dart';
import 'package:kongde/services/navigation_service.dart';

import 'package:kongde/src/rust/api/simple.dart';
import 'package:kongde/src/rust/api/logger_bridge.dart';
import 'package:kongde/src/rust/frb_generated.dart';
import 'package:kongde/utils.dart';
import 'package:kongde/widgets/appbar_mini_window.dart';

final logger = Logger(
  printer: PrettyPrinter(
    methodCount: 2,
    errorMethodCount: 8,
    lineLength: 120,
    colors: true,
    printEmojis: true,
    printTime: true,
  ),
);

final _matLight = ThemeData(colorSchemeSeed: Colors.blueGrey);
final _matDark = ThemeData(colorSchemeSeed: Colors.blueGrey, brightness: Brightness.dark);

late AudioPlayerHandler _audioHandler;
StreamSubscription? _rustLoggerSubscription;

class ScaleIntent extends Intent {
  const ScaleIntent(this.increase);
  final bool increase;
}

class PlayPauseIntent extends Intent {
  const PlayPauseIntent();
}

class ScaleAwareApp extends StatelessWidget {
  final SettingsController settingsController;

  const ScaleAwareApp({Key? key, required this.settingsController})
    : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: {
        LogicalKeySet(LogicalKeyboardKey.meta, LogicalKeyboardKey.equal):
            const ScaleIntent(true),
        LogicalKeySet(LogicalKeyboardKey.meta, LogicalKeyboardKey.minus):
            const ScaleIntent(false),
        LogicalKeySet(LogicalKeyboardKey.space): const PlayPauseIntent(),
      },
      child: Actions(
        actions: {
          ScaleIntent: CallbackAction<ScaleIntent>(
            onInvoke: (intent) {
              if (intent.increase) {
                settingsController.increaseScale();
              } else {
                settingsController.decreaseScale();
              }
              return null;
            },
          ),
          PlayPauseIntent: CallbackAction<PlayPauseIntent>(
            onInvoke: (intent) {
              // 切换播放/暂停状态
              final audioHandler = Get.find<AudioPlayerHandler>();
              if (audioHandler.player.playing) {
                audioHandler.pause();
              } else {
                audioHandler.play();
              }
              return null;
            },
          ),
        },
        child: FocusScope(
          autofocus: true,
          child: Obx(() {
            final scale = settingsController.scaleFactor.value;
            return MediaQuery(
              data: MediaQuery.of(context).copyWith(textScaleFactor: scale),
              child: Directionality(
                textDirection: TextDirection.ltr,
                child: GetMaterialApp(
                    translations: Messages(),
                    locale: settingsController.locale.value,
                    fallbackLocale: const Locale('zh', 'CN'),
                    home: AppConfig.instance.isConfigured
                        ? const MainTabPage()
                        : const SetupPage(),
                    debugShowCheckedModeBanner: false,
                theme: settingsController.uiStyle.value == UiStyle.wp10
                  ? wp10Theme(dark: false)
                  : _matLight.copyWith(
                    textTheme: _matLight.textTheme.apply(fontFamily: 'LXGWWenKaiMono'),
                    iconTheme: _matLight.iconTheme.copyWith(size: (_matLight.iconTheme.size ?? 24) * scale),
                    appBarTheme: _matLight.appBarTheme.copyWith(toolbarHeight: kToolbarHeight * scale),
                    buttonTheme: _matLight.buttonTheme.copyWith(minWidth: 88 * scale, height: 36 * scale),
                  ),
                darkTheme: settingsController.uiStyle.value == UiStyle.wp10
                  ? wp10Theme(dark: true)
                  : _matDark.copyWith(
                    textTheme: _matDark.textTheme.apply(fontFamily: 'LXGWWenKaiMono'),
                    iconTheme: _matDark.iconTheme.copyWith(size: (_matDark.iconTheme.size ?? 24) * scale),
                    appBarTheme: _matDark.appBarTheme.copyWith(toolbarHeight: kToolbarHeight * scale),
                    buttonTheme: _matDark.buttonTheme.copyWith(minWidth: 88 * scale, height: 36 * scale),
                  ),
                themeMode: settingsController.appThemeMode,
                onGenerateRoute: (settings) {
                  if (settings.name == '/collins_dict') {
                    final word = settings.arguments as String?;
                    return GetPageRoute(
                      page: () => CollinsDictPage(initialWord: word),
                    );
                  }
                  return null;
                },
              ),
              ),
            );
          }),
        ),
      ),
    );
  }
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  MediaKit.ensureInitialized();

  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    systemNavigationBarColor: Colors.black,
    statusBarColor: Colors.transparent,
  ));

  await RustLib.init();

  await AppConfig.instance.loadFromPreferences();
  // initClient 和 setClientToken 已在 loadFromPreferences 中处理

  _rustLoggerSubscription = initRustLogger().listen((msg) {
    Future.microtask(() {
      LOGGER.i("[rs]: $msg");
    });

    // 在appbar上的监控小窗执行
    AppBarMiniWindow.show("[rs]: $msg");
  });

  logger.i("Logger initialized");

  _audioHandler = await AudioService.init(
    builder: () => AudioPlayerHandler(),
    config: AudioServiceConfig(
      // androidNotificationChannelId: 'com.mycompany.myapp.channel.audio',
      // androidNotificationChannelName: 'Music playback',
    ),
  );
  Get.put(_audioHandler);

  final settingsController = SettingsController();
  Get.put(settingsController);

  NavigationService.init();

  runApp(ScaleAwareApp(settingsController: settingsController));

  // 从服务器同步设置（不阻塞启动）
  settingsController.loadFromServer();
}
