import 'package:flutter/material.dart';

ThemeData wp10Theme({required bool dark}) {
  final accent = const Color(0xFF0078D4);
  final bg = Colors.black;
  final surface = const Color(0xFF1A1A1A);
  final cardBg = const Color(0xFF1A1A1A);
  final text = Colors.white;
  final secondary = Colors.white54;
  final divider = Colors.white12;

  return ThemeData(
    useMaterial3: false,
    brightness: dark ? Brightness.dark : Brightness.light,
    scaffoldBackgroundColor: bg,
    colorScheme: ColorScheme.fromSeed(
      seedColor: accent,
      brightness: Brightness.dark,
      surface: surface,
    ),
    fontFamily: 'LXGWWenKaiMono',

    // 视觉密度 — 紧凑但间距舒适
    visualDensity: VisualDensity.adaptivePlatformDensity,

    appBarTheme: AppBarTheme(
      backgroundColor: bg,
      foregroundColor: text,
      elevation: 0,
      centerTitle: false,
      titleSpacing: 16,
      titleTextStyle: TextStyle(color: text, fontSize: 20, fontWeight: FontWeight.w300, fontFamily: 'LXGWWenKaiMono'),
    ),

    cardTheme: CardThemeData(
      color: cardBg,
      elevation: 0,
      shadowColor: Colors.transparent,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      margin: EdgeInsets.zero,
    ),

    bottomNavigationBarTheme: BottomNavigationBarThemeData(
      backgroundColor: bg,
      selectedItemColor: accent,
      unselectedItemColor: secondary,
      elevation: 0,
      type: BottomNavigationBarType.fixed,
    ),

    dividerTheme: DividerThemeData(color: divider, thickness: 1, space: 0),
    dividerColor: divider,

    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: accent,
        foregroundColor: text,
        elevation: 0,
        shadowColor: Colors.transparent,
        shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
      ),
    ),

    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(foregroundColor: accent),
    ),

    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: text,
        side: BorderSide(color: text.withAlpha(80)),
        shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      ),
    ),

    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: surface,
      contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      border: const OutlineInputBorder(borderRadius: BorderRadius.zero, borderSide: BorderSide.none),
      enabledBorder: const OutlineInputBorder(borderRadius: BorderRadius.zero, borderSide: BorderSide.none),
      focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.zero, borderSide: BorderSide(color: accent, width: 2)),
      labelStyle: TextStyle(color: secondary),
      hintStyle: TextStyle(color: secondary),
    ),

    chipTheme: ChipThemeData(
      backgroundColor: surface,
      labelStyle: TextStyle(color: text, fontSize: 12),
      side: BorderSide.none,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
    ),

    dialogTheme: DialogThemeData(
      backgroundColor: surface,
      elevation: 0,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
    ),

    snackBarTheme: SnackBarThemeData(
      backgroundColor: surface,
      contentTextStyle: TextStyle(color: text, fontSize: 14),
      behavior: SnackBarBehavior.floating,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
    ),

    popupMenuTheme: PopupMenuThemeData(
      color: surface,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      elevation: 0,
    ),

    bottomSheetTheme: BottomSheetThemeData(
      backgroundColor: surface,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      elevation: 0,
    ),

    sliderTheme: SliderThemeData(
      activeTrackColor: accent,
      inactiveTrackColor: accent.withAlpha(50),
      thumbColor: accent,
      overlayColor: accent.withAlpha(20),
      trackHeight: 2,
    ),

    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent;
        return Colors.white54;
      }),
      trackColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent.withAlpha(100);
        return Colors.white24;
      }),
    ),

    checkboxTheme: CheckboxThemeData(
      fillColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent;
        return Colors.transparent;
      }),
      checkColor: WidgetStateProperty.all(Colors.white),
      side: BorderSide(color: secondary),
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
    ),

    radioTheme: RadioThemeData(
      fillColor: WidgetStateProperty.all(accent),
    ),

    tabBarTheme: TabBarThemeData(
      labelColor: accent,
      unselectedLabelColor: secondary,
      indicatorColor: accent,
      labelStyle: TextStyle(fontSize: 15, fontWeight: FontWeight.w400, fontFamily: 'LXGWWenKaiMono'),
      unselectedLabelStyle: TextStyle(fontSize: 15, fontWeight: FontWeight.w300, fontFamily: 'LXGWWenKaiMono'),
    ),

    listTileTheme: ListTileThemeData(
      textColor: text,
      iconColor: secondary,
      contentPadding: const EdgeInsets.symmetric(horizontal: 16),
      dense: true,
    ),

    expansionTileTheme: ExpansionTileThemeData(
      iconColor: secondary,
      collapsedIconColor: secondary,
      textColor: text,
      collapsedTextColor: text,
      childrenPadding: EdgeInsets.zero,
    ),

    progressIndicatorTheme: ProgressIndicatorThemeData(
      color: accent,
      linearTrackColor: accent.withAlpha(30),
    ),

    // 默认文字样式
    textTheme: TextTheme(
      headlineLarge: TextStyle(fontSize: 28, fontWeight: FontWeight.w300, color: text, fontFamily: 'LXGWWenKaiMono'),
      headlineMedium: TextStyle(fontSize: 22, fontWeight: FontWeight.w300, color: text, fontFamily: 'LXGWWenKaiMono'),
      titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.w400, color: text, fontFamily: 'LXGWWenKaiMono'),
      titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.w400, color: text, fontFamily: 'LXGWWenKaiMono'),
      bodyLarge: TextStyle(fontSize: 15, fontWeight: FontWeight.w400, color: text, fontFamily: 'LXGWWenKaiMono'),
      bodyMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.w400, color: text, fontFamily: 'LXGWWenKaiMono'),
      bodySmall: TextStyle(fontSize: 12, fontWeight: FontWeight.w400, color: secondary, fontFamily: 'LXGWWenKaiMono'),
      labelLarge: TextStyle(fontSize: 14, fontWeight: FontWeight.w500, color: text, fontFamily: 'LXGWWenKaiMono'),
      labelSmall: TextStyle(fontSize: 11, fontWeight: FontWeight.w400, color: secondary, fontFamily: 'LXGWWenKaiMono'),
    ),
  );
}
