import Cocoa
import FlutterMacOS

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
