import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:path_provider/path_provider.dart';
import 'package:webview_flutter/webview_flutter.dart';

class EpubReaderPage extends StatefulWidget {
  final String url;
  final String fileName;

  const EpubReaderPage({super.key, required this.url, required this.fileName});

  @override
  State<EpubReaderPage> createState() => _EpubReaderPageState();
}

class _EpubReaderPageState extends State<EpubReaderPage> {
  late final WebViewController _controller;
  bool _loading = true;
  HttpServer? _server;

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..addJavaScriptChannel('Flutter', onMessageReceived: (msg) {
        debugPrint('JS-CHANNEL: ${msg.message}');
        if (msg.message == 'ready' && mounted) {
          setState(() => _loading = false);
        }
      })
      ..setNavigationDelegate(NavigationDelegate(
        onPageFinished: (_) {
          debugPrint('Page finished loading');
          // Foliate has no ready callback — poll until the nav-bar becomes visible
          _controller.runJavaScript('''
            (function check() {
              var bar = document.getElementById('nav-bar');
              if (bar && bar.style.visibility === 'visible') {
                try { Flutter.postMessage('ready'); } catch(e) {}
              } else {
                setTimeout(check, 300);
              }
            })();
          ''');
        },
        onWebResourceError: (error) {
          debugPrint('WebView error: ${error.description}');
        },
      ));

    _startServer();
  }

  Future<void> _startServer() async {
    try {
      final tempDir = await getTemporaryDirectory();
      final foliateDir = Directory('${tempDir.path}/foliate');
      if (!foliateDir.existsSync()) {
        foliateDir.createSync(recursive: true);
      }

      // Copy essential foliate assets (EPUB reading only, skip PDF cmaps/fonts)
      final assetFiles = [
        'reader.html', 'reader.js', 'view.js', 'epub.js', 'epubcfi.js',
        'paginator.js', 'overlayer.js', 'text-walker.js', 'progress.js',
        'search.js', 'tts.js', 'fixed-layout.js',
        'comic-book.js', 'fb2.js', 'mobi.js', 'pdf.js',
        'ui/tree.js', 'ui/menu.js',
        'vendor/fflate.js', 'vendor/zip.js',
        'vendor/pdfjs/pdf.mjs', 'vendor/pdfjs/pdf.worker.mjs',
        'vendor/pdfjs/text_layer_builder.css',
        'vendor/pdfjs/annotation_layer_builder.css',
      ];
      for (final file in assetFiles) {
        final targetFile = File('${foliateDir.path}/$file');
        if (!targetFile.parent.existsSync()) {
          targetFile.parent.createSync(recursive: true);
        }
        try {
          final data = await rootBundle.load('assets/foliate/$file');
          targetFile.writeAsBytesSync(data.buffer.asUint8List());
        } catch (e) {
          debugPrint('Failed to copy $file: $e');
        }
      }
      debugPrint('Foliate assets copied');

      _server = await HttpServer.bind('127.0.0.1', 0);
      final port = _server!.port;
      debugPrint('Foliate server on port $port');

      // Read EPUB file
      String epubPath = widget.url;
      if (epubPath.startsWith('file://')) {
        epubPath = Uri.parse(epubPath).toFilePath();
      }
      final epubFile = File(epubPath);
      final epubBytes = epubFile.existsSync() ? epubFile.readAsBytesSync() : null;
      final isRemoteUrl = widget.url.startsWith('http://') || widget.url.startsWith('https://');

      _server!.listen((HttpRequest request) async {
        final path = request.uri.path == '/' ? '/reader.html' : request.uri.path;
        debugPrint('Serve: $path');

        if (path == '/book.epub' && epubBytes != null) {
          request.response.headers.contentType =
              ContentType.parse('application/epub+zip');
          request.response.add(epubBytes);
          await request.response.close();
          return;
        }

        final file = File('${foliateDir.path}$path');
        if (file.existsSync()) {
          final ext = path.split('.').last;
          final mimeTypes = {
            'html': 'text/html', 'js': 'application/javascript',
            'mjs': 'application/javascript', 'css': 'text/css',
            'json': 'application/json', 'bcmap': 'application/octet-stream',
            'pfb': 'application/octet-stream', 'ttf': 'font/ttf',
            'woff': 'font/woff', 'woff2': 'font/woff2',
            'wasm': 'application/wasm', 'map': 'application/json',
            'svg': 'image/svg+xml', 'png': 'image/png',
          };
          request.response.headers.contentType =
              ContentType.parse(mimeTypes[ext] ?? 'application/octet-stream');
          request.response.add(file.readAsBytesSync());
        } else {
          request.response.statusCode = 404;
          request.response.write('Not found: $path');
        }
        await request.response.close();
      });

      final epubUrl = isRemoteUrl
          ? widget.url
          : 'http://127.0.0.1:$port/book.epub';
      final readerUrl = 'http://127.0.0.1:$port/reader.html?url=${Uri.encodeComponent(epubUrl)}';
      debugPrint('Loading: $readerUrl');
      await _controller.loadRequest(Uri.parse(readerUrl));
    } catch (e) {
      debugPrint('Server error: $e');
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  void dispose() {
    _server?.close();
    super.dispose();
  }

  void _handleKey(RawKeyEvent event) {
    if (event is! RawKeyDownEvent) return;
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.arrowLeft) {
      _controller.runJavaScript('try{reader.view.goLeft()}catch(e){}');
    } else if (key == LogicalKeyboardKey.arrowRight) {
      _controller.runJavaScript('try{reader.view.goRight()}catch(e){}');
    }
  }

  @override
  Widget build(BuildContext context) {
    return RawKeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKey: _handleKey,
      child: Scaffold(
        appBar: AppBar(
          title: Text(
            widget.fileName,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 15),
          ),
          leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Get.back(),
          ),
        ),
        body: SafeArea(
          child: Stack(
            children: [
              WebViewWidget(controller: _controller),
              if (_loading) const Center(child: CircularProgressIndicator()),
            ],
          ),
        ),
      ),
    );
  }
}
