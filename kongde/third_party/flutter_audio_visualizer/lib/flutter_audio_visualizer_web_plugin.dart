import 'package:flutter_web_plugins/flutter_web_plugins.dart';
import 'flutter_audio_visualizer_web.dart';

void registerPlugins(PluginRegistry registry) {
  FlutterAudioVisualizerPluginWeb.registerWith(registry.registrarFor(
    FlutterAudioVisualizerPluginWeb,
  ));
}
