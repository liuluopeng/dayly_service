import 'package:flutter/material.dart';

ThemeData wp10Theme({required bool dark}) {
  final accent = Color(0xFF0078D4); // WP blue
  final bg = dark ? const Color(0xFF1A1A1A) : Colors.white;
  final surface = dark ? const Color(0xFF2D2D2D) : const Color(0xFFF2F2F2);
  final text = dark ? Colors.white : Colors.black;

  return ThemeData(
    useMaterial3: false,
    brightness: dark ? Brightness.dark : Brightness.light,
    scaffoldBackgroundColor: bg,
    colorScheme: ColorScheme.fromSeed(seedColor: accent, brightness: dark ? Brightness.dark : Brightness.light),
    fontFamily: 'Roboto',

    appBarTheme: AppBarTheme(
      backgroundColor: accent,
      foregroundColor: Colors.white,
      elevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.w300),
    ),

    cardTheme: CardThemeData(
      color: surface,
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
    ),

    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: dark ? const Color(0xFF1A1A1A) : const Color(0xFFF2F2F2),
      elevation: 0,
      indicatorColor: accent.withAlpha(30),
    ),

    bottomNavigationBarTheme: BottomNavigationBarThemeData(
      backgroundColor: dark ? const Color(0xFF1A1A1A) : const Color(0xFFF2F2F2),
      selectedItemColor: accent,
      unselectedItemColor: text.withAlpha(150),
      elevation: 0,
      type: BottomNavigationBarType.fixed,
    ),

    dividerTheme: DividerThemeData(
      color: dark ? Colors.white12 : Colors.black12,
      thickness: 1,
    ),

    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: accent,
        foregroundColor: Colors.white,
        elevation: 0,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
      ),
    ),

    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(foregroundColor: accent),
    ),

    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: accent,
        side: BorderSide(color: accent),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
      ),
    ),

    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: dark ? const Color(0xFF2D2D2D) : const Color(0xFFE8E8E8),
      border: OutlineInputBorder(borderRadius: BorderRadius.circular(0), borderSide: BorderSide.none),
      contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
    ),

    chipTheme: ChipThemeData(
      backgroundColor: dark ? const Color(0xFF3D3D3D) : const Color(0xFFE0E0E0),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
      side: BorderSide.none,
    ),

    dialogTheme: DialogThemeData(
      backgroundColor: dark ? const Color(0xFF2D2D2D) : Colors.white,
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(0)),
    ),

    sliderTheme: SliderThemeData(
      activeTrackColor: accent,
      inactiveTrackColor: accent.withAlpha(50),
      thumbColor: accent,
      overlayColor: accent.withAlpha(20),
    ),

    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent;
        return dark ? Colors.white60 : Colors.grey;
      }),
      trackColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent.withAlpha(100);
        return dark ? Colors.white24 : Colors.black26;
      }),
    ),

    checkboxTheme: CheckboxThemeData(
      fillColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return accent;
        return Colors.transparent;
      }),
      checkColor: WidgetStateProperty.all(Colors.white),
      side: BorderSide(color: text.withAlpha(150)),
    ),

    radioTheme: RadioThemeData(
      fillColor: WidgetStateProperty.all(accent),
    ),

    tabBarTheme: TabBarThemeData(
      labelColor: Colors.white,
      unselectedLabelColor: Colors.white60,
      indicatorColor: Colors.white,
    ),
  );
}
