# Swift & Objective-C Backend

Fish provides first-class support for Swift and Objective-C projects across macOS, iOS, and Linux.

## Detection
Fish automatically detects Swift projects by inspecting:
- `Package.swift` (Swift Package Manager)
- `*.xcodeproj` / `*.xcworkspace` (Xcode projects)

## Supported Commands
```bash
fish build     # Compiles SwiftPM modules using swift build
fish test      # Executes XCTest test suites
fish check     # Runs swiftc syntax and typecheck validation
```

## Configuration in `fish.toml`
```toml
backend = "swift"
jobs = 4

[cache]
enabled = true
```
