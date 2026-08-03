//
//  FlutterAudioVisualizerPlugin.swift
//  flutter_audio_visualizer
//
//  Created by Flutter Audio Visualizer
//

import FlutterMacOS
import AppKit
import AVFoundation
import Accelerate

public class FlutterAudioVisualizerPlugin: NSObject, FlutterPlugin, FlutterStreamHandler {
    private var methodChannel: FlutterMethodChannel?
    private var eventChannel: FlutterEventChannel?
    private var eventSink: FlutterEventSink?
    
    // 自有播放器模式
    private var audioEngine: AVAudioEngine?
    private var audioPlayerNode: AVAudioPlayerNode?
    private var audioFile: AVAudioFile?
    
    // 外部引擎模式
    private var externalEngine: AVAudioEngine?
    private var tapInstalled = false
    
    private var fftSetup: FFTSetup?
    private var isPlaying = false
    
    private let fftSize = 1024
    private var audioFormat: AVAudioFormat?
    
    public static func register(with registrar: FlutterPluginRegistrar) {
        let methodChannel = FlutterMethodChannel(name: "flutter_audio_visualizer", binaryMessenger: registrar.messenger)
        let eventChannel = FlutterEventChannel(name: "flutter_audio_visualizer/events", binaryMessenger: registrar.messenger)
        
        let instance = FlutterAudioVisualizerPlugin()
        instance.methodChannel = methodChannel
        instance.eventChannel = eventChannel
        
        registrar.addMethodCallDelegate(instance, channel: methodChannel)
        eventChannel.setStreamHandler(instance)
    }
    
    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "initialize":
            if let args = call.arguments as? [String: Any],
               let audioSessionId = args["audioSessionId"] as? Int {
                initialize(audioSessionId: audioSessionId, result: result)
            } else {
                result(FlutterError(code: "INVALID_ARGUMENTS", message: "audioSessionId is required", details: nil))
            }
        case "start":
            start(result: result)
        case "stop":
            stop(result: result)
        case "setCaptureSize":
            if let args = call.arguments as? [String: Any],
               let size = args["size"] as? Int {
                setCaptureSize(size: size, result: result)
            } else {
                result(FlutterError(code: "INVALID_ARGUMENTS", message: "size is required", details: nil))
            }
        case "initializeWithFile":
            if let args = call.arguments as? [String: Any],
               let filePath = args["filePath"] as? String {
                initializeWithFile(filePath: filePath, result: result)
            } else {
                result(FlutterError(code: "INVALID_ARGUMENTS", message: "filePath is required", details: nil))
            }
        case "requestPermission":
            requestPermission(result: result)
        case "processBuffer":
            if let args = call.arguments as? [String: Any],
               let data = args["data"] as? [Double] {
                processExternalBuffer(data: data, result: result)
            } else {
                result(FlutterError(code: "INVALID_ARGUMENTS", message: "data is required", details: nil))
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }
    
    private func initialize(audioSessionId: Int, result: @escaping FlutterResult) {
        result(true)
    }
    
    private func initializeWithFile(filePath: String, result: @escaping FlutterResult) {
        do {
            let url = URL(fileURLWithPath: filePath)
            audioFile = try AVAudioFile(forReading: url)
            audioFormat = audioFile?.processingFormat
            setupFFT()
            result(true)
        } catch {
            result(FlutterError(code: "FILE_ERROR", message: error.localizedDescription, details: nil))
        }
    }
    
    // MARK: - 外部 PCM 数据处理（核心新功能）
    
    /// 从外部接收 PCM 数据并计算 FFT，不创建播放器
    private func processExternalBuffer(data: [Double], result: @escaping FlutterResult) {
        guard let fftSetup = fftSetup else {
            setupFFT()
            guard let fftSetup = fftSetup else {
                result(FlutterError(code: "FFT_NOT_READY", message: "FFT setup failed", details: nil))
                return
            }
            // 继续使用新 setup
            let fftData = computeFFT(from: data.map { Float($0) }, fftSetup: fftSetup)
            sendFFTData(fftData)
            result(true)
            return
        }
        
        let fftData = computeFFT(from: data.map { Float($0) }, fftSetup: fftSetup)
        sendFFTData(fftData)
        result(true)
    }
    
