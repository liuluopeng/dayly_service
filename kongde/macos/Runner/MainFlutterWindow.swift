import Cocoa
import FlutterMacOS

/// 覆蓋在 Flutter 視圖上的拖放接收層
class DragReceiverView: NSView {
  private var channel: FlutterMethodChannel?

  init(channel: FlutterMethodChannel?) {
    self.channel = channel
    super.init(frame: .zero)
    registerForDraggedTypes([.fileURL, .URL])
  }

  required init?(coder: NSCoder) {
    super.init(coder: coder)
    registerForDraggedTypes([.fileURL, .URL])
  }

  override func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
    return .copy
  }

  override func draggingUpdated(_ sender: NSDraggingInfo) -> NSDragOperation {
    return .copy
  }

  override func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
    let pasteboard = sender.draggingPasteboard
    guard let urls = pasteboard.readObjects(forClasses: [NSURL.self], options: nil) as? [URL]
    else { return false }
    for url in urls {
      guard url.isFileURL else { continue }
      _ = url.startAccessingSecurityScopedResource()
      _ = url.deletingLastPathComponent().startAccessingSecurityScopedResource()
      let path = url.path
      NSLog("[kongde] DragReceiver: file dropped: \(path)")
      channel?.invokeMethod("openFile", arguments: path)
    }
    return !urls.isEmpty
  }
}

class MainFlutterWindow: NSWindow {
  private var globalMonitor: Any?

  override func awakeFromNib() {
    NSLog("[kongde] MainFlutterWindow.awakeFromNib")
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    // 注册原生音频播放器
    let audioRegistrar = flutterViewController.registrar(forPlugin: "MacOSAudioPlayer")
    MacOSAudioPlayer.register(with: audioRegistrar)

    let binaryMessenger = flutterViewController.engine.binaryMessenger

    // 环境监听 channel（麦克风直通耳机）
    let surroundChannel = FlutterMethodChannel(
      name: "kongde/native_audio",
      binaryMessenger: binaryMessenger
    )
    surroundChannel.setMethodCallHandler { (call, result) in
      switch call.method {
      case "surroundStart":
        do {
          try SurroundListenManager.shared.start()
          result(true)
        } catch {
          result(FlutterError(code: "start_error", message: error.localizedDescription, details: nil))
        }
      case "surroundStop":
        SurroundListenManager.shared.stop()
        result(true)
      case "surroundSetGain":
        if let args = call.arguments as? [String: Any], let gain = args["gain"] as? Double {
          SurroundListenManager.shared.setGain(Float(gain))
          result(true)
        } else {
          result(FlutterError(code: "bad_args", message: "gain missing", details: nil))
        }
      case "surroundPause":
        SurroundListenManager.shared.pause()
        result(true)
      case "surroundResume":
        do {
          try SurroundListenManager.shared.resume()
          result(true)
        } catch {
          result(FlutterError(code: "resume_error", message: error.localizedDescription, details: nil))
        }
      default:
        result(FlutterMethodNotImplemented)
      }
    }

    // 窗口控制 channel
    let windowChannel = FlutterMethodChannel(
      name: "com.kongde/window",
      binaryMessenger: binaryMessenger
    )
    windowChannel.setMethodCallHandler { (call, result) in
      switch call.method {
      case "hide":
        NSApp.hide(nil)
        result(true)
      case "show":
        NSApp.unhide(nil)
        NSApp.activate(ignoringOtherApps: true)
        self.makeKeyAndOrderFront(nil)
        result(true)
      case "isVisible":
        result(self.isVisible)
      default:
        result(FlutterMethodNotImplemented)
      }
    }

    // 全局热键: CMD+CTRL+S
    globalMonitor = NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { event in
      let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
      let isCmd = flags.contains(.command)
      let isCtrl = flags.contains(.control)
      if isCmd && isCtrl && event.charactersIgnoringModifiers == "s" {
        DispatchQueue.main.async {
          if NSApp.isHidden {
            NSApp.unhide(nil)
            NSApp.activate(ignoringOtherApps: true)
            self.makeKeyAndOrderFront(nil)
          } else {
            NSApp.hide(nil)
          }
        }
      }
    }

    let channel = FlutterMethodChannel(
      name: "com.example.kongde/open_file",
      binaryMessenger: binaryMessenger
    )

    let dragView = DragReceiverView(channel: channel)
    dragView.frame = flutterViewController.view.bounds
    dragView.autoresizingMask = [.width, .height]
    flutterViewController.view.addSubview(dragView)

    if let appDelegate = NSApplication.shared.delegate as? AppDelegate {
      appDelegate.flushOpenFileQueue(channel: channel)
    }

    super.awakeFromNib()
  }

  deinit {
    if let monitor = globalMonitor {
      NSEvent.removeMonitor(monitor)
    }
  }
}
