import Cocoa
import FlutterMacOS
import AVFoundation

// 环境监听：麦克风直通耳机输出（AVAudioEngine 输入 → 混音器 → 输出）
class SurroundListenManager {
    static let shared = SurroundListenManager()
    private let engine = AVAudioEngine()
    private let micMixer = AVAudioMixerNode()
    private let eq = AVAudioUnitEQ(numberOfBands: 1)
    private var isRunning = false
    private var isPaused = false

    private init() {
        engine.attach(micMixer)
        engine.attach(eq)
    }

    func start() throws {
        if isRunning { return }
        let input = engine.inputNode
        let format = input.inputFormat(forBus: 0)
        engine.connect(input, to: micMixer, format: format)
        engine.connect(micMixer, to: eq, format: format)
        engine.connect(eq, to: engine.mainMixerNode, format: format)
        micMixer.outputVolume = 1.0
        if let band = eq.bands.first {
            band.filterType = .parametric
            band.frequency = 1000
            band.bandwidth = 2.0
            band.gain = 0.0
            band.bypass = false
        }
        try engine.start()
        isRunning = true
        isPaused = false
    }

    func stop() {
        guard isRunning else { return }
        engine.stop()
        isRunning = false
        isPaused = false
    }

    func setGain(_ gain: Float) {
        if gain <= 1.0 {
            micMixer.outputVolume = gain
            if let band = eq.bands.first { band.gain = 0.0 }
        } else {
            micMixer.outputVolume = 1.0
            let db = 20.0 * log10(Double(gain))
            let clamped = max(-12.0, min(24.0, db))
            if let band = eq.bands.first { band.gain = Float(clamped) }
        }
    }

    func pause() {
        guard isRunning, !isPaused else { return }
        engine.pause()
        isPaused = true
    }

    func resume() throws {
        guard isRunning, isPaused else { return }
        try engine.start()
        isPaused = false
    }
}

@main
class AppDelegate: FlutterAppDelegate {
  private weak var openFileChannel: FlutterMethodChannel?
  private var pendingFilePaths: [String] = []

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }

  override func applicationDidFinishLaunching(_ notification: Notification) {
    super.applicationDidFinishLaunching(notification)
    NSLog("[kongde] applicationDidFinishLaunching")
  }

  override func application(_ sender: NSApplication, openFile filename: String) -> Bool {
    NSLog("[kongde] application:openFile: '\(filename)'")
    activateSecurityScope(filename)
    enqueueOrSend(filename)
    return true
  }

  override func application(_ sender: NSApplication, openFiles filenames: [String]) {
    NSLog("[kongde] application:openFiles: \(filenames)")
    for filename in filenames {
      activateSecurityScope(filename)
      enqueueOrSend(filename)
    }
  }

  private func activateSecurityScope(_ path: String) {
    let url = URL(fileURLWithPath: path)
    _ = url.startAccessingSecurityScopedResource()
    _ = url.deletingLastPathComponent().startAccessingSecurityScopedResource()
  }

  func enqueueOrSend(_ path: String) {
    if let channel = openFileChannel {
      NSLog("[kongde] channel ready, sending: \(path)")
      channel.invokeMethod("openFile", arguments: path)
    } else {
      NSLog("[kongde] channel NOT ready, queueing: \(path)")
      pendingFilePaths.append(path)
    }
  }

  func flushOpenFileQueue(channel: FlutterMethodChannel) {
    openFileChannel = channel
    let paths = pendingFilePaths
    pendingFilePaths.removeAll()
    NSLog("[kongde] Flushing \(paths.count) pending files")
    for path in paths {
      channel.invokeMethod("openFile", arguments: path)
    }
  }
}
