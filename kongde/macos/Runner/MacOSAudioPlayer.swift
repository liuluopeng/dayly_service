import Cocoa
import FlutterMacOS
import AVFoundation
import Accelerate

/// macOS 专用音频播放器：AVAudioEngine + 实时 PCM tap
class MacOSAudioPlayer: NSObject, FlutterPlugin {
    private static var channel: FlutterMethodChannel?
    private var audioEngine: AVAudioEngine?
    private var playerNode: AVAudioPlayerNode?
    private var audioFile: AVAudioFile?
    private var audioFormat: AVAudioFormat?
    
    private var fftSetup: FFTSetup?
    private let fftSize = 1024
    private let tapBufferSize: AVAudioFrameCount = 256  // 更小的 buffer → 更高帧率
    
    public static func register(with registrar: FlutterPluginRegistrar) {
        channel = FlutterMethodChannel(name: "com.kongde/macos_audio", binaryMessenger: registrar.messenger)
        let instance = MacOSAudioPlayer()
        registrar.addMethodCallDelegate(instance, channel: channel!)
    }
    
    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "play":
            guard let args = call.arguments as? [String: Any],
                  let filePath = args["filePath"] as? String else {
                result(FlutterError(code: "INVALID_ARGS", message: "filePath required", details: nil))
                return
            }
            do {
                try loadAndPlay(filePath: filePath)
                result(true)
            } catch {
                result(FlutterError(code: "PLAY_ERROR", message: error.localizedDescription, details: nil))
            }
        case "stop":
            stop()
            result(true)
        case "pause":
            pause()
            result(true)
        case "resume":
            resume()
            result(true)
        case "isPlaying":
            result(playerNode?.isPlaying ?? false)
        case "getPosition":
            result(Int(getPosition() * 1000))
        case "getDuration":
            result(Int(getDuration() * 1000))
        case "seek":
            guard let args = call.arguments as? [String: Any],
                  let positionMs = args["positionMs"] as? Int else {
                result(FlutterError(code: "INVALID_ARGS", message: "positionMs required", details: nil))
                return
            }
            seek(to: TimeInterval(positionMs) / 1000.0)
            result(true)
        default:
            result(FlutterMethodNotImplemented)
        }
    }
    
    // MARK: - 播放控制
    
    func loadAndPlay(filePath: String) throws {
        stop()
        
        let url = URL(fileURLWithPath: filePath)
        audioFile = try AVAudioFile(forReading: url)
        audioFormat = audioFile?.processingFormat
        
        guard let audioFormat = audioFormat else {
            throw NSError(domain: "MacOSAudioPlayer", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid audio format"])
        }
        
        audioEngine = AVAudioEngine()
        playerNode = AVAudioPlayerNode()
        
        guard let engine = audioEngine, let player = playerNode else {
            throw NSError(domain: "MacOSAudioPlayer", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to create engine"])
        }
        
        engine.attach(player)
        engine.connect(player, to: engine.mainMixerNode, format: audioFormat)
        
        let bufferSize = tapBufferSize
        player.installTap(onBus: 0, bufferSize: bufferSize, format: audioFormat) { [weak self] (buffer, time) in
            self?.handleBuffer(buffer)
        }
        
        let log2n = vDSP_Length(log2(Double(fftSize)))
        fftSetup = vDSP_create_fftsetup(log2n, FFTRadix(kFFTRadix2))
        
        try engine.start()
        player.scheduleFile(audioFile!, at: nil, completionHandler: nil)
        player.volume = 0
        player.play()
    }
    
    func stop() {
        playerNode?.removeTap(onBus: 0)
        audioEngine?.stop()
        playerNode?.stop()
        audioEngine = nil
        playerNode = nil
        audioFile = nil
    }
    
    func pause() {
        playerNode?.pause()
    }
    
    func resume() {
        playerNode?.play()
    }
    
    func seek(to time: TimeInterval) {
        guard let player = playerNode, let audioFile = audioFile, let audioFormat = audioFormat else { return }
        
        let wasPlaying = player.isPlaying
        player.stop()
        player.removeTap(onBus: 0)
        
        let framePosition = AVAudioFramePosition(time * audioFormat.sampleRate)
        let totalFrames = audioFile.length
        let framesToSchedule = totalFrames - framePosition
        guard framesToSchedule > 0 else { return }
        
        // 从指定位置读取音频帧
        audioFile.framePosition = framePosition
        let buffer = AVAudioPCMBuffer(pcmFormat: audioFormat, frameCapacity: AVAudioFrameCount(framesToSchedule))!
        try? audioFile.read(into: buffer, frameCount: AVAudioFrameCount(framesToSchedule))
        
        let bufferSize = tapBufferSize
        player.installTap(onBus: 0, bufferSize: bufferSize, format: audioFormat) { [weak self] (buffer, time) in
            self?.handleBuffer(buffer)
        }
        
        player.scheduleBuffer(buffer, completionHandler: nil)
        if wasPlaying {
            player.play()
        }
    }
    
    var isPlaying: Bool {
        return playerNode?.isPlaying ?? false
    }
    
    func getPosition() -> TimeInterval {
        guard let playerNode = playerNode,
              let lastRenderTime = playerNode.lastRenderTime,
              let playerTime = playerNode.playerTime(forNodeTime: lastRenderTime),
              let audioFormat = audioFormat else {
            return 0
        }
        return Double(playerTime.sampleTime) / audioFormat.sampleRate
    }
    
    func getDuration() -> TimeInterval {
        guard let audioFile = audioFile, let audioFormat = audioFormat else { return 0 }
        return Double(audioFile.length) / audioFormat.sampleRate
    }
    
    // MARK: - PCM / FFT → 通过 MethodChannel 推送到 Dart
    
    private func handleBuffer(_ buffer: AVAudioPCMBuffer) {
        guard let player = playerNode, player.isPlaying else { return }
        
        guard let channelData = buffer.floatChannelData else { return }
        
        let frameCount = Int(buffer.frameLength)
        var samples = [Float](repeating: 0, count: min(frameCount, fftSize))
        let copyCount = min(frameCount, fftSize)
        for i in 0..<copyCount {
            samples[i] = channelData[0][i]
        }
        
        guard let fftSetup = fftSetup else { return }
        let log2n = vDSP_Length(log2(Double(fftSize)))
        var realIn = samples
        var imagIn = [Float](repeating: 0, count: fftSize)
        
        var magnitudes = [Float](repeating: 0, count: fftSize / 2)
        
        realIn.withUnsafeMutableBufferPointer { realInPtr in
            imagIn.withUnsafeMutableBufferPointer { imagInPtr in
                var complexSplit = DSPSplitComplex(realp: realInPtr.baseAddress!, imagp: imagInPtr.baseAddress!)
                vDSP_fft_zrip(fftSetup, &complexSplit, 1, log2n, FFTDirection(FFT_FORWARD))
                vDSP_zvmags(&complexSplit, 1, &magnitudes, 1, vDSP_Length(fftSize / 2))
            }
        }
        
        let halfSize = fftSize / 4
        let fftData = (0..<halfSize).map { Double(magnitudes[$0]) }
        
        // 通过 MethodChannel 推送 FFT 数据到 Dart
        DispatchQueue.main.async {
            MacOSAudioPlayer.channel?.invokeMethod("onFFT", arguments: fftData)
        }
    }
}
