import Cocoa
import FlutterMacOS
import AVFoundation
import Accelerate

/// macOS 专用音频播放器：AVAudioEngine + 实时 PCM tap
class MacOSAudioPlayer: NSObject, FlutterPlugin, FlutterStreamHandler {
    private var audioEngine: AVAudioEngine?
    private var playerNode: AVAudioPlayerNode?
    private var audioFile: AVAudioFile?
    private var audioFormat: AVAudioFormat?
    
    private var fftSetup: FFTSetup?
    private var pcmEventSink: FlutterEventSink?
    private var positionEventSink: FlutterEventSink?
    
    private var positionTimer: Timer?
    private var pausedPosition: TimeInterval = 0
    
    private let fftSize = 1024
    
    // MARK: - Flutter Plugin
    
    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(name: "com.kongde/macos_audio", binaryMessenger: registrar.messenger)
        let pcmEvent = FlutterEventChannel(name: "com.kongde/macos_audio/pcm", binaryMessenger: registrar.messenger)
        let posEvent = FlutterEventChannel(name: "com.kongde/macos_audio/position", binaryMessenger: registrar.messenger)
        
        let instance = MacOSAudioPlayer()
        registrar.addMethodCallDelegate(instance, channel: channel)
        pcmEvent.setStreamHandler(instance)
        posEvent.setStreamHandler(instance)
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
        default:
            result(FlutterMethodNotImplemented)
        }
    }
    
    // MARK: - 播放控制
    
    func loadAndPlay(filePath: String) throws {
        stop()
        
        NSLog("[MacOSAudioPlayer] loadAndPlay: \(filePath)")
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
        
        // 在 playerNode 上安装 tap，在音量处理之前拿 PCM 数据
        let bufferSize = AVAudioFrameCount(fftSize)
        NSLog("[MacOSAudioPlayer] Installing tap on playerNode, bufferSize=\(bufferSize)")
        player.installTap(onBus: 0, bufferSize: bufferSize, format: audioFormat) { [weak self] (buffer, time) in
            self?.handleBuffer(buffer)
        }
        
        let log2n = vDSP_Length(log2(Double(fftSize)))
        fftSetup = vDSP_create_fftsetup(log2n, FFTRadix(kFFTRadix2))
        
        NSLog("[MacOSAudioPlayer] Starting engine")
        try engine.start()
        player.scheduleFile(audioFile!, at: nil, completionHandler: nil)
        player.volume = 0  // 静音，只为获取频谱
        player.play()
        NSLog("[MacOSAudioPlayer] Player started")
        
        startPositionTimer()
    }
    
    func stop() {
        stopPositionTimer()
        playerNode?.removeTap(onBus: 0)
        audioEngine?.stop()
        playerNode?.stop()
        audioEngine = nil
        playerNode = nil
        audioFile = nil
        pausedPosition = 0
        DispatchQueue.main.async {
            self.pcmEventSink?([])
        }
    }
    
    func pause() {
        playerNode?.removeTap(onBus: 0)
        playerNode?.pause()
        pausedPosition = getPosition()
        stopPositionTimer()
    }
    
    func resume() {
        // 重新安装 tap
        if let playerNode = playerNode, let audioFormat = audioFormat {
            let bufferSize = AVAudioFrameCount(fftSize)
            playerNode.installTap(onBus: 0, bufferSize: bufferSize, format: audioFormat) { [weak self] (buffer, time) in
                self?.handleBuffer(buffer)
            }
        }
        playerNode?.play()
        startPositionTimer()
    }
    
    var isPlaying: Bool {
        return playerNode?.isPlaying ?? false
    }
    
    func getPosition() -> TimeInterval {
        guard let playerNode = playerNode,
              let lastRenderTime = playerNode.lastRenderTime,
              let playerTime = playerNode.playerTime(forNodeTime: lastRenderTime),
              let audioFormat = audioFormat else {
            return pausedPosition
        }
        return Double(playerTime.sampleTime) / audioFormat.sampleRate
    }
    
    func getDuration() -> TimeInterval {
        guard let audioFile = audioFile, let audioFormat = audioFormat else { return 0 }
        return Double(audioFile.length) / audioFormat.sampleRate
    }
    
    // MARK: - 位置定时器
    
    private func startPositionTimer() {
        stopPositionTimer()
        positionTimer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [weak self] _ in
            guard let self = self, self.isPlaying else { return }
            let pos = Int(self.getPosition() * 1000)
            DispatchQueue.main.async {
                self.positionEventSink?(pos)
            }
        }
    }
    
    private func stopPositionTimer() {
        positionTimer?.invalidate()
        positionTimer = nil
    }
    
    // MARK: - PCM / FFT
    
    private func handleBuffer(_ buffer: AVAudioPCMBuffer) {
        guard let channelData = buffer.floatChannelData else { return }
        
        let frameCount = Int(buffer.frameLength)
        var samples = [Float](repeating: 0, count: min(frameCount, fftSize))
        let copyCount = min(frameCount, fftSize)
        for i in 0..<copyCount {
            samples[i] = channelData[0][i]
        }
        
        // 打印原始 PCM 前几个值
        let pcmPreview = samples.prefix(8).map { String(format: "%.4f", $0) }.joined(separator: ", ")
        NSLog("[MacOSAudioPlayer] PCM: [\(pcmPreview)] frameCount=\(frameCount)")
        
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
        
        // 打印前几个频率的幅值
        let preview = fftData.prefix(8).map { String(format: "%.1f", $0) }.joined(separator: ", ")
        NSLog("[MacOSAudioPlayer] FFT: [\(preview)] playing=\(playerNode?.isPlaying ?? false)")
        
        DispatchQueue.main.async { [weak self] in
            self?.pcmEventSink?(fftData)
        }
    }
    
    // MARK: - FlutterStreamHandler
    
    public func onListen(withArguments arguments: Any?, eventSink: @escaping FlutterEventSink) -> FlutterError? {
        NSLog("[MacOSAudioPlayer] onListen called")
        if let args = arguments as? [String: Any], let type = args["type"] as? String {
            if type == "position" {
                positionEventSink = eventSink
            }
        } else {
            pcmEventSink = eventSink
        }
        return nil
    }
    
    public func onCancel(withArguments arguments: Any?) -> FlutterError? {
        if let args = arguments as? [String: Any], let type = args["type"] as? String {
            if type == "position" {
                positionEventSink = nil
            }
        } else {
            pcmEventSink = nil
        }
        return nil
    }
}
