# Hướng dẫn Kiến trúc Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này cung cấp cái nhìn tổng quan kỹ thuật toàn diện về kiến trúc hệ thống của Fish, các module lõi và đường ống thực thi (execution pipeline).

---

## Tổng quan Hệ thống

Fish là hệ thống điều phối biên dịch hiệu năng cao, ưu tiên bộ nhớ đệm (cache-first), được thiết kế chuyên biệt cho monorepo đa ngôn ngữ và phát triển phân tán. Thay vì thay thế các trình biên dịch gốc, Fish đóng vai trò là tầng điều phối thông minh giữa các chuỗi công cụ (toolchains), quản lý đồ thị phụ thuộc DAG, bộ nhớ đệm theo nội dung (CAS), môi trường cách ly (hermetic sandbox), và thực thi công việc song song (work-stealing).

```text
┌─────────────────────────────────────────────────────────────┐
│                    fish-cli / Web UI                        │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│       fish-core (Discovery, Toolchains, compile_commands)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│           fish-graph (DAG & Algebraic Query Engine)         │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│   fish-scheduler (Governor, Jobserver, Racing, Watcher)     │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────┐┌──────────────▼──────────────┐
│ fish-executor & Middleware  ││  fish-cache & fish-cas      │
└──────────────┬──────────────┘└──────────────┬──────────────┘
               │                              │
┌──────────────▼──────────────────────────────▼──────────────┐
│      11+ Language Backends & Distributed Worker Network     │
└─────────────────────────────────────────────────────────────┘
```

---

## Các Crate Lõi và Trách nhiệm

### 1. Khám phá Không gian làm việc (`fish-core`)
- **Phát hiện Manifest**: Quét và phân tích cú pháp `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`, `Package.swift`, `pubspec.yaml`, `build.zig`, `Dockerfile`.
- **Cơ sở dữ liệu biên dịch**: Tự động sinh tệp chuẩn `compile_commands.json` cho Clangd và các IDE (`CompilationDatabase`).
- **Quản lý Toolchain cách ly**: Quản lý và cô lập đường dẫn cũng như biến môi trường của các trình biên dịch (`ToolchainRegistry`, `ToolchainSpec`).
- **Lọc vi mô đầu vào (Micro-Input Filtering)**: Sử dụng mẫu glob để lọc các tệp đầu vào, giảm thiểu việc mất hiệu lực cache không cần thiết (`MicroInputFilter`).

### 2. Đồ thị Biên dịch (`fish-graph`)
- **Đồ thị tác vụ topo**: Xây dựng đồ thị có hướng không chu trình (DAG) cho các tác vụ build và phát hiện chu trình phụ thuộc.
- **Truy vấn đại số đồ thị**: Đánh giá các biểu thức truy vấn phong cách Bazel (`deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`).
- **Mở rộng node động**: Sinh các đồ thị tác vụ con linh hoạt ngay trong quá trình thực thi (`DynamicGraphExpander`).

### 3. Thực thi & Hiện thực hóa Dữ liệu Nhanh (`fish-executor`)
- **Điều phối tiến trình**: Thực thi tác vụ bất đồng bộ không chặn (non-blocking async) với thời gian chờ (timeout) và bắt luồng dữ liệu xuất ra.
- **Sao chép dữ liệu nhanh (Fast Extents Cloning)**: Sử dụng cơ chế Copy-on-Write (CoW) extents và hardlink để xuất artifact mà không tốn chi phí I/O sao chép (`KernelCowCloner`).
- **Điều phối Linker thông minh**: Tự động phát hiện và tổng hợp cờ cho `mold`, `lld`, `lld-link`, và `msvc` (`LinkerDispatcher`).
- **Tệp phản hồi trình biên dịch (Compiler Response Files)**: Tạo tệp `@fish_args.rsp` khi độ dài tham số lệnh vượt quá giới hạn của hệ điều hành.

