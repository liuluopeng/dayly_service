import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/wifi_api/user.dart';
import 'package:kongde/services/sqlite_storage.dart';
import 'package:kongde/utils.dart';

enum BackgroundType { solid, blur, defaultColor }

enum AppThemeMode { light, dark, system }
enum UiStyle { material, wp10 }

class SettingsController extends GetxController {
  static const String _backgroundTypeKey = 'background_type';
  static const String _themeModeKey = 'theme_mode';
  static const String _scaleFactorKey = 'scale_factor';
  static const String _localeKey = 'locale';

  final Rx<BackgroundType> backgroundType = BackgroundType.solid.obs;
  final Rx<AppThemeMode> themeMode = AppThemeMode.system.obs;
  final Rx<double> scaleFactor = 1.0.obs;
  final Rx<Locale> locale = const Locale('zh', 'CN').obs;
  final Rx<UiStyle> uiStyle = UiStyle.material.obs;

  SqliteStorage get _store => SqliteStorage();

  @override
  void onInit() {
    super.onInit();
    _store.init().then((_) => _loadSettings());
  }

  Future<void> _loadSettings() async {
    final typeIndex = await _store.getInt(_backgroundTypeKey) ?? 0;
    if (typeIndex < BackgroundType.values.length) {
      backgroundType.value = BackgroundType.values[typeIndex];
    }
    final themeModeIndex = await _store.getInt(_themeModeKey) ?? 2;
    if (themeModeIndex < AppThemeMode.values.length) {
      themeMode.value = AppThemeMode.values[themeModeIndex];
    }
    scaleFactor.value = await _store.getDouble(_scaleFactorKey) ?? 1.0;
    final localeCode = await _store.getString(_localeKey) ?? 'zh_CN';
    final parts = localeCode.split('_');
    locale.value = Locale(parts[0], parts.length > 1 ? parts[1] : '');
    await _loadUiStyle();
  }

  /// 从后端加载设置并应用（登录后调用）
  Future<void> loadFromServer() async {
    try {
      final settings = await getUserSettingsForDart();
      LOGGER.i("从服务器加载设置: language=${settings.language}, theme=${settings.flutterTheme}");

      final newLocale = settings.language == 'en'
          ? const Locale('en', 'US')
          : const Locale('zh', 'CN');
      if (locale.value != newLocale) {
        locale.value = newLocale;
        Get.updateLocale(newLocale);
        await _store.setString(_localeKey, '${newLocale.languageCode}_${newLocale.countryCode}');
      }

      final newTheme = settings.flutterTheme == 'light'
          ? AppThemeMode.light
          : AppThemeMode.dark;
      if (themeMode.value != newTheme) {
        themeMode.value = newTheme;
        Get.changeThemeMode(appThemeMode);
        await _store.setInt(_themeModeKey, newTheme.index);
      }
    } catch (e) {
      LOGGER.w("从服务器加载设置失败，使用本地缓存: $e");
    }
  }

  Future<void> setBackgroundType(BackgroundType type) async {
    await _store.setInt(_backgroundTypeKey, type.index);
    backgroundType.value = type;
  }

  Future<void> setThemeMode(AppThemeMode mode) async {
    await _store.setInt(_themeModeKey, mode.index);
    themeMode.value = mode;
    Get.changeThemeMode(appThemeMode);
    _syncThemeToServer(mode);
  }

  void _syncThemeToServer(AppThemeMode mode) {
    final themeStr = mode == AppThemeMode.light ? 'light' : 'dark';
    updateUserSettingsForDart(flutterTheme: themeStr).then((_) {
      LOGGER.i("主题已同步到服务器: $themeStr");
    }).catchError((e) {
      LOGGER.w("同步主题到服务器失败: $e");
    });
  }

  Future<void> setLocale(Locale newLocale) async {
    await _store.setString(_localeKey, '${newLocale.languageCode}_${newLocale.countryCode}');
    locale.value = newLocale;
    Get.updateLocale(newLocale);
    _syncLocaleToServer(newLocale);
  }

  void _syncLocaleToServer(Locale newLocale) {
    final langStr = newLocale.languageCode;
    updateUserSettingsForDart(language: langStr).then((_) {
      LOGGER.i("语言已同步到服务器: $langStr");
    }).catchError((e) {
      LOGGER.w("同步语言到服务器失败: $e");
    });
  }

  Future<void> setScaleFactor(double value) async {
    await _store.setDouble(_scaleFactorKey, value);
    scaleFactor.value = value;
  }

  void increaseScale() {
    final newValue = scaleFactor.value + 0.1;
    if (newValue <= 2.0) {
      setScaleFactor(newValue);
    }
  }

  void decreaseScale() {
    final newValue = scaleFactor.value - 0.1;
    if (newValue >= 0.5) {
      setScaleFactor(newValue);
    }
  }

  bool get isBlurBackground => backgroundType.value == BackgroundType.blur;
  bool get isDefaultColor =>
      backgroundType.value == BackgroundType.defaultColor;

  ThemeMode get appThemeMode {
    switch (themeMode.value) {
      case AppThemeMode.light: return ThemeMode.light;
      case AppThemeMode.dark: return ThemeMode.dark;
      case AppThemeMode.system: return ThemeMode.system;
    }
  }

  Future<void> setUiStyle(UiStyle style) async {
    await _store.setString('ui_style', style.name);
    uiStyle.value = style;
  }

  Future<void> _loadUiStyle() async {
    final saved = await _store.getString('ui_style');
    if (saved == 'wp10') uiStyle.value = UiStyle.wp10;
  }
}
