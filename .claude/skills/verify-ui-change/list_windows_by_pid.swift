import CoreGraphics
import Foundation

// Usage: swift list_windows_by_pid.swift <PID>
// Prints "<CGWindowID>\t<window title>" for every on-screen window owned by
// that process — used to find the window ID to pass to `screencapture -l`
// without resorting to a full-screen capture (which can expose whatever
// else is on the user's desktop).
let pid: Int32 = Int32(CommandLine.arguments[1])!
let options = CGWindowListOption(arrayLiteral: .optionOnScreenOnly)
guard let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: AnyObject]] else {
    exit(1)
}
for w in list {
    let ownerPid = w[kCGWindowOwnerPID as String] as? Int32 ?? -1
    if ownerPid == pid {
        let num = w[kCGWindowNumber as String] as? Int ?? -1
        let name = w[kCGWindowName as String] as? String ?? ""
        print("\(num)\t\(name)")
    }
}
