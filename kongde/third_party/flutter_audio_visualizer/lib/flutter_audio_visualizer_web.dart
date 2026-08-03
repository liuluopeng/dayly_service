import 'dart:async';
import 'dart:html' as html;
import 'dart:typed_data';
import 'package:js/js.dart';

import 'package:flutter/services.dart';
import 'package:flutter_web_plugins/flutter_web_plugins.dart';

@JS('AudioContext')
external dynamic AudioContextConstructor();

@JS()
@anonymous
@staticInterop
class AudioContext {
  external AnalyserNode createAnalyser();
  external MediaElementAudioSourceNode createMediaElementSource(
      html.AudioElement element);
  external AudioDestinationNode get destination;
  external String get state;
  external Future<dynamic> resume();
}

@JS()
@anonymous
@staticInterop
class AnalyserNode extends AudioNode {
  external set fftSize(int value);
  external int get fftSize;
  external int get frequencyBinCount;
  external void getByteFrequencyData(Uint8List array);
}

@JS()
@anonymous
@staticInterop
class AudioNode {
  external void connect(AudioNode destination);
}

@JS()
@anonymous
@staticInterop
class MediaElementAudioSourceNode extends AudioNode {}

@JS()
@anonymous
@staticInterop
class AudioDestinationNode extends AudioNode {}

class FlutterAudioVisualizerPluginWeb {
  static void registerWith(Registrar registrar) {
    final methodChannel = MethodChannel(
      'flutter_audio_visualizer',
      const StandardMethodCodec(),
      registrar.messenger,
    );

    final eventChannel = EventChannel(
      'flutter_audio_visualizer/events',
      const StandardMethodCodec(),
      registrar.messenger,
    );

    final instance = FlutterAudioVisualizerPluginWeb();
    methodChannel.setMethodCallHandler(instance.handleMethodCall);
    eventChannel.receiveBroadcastStream().listen((event) {
      instance._eventSink?.add(event);
    });
  }

  AudioContext? _audioContext;
  AnalyserNode? _analyser;
  html.AudioElement? _audioElement;
  html.MediaStream? _mediaStream;
  StreamController<List<double>>? _eventController;
  StreamSubscription? _analysisSubscription;

  StreamSink<List<double>>? get _eventSink => _eventController?.sink;

  Future<dynamic> handleMethodCall(MethodCall call) async {
    switch (call.method) {
      case 'initialize':
        final audioSessionId = call.arguments as int;
        return _initialize(audioSessionId);
      case 'initializeWithFile':
        final filePath = call.arguments as String;
        return _initializeWithFile(filePath);
      case 'start':
        return _start();
      case 'stop':
        return _stop();
      case 'setCaptureSize':
        final size = call.arguments as int;
        return _setCaptureSize(size);
      case 'requestPermission':
        return _requestPermission();
      default:
        throw UnimplementedError('${call.method} is not implemented');
    }
  }

  Future<bool> _initialize(int audioSessionId) async {
    _audioContext = AudioContextConstructor();
    return true;
  }

  Future<bool> _initializeWithFile(String filePath) async {
    try {
      _audioContext = AudioContextConstructor();
      _audioElement = html.AudioElement(filePath);
      _analyser = _audioContext!.createAnalyser();

      _analyser!.fftSize = 1024;

      final source = _audioContext!.createMediaElementSource(_audioElement!);
      source.connect(_analyser!);
      _analyser!.connect(_audioContext!.destination);

      return true;
    } catch (e) {
      print('Error initializing audio file: $e');
      return false;
    }
  }

  Future<bool> _start() async {
    try {
      if (_audioElement != null) {
        _audioElement!.play();
      }

      _eventController = StreamController<List<double>>.broadcast();

      _analysisSubscription = Stream.periodic(
        const Duration(milliseconds: 16),
        (_) => _analyzeAudio(),
      ).listen((data) {
        _eventSink?.add(data);
      });

      return true;
    } catch (e) {
      print('Error starting audio: $e');
      return false;
    }
  }

  Future<bool> _stop() async {
    try {
      await _analysisSubscription?.cancel();
      _analysisSubscription = null;

      await _eventController?.close();
      _eventController = null;

      _audioElement?.pause();

      return true;
    } catch (e) {
      print('Error stopping audio: $e');
      return false;
    }
  }

  Future<bool> _setCaptureSize(int size) async {
    if (_analyser != null) {
      _analyser!.fftSize = size;
    }
    return true;
  }

  Future<bool> _requestPermission() async {
    try {
      if (_audioContext != null && _audioContext!.state == 'suspended') {
        await _audioContext!.resume();
      }
      return true;
    } catch (e) {
      print('Error requesting permission: $e');
      return false;
    }
  }

  List<double> _analyzeAudio() {
    if (_analyser == null) {
      return [];
    }

    final bufferLength = _analyser!.frequencyBinCount;
    final dataArray = Uint8List(bufferLength);
    _analyser!.getByteFrequencyData(dataArray);

    final result = <double>[];
    for (var i = 0; i < bufferLength; i++) {
      result.add(dataArray[i].toDouble());
    }

    return result;
  }
}
