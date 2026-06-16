import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class ScanPage extends StatefulWidget {
  const ScanPage({super.key});

  @override
  State<ScanPage> createState() => _ScanPageState();
}

class _ScanPageState extends State<ScanPage> {
  String _scannedText = '';
  bool _isScanning = true;
  final MobileScannerController _controller = MobileScannerController();

  void _onDetect(BarcodeCapture capture) {
    final barcode = capture.barcodes.first;
    if (barcode.rawValue != null && _isScanning) {
      setState(() {
        _scannedText = barcode.rawValue!;
        _isScanning = false;
      });
      HapticFeedback.lightImpact();
    }
  }

  void _copyToClipboard() {
    Clipboard.setData(ClipboardData(text: _scannedText));
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('common.copied'.tr)));
  }

  void _resetScan() {
    setState(() {
      _scannedText = '';
      _isScanning = true;
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(
        title: 'scan.title'.tr,
        actions: [
          if (_scannedText.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: _resetScan,
              tooltip: 'scan.rescan'.tr,
            ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: Stack(
                children: [
                  MobileScanner(controller: _controller, onDetect: _onDetect),
                  if (_isScanning)
                    Container(
                      decoration: BoxDecoration(
                        color: Colors.black.withOpacity(0.3),
                      ),
                      child: Center(
                        child: Container(
                          width: 250,
                          height: 250,
                          decoration: BoxDecoration(
                            border: Border.all(color: Colors.white, width: 2),
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Center(
                            child: Text(
                              'scan.instruction'.tr,
                              style:
                                  TextStyle(color: Colors.white, fontSize: 16),
                            ),
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
            if (_scannedText.isNotEmpty)
              Container(
                padding: const EdgeInsets.all(16),
                color: Colors.white,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text(
                      'scan.result'.tr,
                      style:
                          TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 8),
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.grey[100],
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(color: Colors.grey[300]!),
                      ),
                      child: Text(
                        _scannedText,
                        style: const TextStyle(fontSize: 14),
                      ),
                    ),
                    const SizedBox(height: 16),
                    ElevatedButton.icon(
                      onPressed: _copyToClipboard,
                      icon: const Icon(Icons.copy),
                      label: Text('scan.copyText'.tr),
                      style: ElevatedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(vertical: 16),
                      ),
                    ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}
