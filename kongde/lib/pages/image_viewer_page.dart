import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;
import 'package:kongde/config/app_config.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ImageViewerPage extends StatefulWidget {
  final String imageUrl;
  final String fileName;

  const ImageViewerPage({
    super.key,
    required this.imageUrl,
    required this.fileName,
  });

  @override
  State<ImageViewerPage> createState() => _ImageViewerPageState();
}

class _ImageViewerPageState extends State<ImageViewerPage> {
  Uint8List? _imageBytes;
  bool _isLoading = true;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _loadImage();
  }

  Future<void> _loadImage() async {
    try {
      final url = widget.imageUrl.startsWith('http')
          ? widget.imageUrl
          : '${AppConfig.instance.serverUrl}${widget.imageUrl}';

      final response = await http.get(
        Uri.parse(url),
        headers: AppConfig.instance.getApiHeaders(),
      );

      if (response.statusCode == 200) {
        setState(() {
          _imageBytes = response.bodyBytes;
          _isLoading = false;
        });
      } else {
        setState(() {
          _errorMessage = 'imageViewer.loadFailedHttp'.trParams({'statusCode': '${response.statusCode}'});
          _isLoading = false;
        });
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'imageViewer.loadFailed'.trParams({'error': '$e'});
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: widget.fileName),
      body: SafeArea(
        child: _isLoading
            ? const Center(child: CircularProgressIndicator())
            : _errorMessage != null
            ? Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(
                      Icons.error_outline,
                      size: 48,
                      color: Colors.red,
                    ),
                    const SizedBox(height: 16),
                    Text(_errorMessage!),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: _loadImage,
                      child: Text('common.retry'.tr),
                    ),
                  ],
                ),
              )
            : InteractiveViewer(
                minScale: 0.1,
                maxScale: 2000.0,
                child: Center(
                  child: Image.memory(
                    _imageBytes!,
                    fit: BoxFit.contain,
                    width: double.infinity,
                    height: double.infinity,
                  ),
                ),
              ),
      ),
    );
  }
}
