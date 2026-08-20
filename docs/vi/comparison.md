# Bảng So Sánh Trực Diện: Fish và Các Hệ Thống Build Khác

Fish được thiết kế và xây dựng bằng Rust 2024 dành cho các monorepo đa ngôn ngữ hiện đại. Dưới đây là bảng so sánh trực tiếp, toàn diện với Bazel, Turborepo và Buck2:

| Tính năng / Khía cạnh | Fish | Turborepo | Bazel | Buck2 |
| :--- | :--- | :--- | :--- | :--- |
| **Ngôn ngữ phát triển** | Rust 2024 | Go / Rust | Java / C++ | Rust |
| **Hỗ trợ ngôn ngữ** | Đa ngôn ngữ (11+ toolchains) | Tập trung JS / TS | Đa ngôn ngữ (Starlark) | Đa ngôn ngữ (Starlark) |
| **Mô hình cấu hình** | `fish.toml` tự động nhận diện | `turbo.json` | `WORKSPACE` + `BUILD.bazel` | `BUCK` files |
| **Độ phức tạp thiết lập** | Cực thấp / Tự động | Thấp | Rất cao | Cao |
| **Bộ băm Fingerprint** | Blake3 (siêu nhanh) | SHA-256 | SHA-256 | Blake3 / SHA-256 |
| **Nén & CAS Cache** | Zstandard (Zstd) + CoW | Tar.gz / Gzip | Zstd / Custom | Zstd / Custom |
| **Xuất Artifact ra đĩa** | Reflink / Copy-on-Write (0ms) | File copy | Symlinks / Hardlinks | Reflink / CoW |
| **Phân mảnh khối** | FastCDC (16KB - 256KB) | Toàn bộ tệp nén | Toàn bộ tệp nén | Chunked CAS |
| **Tốc độ VFS** | Cây In-Memory RAM (<2ms) | Quét filesystem | Daemon Inotify/Watchman | Watchman / EdenFS |
| **Semantic Invalidation** | AST Interface Hash (ABI) | Băm toàn bộ file | Header-only compile | Header / rmeta compile |
| **Chuẩn đoán AI** | Tích hợp IPC & Explain | Không có | Không có | Không có |
| **Giao diện Dashboard** | Tích hợp sẵn Web GUI & TUI | Vercel Web App | Phụ thuộc bên thứ ba | Open-source console |

---

## Phân Tích Kiến Trúc Chuyên Sâu

### Fish và Turborepo
* **Phạm vi ngôn ngữ:** Turborepo phục vụ chủ yếu hệ sinh thái JavaScript/TypeScript. Fish coi tất cả 11+ chuỗi công cụ (Rust, Go, C++, Python, Docker, v.v.) là công dân hạng nhất.
* **Tốc độ I/O:** Turborepo sao chép file thủ công. Fish dùng Reflink CoW và phân mảnh khối FastCDC để triệt tiêu thời gian copy đĩa và băng thông mạng.

### Fish và Bazel
* **Trải nghiệm sử dụng:** Bazel yêu cầu viết file BUILD cho từng thư mục. Fish tự động nhận diện dự án thông qua lệnh `fish init`.
* **Tài nguyên tiêu thụ:** Bazel phụ thuộc Java JVM nặng nề. Fish là binary Rust duy nhất, siêu nhẹ và khởi động tức thì.