### 4. Bộ lập lịch & Kiểm soát Tài nguyên (`fish-scheduler`)
- **Cướp tác vụ song song (Parallel Work-Stealing)**: Lập lịch tác vụ không khóa (lock-free) trên tất cả các lõi phần cứng khả dụng.
- **Kiểm soát tài nguyên phần cứng (Kernel Resource Governor)**: Theo dõi áp lực bộ nhớ RAM và tự động điều tiết mức độ đồng thời để ngăn ngừa hiện tượng tràn RAM (OOM) (`KernelResourceGovernor`).
- **Đường ống biên dịch (Compiler Pipelining)**: Điều phối biên dịch nhiều giai đoạn để mở khóa các target phụ thuộc phía dưới ngay khi siêu dữ liệu (metadata) sẵn sàng (`PipelinedCompilationCoordinator`).
- **Nhóm GNU Jobserver**: Bể token toàn cục điều phối phân bổ luồng giữa các lần gọi trình biên dịch lồng nhau (`JobserverPool`).
- **Đua thực thi phân tán động (Dynamic Remote Racing)**: Cho phép chạy đua song song giữa máy cục bộ và worker đám mây (`DynamicRacingExecutor`).
- **Thực thi phân tán DTE**: Áp dụng thuật toán Longest Processing Time (LPT) bin-packing để cân bằng tải CI (`DteBinPacker`).
- **Theo dõi tệp thời gian thực (Filesystem Watcher)**: Daemon ngầm theo dõi các sự kiện thay đổi tệp và nung nóng sẵn đồ thị cache (`FsWatcherDaemon`).

### 5. Lưu trữ theo Nội dung (`fish-cache` & `fish-cas`)
- **Dấu vân tay (Fingerprinting)**: Băm nội dung bằng hàm băm Blake3 trên các tệp nguồn, biến môi trường và cờ biên dịch.
- **Kho lưu trữ CAS**: Lưu trữ artifact khử trùng lặp (deduplicated) với chuẩn nén Zstandard siêu tốc.
- **Cache phân tầng kết hợp**: Tích hợp tầng L1 bộ nhớ/ổ đĩa cục bộ và tầng L2 cache từ xa (S3/HTTP).

### 6. Giao diện Người dùng & Đo lường (`fish-cli`)
- **Giao diện dòng lệnh (CLI)**: Các lệnh tiện dụng cho build, test, check, graph, doctor, query, affected, và quản lý daemon.
- **Trực quan hóa đồ thị SVG tương tác**: Bảng vẽ đồ thị DAG thời gian thực trên nền web với tính năng phóng to/thu nhỏ, tìm kiếm, tiêu điểm node và làm nổi bật đường găng (critical path).
- **Bản địa hóa UI 5 ngôn ngữ**: Tích hợp sẵn bộ từ điển hỗ trợ Tiếng Anh, Tiếng Việt, Tiếng Trung giản thể, Tiếng Trung phồn thể, và Tiếng Nhật.
- **Daemon IPC cục bộ**: Daemon TCP qua cổng loopback `127.0.0.1:9527` giúp phân giải đồ thị ấm ngay lập tức.

---

## Các Backend Ngôn ngữ

Fish bao gồm 11 bộ điều hợp (adapter) ngôn ngữ chuyên dụng:

| Backend | Định danh | Manifest chính | Trình biên dịch / Công cụ mặc định |
| :--- | :--- | :--- | :--- |
| **Rust** | `rust` | `Cargo.toml` | `cargo`, `rustc` |
| **C / C++** | `cc` | `CMakeLists.txt`, `Makefile` | `cmake`, `clang`, `gcc`, `msvc` |
| **Go** | `go` | `go.mod` | `go build`, `go test` |
| **TypeScript / Node** | `ts` | `package.json` | `npm`, `pnpm`, `yarn`, `bun` |
| **Python** | `py` | `pyproject.toml`, `requirements.txt` | `python -m build`, `pytest`, `uv` |
| **Java / Kotlin** | `java` | `pom.xml`, `build.gradle` | `mvn`, `gradle` |
| **.NET** | `dotnet` | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` |
| **Swift** | `swift` | `Package.swift` | `swift build`, `swift test` |
| **Dart / Flutter** | `dart` | `pubspec.yaml` | `dart compile`, `flutter build` |
| **Zig** | `zig` | `build.zig` | `zig build` |
| **Docker** | `docker` | `Dockerfile` | `docker build` |

---

## Bảo mật & Xác minh

- **Ký số mật mã Artifact (`fish-signing`)**: Tạo và xác thực chữ ký số bằng thuật toán Ed25519.
- **Sinh SBOM**: Xuất danh mục thành phần phần mềm chuẩn SPDX và CycloneDX.
- **Quét lỗ hổng bảo mật (`fish-security`)**: Tự động quét các gói phụ thuộc với thang điểm CVSS và chặn build khi có lỗ hổng nghiêm trọng.
- **Quản lý bí mật (`fish-secrets`)**: Tích hợp HashiCorp Vault, AWS Secrets Manager và Kubernetes Secret với tính năng tự động ẩn thông tin nhạy cảm trên console.
