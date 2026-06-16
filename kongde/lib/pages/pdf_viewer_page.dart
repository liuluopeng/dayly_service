import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

class PdfViewerPage extends StatelessWidget {
  final String url;
  final String fileName;

  const PdfViewerPage({super.key, required this.url, required this.fileName});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(fileName, overflow: TextOverflow.ellipsis),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Get.back(),
        ),
      ),
      body: SafeArea(
        child: SfPdfViewer.network(
          url,
          canShowScrollHead: true,
          canShowScrollStatus: true,
          enableDoubleTapZooming: true,
        ),
      ),
    );
  }
}
