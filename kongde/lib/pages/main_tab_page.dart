import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:kongde/controllers/settings_controller.dart';
import 'package:kongde/controllers/tab_bar_controller.dart';
import 'package:kongde/pages/home_page.dart';
import 'package:kongde/pages/menu.dart';
import 'package:kongde/pages/profile_page.dart';

class MainTabPage extends StatefulWidget {
  const MainTabPage({super.key});

  @override
  State<MainTabPage> createState() => _MainTabPageState();
}

class _MainTabPageState extends State<MainTabPage> {
  late final TabBarController _tabBarController;
  DateTime? _lastBackPressed;
  Timer? _exitTimer;

  final List<Widget> _pages = [HomePage(), ContactsPage(), ProfilePage()];

  @override
  void initState() {
    super.initState();
    _tabBarController = Get.put(TabBarController());
  }

  @override
  void dispose() {
    _exitTimer?.cancel();
    super.dispose();
  }

  void _onPopInvokedWithResult(bool didPop, dynamic result) {
    if (didPop) return;

    final currentIndex = _tabBarController.currentIndex.value;

    if (currentIndex != 0) {
      _tabBarController.changeTab(currentIndex - 1);
    } else {
      _handleBackPress();
    }
  }

  void _handleBackPress() {
    final now = DateTime.now();

    if (_lastBackPressed == null ||
        now.difference(_lastBackPressed!) > const Duration(seconds: 1)) {
      _lastBackPressed = now;
      _showExitToast();

      _exitTimer?.cancel();
      _exitTimer = Timer(const Duration(seconds: 2), () {
        _lastBackPressed = null;
      });
    } else {
      SystemNavigator.pop();
    }
  }

  void _showExitToast() {
    final scaffoldMessenger = ScaffoldMessenger.of(context);
    scaffoldMessenger.showSnackBar(
      SnackBar(
        content: Text('app.exitHint'.tr),
        duration: Duration(seconds: 2),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: _onPopInvokedWithResult,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final isLandscape = constraints.maxWidth > constraints.maxHeight;
          final isDesktop = constraints.maxWidth > 600;

          if (isLandscape || isDesktop) {
            return _buildLandscapeLayout();
          } else {
            return _buildPortraitLayout();
          }
        },
      ),
    );
  }

  Widget _buildPortraitLayout() {
    final isWp10 = Get.find<SettingsController>().uiStyle.value == UiStyle.wp10;

    if (isWp10) {
      // WP10 Pivot 风格
      return Scaffold(
        backgroundColor: Colors.black,
        body: SafeArea(
          child: Column(
            children: [
              // Pivot 导航栏（水平滑动）
              SizedBox(
                height: 44,
                child: ListView(
                  scrollDirection: Axis.horizontal,
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  children: [
                    _buildPivot('nav.home'.tr, 0),
                    _buildPivot('nav.menu'.tr, 1),
                    _buildPivot('nav.profile'.tr, 2),
                  ],
                ),
              ),
              Expanded(
                child: Obx(() => _pages[_tabBarController.currentIndex.value]),
              ),
            ],
          ),
        ),
      );
    }

    // Material 风格
    return Scaffold(
      body: SafeArea(
        child: Obx(() {
          return _pages[_tabBarController.currentIndex.value];
        }),
      ),
      drawer: Drawer(
        child: ListView(
          padding: EdgeInsets.zero,
          children: [
            const DrawerHeader(
              decoration: BoxDecoration(color: Colors.blue),
              child: Text('Drawer Header'),
            ),
            ListTile(
              title: const Text('Item 1'),
              onTap: () {
                Get.back();
              },
            ),
            ListTile(
              title: const Text('Item 2'),
              onTap: () {
                Get.back();
              },
            ),
          ],
        ),
      ),
      bottomNavigationBar: Obx(() {
        return BottomNavigationBar(
          currentIndex: _tabBarController.currentIndex.value,
          onTap: (index) {
            _tabBarController.changeTab(index);
          },
          type: BottomNavigationBarType.fixed,
          items: [
            BottomNavigationBarItem(icon: Icon(Icons.chat), label: 'nav.home'.tr),
            BottomNavigationBarItem(icon: Icon(Icons.view_module), label: 'nav.menu'.tr),
            BottomNavigationBarItem(icon: Icon(Icons.person), label: 'nav.profile'.tr),
          ],
        );
      }),
    );
  }

  Widget _buildPivot(String label, int index) {
    final selected = _tabBarController.currentIndex.value == index;
    return GestureDetector(
      onTap: () => _tabBarController.changeTab(index),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        decoration: BoxDecoration(
          border: selected
              ? const Border(bottom: BorderSide(color: Colors.white, width: 2))
              : null,
        ),
        child: Text(label,
          style: TextStyle(
            color: selected ? Colors.white : Colors.white54,
            fontSize: 16, fontWeight: FontWeight.w400,
          ),
        ),
      ),
    );
  }

  Widget _buildLandscapeLayout() {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _tabBarController.currentIndex.value,
            onDestinationSelected: (index) {
              _tabBarController.changeTab(index);
            },
            labelType: NavigationRailLabelType.all,
            destinations: [
              NavigationRailDestination(
                icon: Icon(Icons.chat),
                selectedIcon: Icon(Icons.chat),
                label: Text('nav.home'.tr),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.contacts),
                selectedIcon: Icon(Icons.contacts),
                label: Text('nav.menu'.tr),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.person),
                selectedIcon: Icon(Icons.person),
                label: Text('nav.profile'.tr),
              ),
            ],
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(
            child: Obx(() {
              return _pages[_tabBarController.currentIndex.value];
            }),
          ),
        ],
      ),
    );
  }
}
