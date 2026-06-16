import 'package:get/get.dart';

class JishuqiController extends GetxController {
  var count = 0.obs;
  increment() => count++;
}

class TabBarController extends GetxController {
  var currentIndex = 0.obs;

  void changeTab(int index) {
    currentIndex.value = index;
  }
}
