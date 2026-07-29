import 'dart:async';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
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

  final List<Widget> _pages = [HomePage(), ContactsPage(), ProfilePage()];

  @override
  void initState() {
    super.initState();
    _tabBarController = Get.put(TabBarController());
  }

  @override
  void dispose() {
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        if (kIsWeb) {
          return _buildWebLayout();
        }
        final isLandscape = constraints.maxWidth > constraints.maxHeight;
        final isDesktop = constraints.maxWidth > 600;

        if (isLandscape || isDesktop) {
          return _buildLandscapeLayout();
        } else {
          return _buildPortraitLayout();
        }
      },
    );
  }

  Widget _buildWebLayout() {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _tabBarController.currentIndex.value,
            onDestinationSelected: (index) {
              _tabBarController.changeTab(index);
            },
            labelType: NavigationRailLabelType.all,
            minWidth: 72,
            groupAlignment: 0.0,
            leading: Padding(
              padding: const EdgeInsets.symmetric(vertical: 8),
              child: Icon(Icons.apps, size: 28, color: Theme.of(context).colorScheme.primary),
            ),
            destinations: [
              NavigationRailDestination(
                icon: Icon(Icons.chat_outlined),
                selectedIcon: Icon(Icons.chat),
                label: Text('nav.home'.tr),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.view_module_outlined),
                selectedIcon: Icon(Icons.view_module),
                label: Text('nav.menu'.tr),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.person_outlined),
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

  Widget _buildPortraitLayout() {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, result) {
        if (didPop) return;
        final currentIndex = _tabBarController.currentIndex.value;
        if (currentIndex != 0) {
          _tabBarController.changeTab(currentIndex - 1);
        }
      },
      child: Scaffold(
        body: SafeArea(
          child: Obx(() {
            return _pages[_tabBarController.currentIndex.value];
          }),
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
      ),
    );
  }

  Widget _buildLandscapeLayout() {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, result) {
        if (didPop) return;
        final currentIndex = _tabBarController.currentIndex.value;
        if (currentIndex != 0) {
          _tabBarController.changeTab(currentIndex - 1);
        }
      },
      child: Scaffold(
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
      ),
    );
  }
}