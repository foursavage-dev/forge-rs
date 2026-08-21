# Kiến Trúc 36 Crates (`crates/`)

Fish được cấu trúc thành 36 crates Rust mô-đun hóa cao theo từng tầng kiến trúc phân lớp rõ ràng.

## Phân Tầng Kiến Trúc
1. **Tầng Nền Tảng (Foundation)**:
   - `fish-core`: Khám phá dự án, mô hình manifest, cấu hình `fish.toml`.
   - `fish-graph`: Đồ thị DAG, sắp xếp tô-pô phi khóa, đại số truy vấn graph.
   - `fish-executor`: Quản lý child process OS, response files `@args.rsp`, chuỗi middleware.
2. **Tầng Lưu Trữ & Cache (Storage)**:
   - `fish-cas`: Kho lưu trữ Content-Addressable Storage nén ZSTD và FastCDC chunking.
   - `fish-cache`: Bộ đệm fingerprint hai pha dọn dẹp GC.
   - `fish-remote-cache`: Giao thức gRPC REAPI v2 và bộ đệm TCP streaming.
3. **Tầng Lập Lịch & Thực Thi (Scheduling)**:
   - `fish-scheduler`: Bộ lập lịch Critical-Path Lookahead, Chase-Lev work-stealing và GNU Jobserver tokens.
   - `fish-worker`: Cụm thực thi worker từ xa và daemon IPC.
   - `fish-sandbox`: Truy vết eBPF cấp nhân Linux và sandbox WASM.
4. **Tầng 11 Backend Ngôn Ngữ**:
   - `fish-backend-rust`, `fish-backend-cc`, `fish-backend-go`, `fish-backend-ts`, `fish-backend-py`, `fish-backend-docker`, `fish-backend-java`, `fish-backend-dotnet`, `fish-backend-swift`, `fish-backend-dart`, `fish-backend-zig`.
5. **Tầng Bảo Mật & Tiện Ích**:
   - `fish-security`, `fish-signing`, `fish-secrets`, `fish-flaky-detection`, `fish-notifications`, `fish-analytics`, `fish-templates`, `fish-docker-builder`, `fish-incremental`, `fish-multiplatform`, `fish-installer`.
6. **Tầng Ứng Dụng CLI**:
   - `fish-cli`: Giao diện dòng lệnh hợp nhất, bảng điều khiển TUI và máy chủ `fish lsp`.
