import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';
import 'package:kongde/src/rust/api/wifi_api/ggtt.dart';

class WubiQueryPage extends StatefulWidget {
  const WubiQueryPage({super.key});

  @override
  State<WubiQueryPage> createState() => _WubiQueryPageState();
}

class _WubiQueryPageState extends State<WubiQueryPage> {
  final TextEditingController _textController = TextEditingController();
  final RxString _result = "".obs;
  final RxBool _isLoading = false.obs;
  final _ggttCode = Rxn<GgttCode>();


  @override
  void initState() {
    super.initState();
  }



  Future<void> _queryWubi() async {
    final word = _textController.text.trim();
    if (word.isEmpty ) return;

    _isLoading.value = true;
    try {
      final result = await searchGgttCodeForDart( search: word);
      _ggttCode.value = result;
      _result.value = 'wubi.code86'.trParams({'code': '${result.code86}'});
    } catch (e) {
      _result.value = 'wubi.queryError'.trParams({'error': '$e'});
      print('查询错误: $e');
    } finally {
      _isLoading.value = false;
    }
  }

  @override
  Widget build(context) {
    print("跳转到了五笔查询页面");
    return Scaffold(
      appBar: CommonAppBar(title: 'wubi.title'.tr),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(20.0),
          child: SingleChildScrollView(
            child: Column(
              children: [
                TextField(
                  controller: _textController,
                  decoration: InputDecoration(
                    labelText: 'wubi.inputLabel'.tr,
                    border: OutlineInputBorder(),
                    suffixIcon: IconButton(
                      icon: Icon(Icons.search),
                      onPressed: _queryWubi,
                    ),
                  ),
                  maxLength: 1,
                  textAlign: TextAlign.center,
                  style: TextStyle(fontSize: 24),
                ),
                SizedBox(height: 20),
                ElevatedButton(
                  onPressed: _queryWubi,
                  style: ElevatedButton.styleFrom(
                    padding: EdgeInsets.symmetric(horizontal: 40, vertical: 15),
                    textStyle: TextStyle(fontSize: 18),
                  ),
                  child: Text('common.search'.tr),
                ),
                SizedBox(height: 30),
                Obx(() {
                  if (_isLoading.value) {
                    return CircularProgressIndicator();
                  }
                  return Column(
                    children: [
                      Container(
                        padding: EdgeInsets.all(20),
                        decoration: BoxDecoration(
                          border: Border.all(color: Colors.grey),
                          borderRadius: BorderRadius.circular(10),
                        ),
                        child: Text(
                          _result.value.isEmpty ? 'wubi.placeholder'.tr : _result.value,
                          style: TextStyle(fontSize: 18),
                          textAlign: TextAlign.center,
                        ),
                      ),
                      SizedBox(height: 20),
                      _buildSvgDisplay(),
                    ],
                  );
                }),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSvgDisplay() {
    final code = _ggttCode.value;
    if (code == null) return SizedBox.shrink();

    final List<Widget> svgWidgets = [];

    final svgDataList = [
      ('svg1', code.svg1),
      ('svg2', code.svg2),
      ('svg3', code.svg3),
      ('svg4', code.svg4),
    ];

    final code86 = code.code86.toUpperCase();

    for (var i = 0; i < svgDataList.length; i++) {
      final (key, svgData) = svgDataList[i];
      if (svgData != null) {
        try {
          final letter = i < code86.length ? code86[i] : '';
          svgWidgets.add(
            Padding(
              padding: const EdgeInsets.all(10.0),
              child: Container(
                padding: EdgeInsets.all(10),
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.blue),
                  borderRadius: BorderRadius.circular(5),
                ),
                child: Column(
                  children: [
                    if (letter.isNotEmpty)
                      Text(
                        letter,
                        style: TextStyle(
                          fontSize: 48,
                          fontWeight: FontWeight.bold,
                          color: Colors.blue,
                        ),
                      ),
                    SizedBox(height: 10),
                    SvgPicture.string(
                      svgData,
                      width: 150,
                      height: 150,
                      fit: BoxFit.contain,
                    ),
                  ],
                ),
              ),
            ),
          );
        } catch (e) {
          svgWidgets.add(
            Padding(
              padding: const EdgeInsets.all(10.0),
              child: Text('wubi.displayError'.trParams({'key': key, 'error': '$e'})),
            ),
          );
        }
      }
    }

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(children: svgWidgets),
    );
  }
}