    private func computeFFT(from samples: [Float], fftSetup: FFTSetup) -> [Double] {
        let log2n = vDSP_Length(log2(Double(fftSize)))
        var realIn = [Float](repeating: 0, count: fftSize)
        var imagIn = [Float](repeating: 0, count: fftSize)
        
        let copyCount = min(samples.count, fftSize)
        for i in 0..<copyCount {
            realIn[i] = samples[i]
        }
        
        var magnitudes = [Float](repeating: 0, count: fftSize / 2)
        
        realIn.withUnsafeMutableBufferPointer { realInPtr in
            imagIn.withUnsafeMutableBufferPointer { imagInPtr in
                var complexSplit = DSPSplitComplex(realp: realInPtr.baseAddress!, imagp: imagInPtr.baseAddress!)
                vDSP_fft_zrip(fftSetup, &complexSplit, 1, log2n, FFTDirection(FFT_FORWARD))
                vDSP_zvmags(&complexSplit, 1, &magnitudes, 1, vDSP_Length(fftSize / 2))
            }
        }
        
        let halfSize = fftSize / 4
        return (0..<halfSize).map { Double(magnitudes[$0]) }
    }
    
    private func sendFFTData(_ data: [Double]) {
        DispatchQueue.main.async { [weak self] in
            self?.eventSink?(data)
        }
    }
    
    // MARK: - 自有播放器模式（原有功能）
    
    private func start(result: @escaping FlutterResult) {
        guard let audioFile = audioFile, let audioFormat = audioFormat else {
            result(FlutterError(code: "NOT_INITIALIZED", message: "Audio file not initialized", details: nil))
            return
        }
        
        stop(result: { _ in })
        
        audioEngine = AVAudioEngine()
        audioPlayerNode = AVAudioPlayerNode()
        
        guard let audioEngine = audioEngine, let audioPlayerNode = audioPlayerNode else {
            result(FlutterError(code: "ENGINE_ERROR", message: "Failed to create audio engine", details: nil))
            return
        }
        
        audioEngine.attach(audioPlayerNode)
        audioEngine.connect(audioPlayerNode, to: audioEngine.mainMixerNode, format: audioFormat)
        
        let bufferSize = AVAudioFrameCount(fftSize)
        audioPlayerNode.installTap(onBus: 0, bufferSize: bufferSize, format: audioFormat) { [weak self] (buffer, time) in
            self?.processAudioBuffer(buffer: buffer)
        }
        
        do {
            try audioEngine.start()
            audioPlayerNode.scheduleFile(audioFile, at: nil, completionHandler: nil)
            audioPlayerNode.play()
            isPlaying = true
            result(true)
        } catch {
            result(FlutterError(code: "ENGINE_ERROR", message: error.localizedDescription, details: nil))
        }
    }
    
    private func stop(result: @escaping FlutterResult) {
        audioPlayerNode?.removeTap(onBus: 0)
        audioEngine?.stop()
        audioPlayerNode?.stop()
        isPlaying = false
        result(true)
    }
    
    private func setCaptureSize(size: Int, result: @escaping FlutterResult) {
        result(true)
    }
    
    private func requestPermission(result: @escaping FlutterResult) {
        result(true)
    }
    
    private func setupFFT() {
        let log2n = vDSP_Length(log2(Double(fftSize)))
        fftSetup = vDSP_create_fftsetup(log2n, FFTRadix(kFFTRadix2))
    }
    
    private func processAudioBuffer(buffer: AVAudioPCMBuffer) {
        guard let channelData = buffer.floatChannelData,
              let fftSetup = fftSetup else {
            return
        }
        
        let frameCount = Int(buffer.frameLength)
        var realIn = [Float](repeating: 0, count: fftSize)
        let copyCount = min(frameCount, fftSize)
        for i in 0..<copyCount {
            realIn[i] = channelData[0][i]
        }
        
        let fftData = computeFFT(from: realIn, fftSetup: fftSetup)
        sendFFTData(fftData)
    }
    
    // MARK: - FlutterStreamHandler
    
    public func onListen(withArguments arguments: Any?, eventSink: @escaping FlutterEventSink) -> FlutterError? {
        self.eventSink = eventSink
        return nil
    }
    
    public func onCancel(withArguments arguments: Any?) -> FlutterError? {
        self.eventSink = nil
        return nil
    }
}
