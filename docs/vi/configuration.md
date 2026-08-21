# Hướng dẫn Cấu hình Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này hướng dẫn cách cấu hình các workspace Fish thông qua tệp `fish.toml`.

---

## Tổng quan Tệp Cấu hình

Fish đọc cấu hình dự án từ tệp `fish.toml` đặt tại thư mục gốc của workspace. Nếu không có `fish.toml`, Fish sẽ tự động áp dụng các giá trị mặc định tối ưu.

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.fish/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

---

## Các Mục Cấu hình Cấp cao nhất

### `[build]` —" Thiết lập Thực thi

| Khóa | Kiểu | Mặc định | Mô tả |
| :--- | :--- | :--- | :--- |
| `backend` | chuỗi | Auto | Backend toolchain chính (`rust`, `ts`, `go`, `cc`, `python`, `java`, `dotnet`, `docker`). |
| `jobs` | số nguyên | `num_cpus` | Số lượng tác vụ worker song song tối đa. |
| `no_cache` | boolean | `false` | Vô hiệu hóa việc tìm kiếm trên cache cục bộ và cache từ xa. |
| `sandbox` | boolean | `false` | Thực thi các tác vụ trong môi trường sandbox cách ly. |
| `semantic` | boolean | `false` | Bật tính năng phát hiện thay đổi ngữ nghĩa AST. |
| `critical_path` | boolean | `false` | Ưu tiên các điểm nghẽn trên đường găng (critical path) của đồ thị phụ thuộc. |
| `ram_limit` | số nguyên (1-100) | `85` | Điều tiết mức độ đồng thời khi bộ nhớ khả dụng giảm xuống dưới tỷ lệ này. |
| `timeout` | số nguyên | None | Thời gian chờ tối đa cho mỗi tác vụ tính bằng giây. |

---

### `[cache]` —" Thiết lập Lưu trữ Cục bộ

| Khóa | Kiểu | Mặc định | Mô tả |
| :--- | :--- | :--- | :--- |
| `dir` | chuỗi | `~/.fish/cache` | Đường dẫn đến thư mục Content-Addressable Storage (CAS) cục bộ. |
| `reflink` | boolean | `true` | Sử dụng Copy-on-Write (CoW) extents hoặc hardlinks để xuất artifact mà không tốn I/O copy. |

---

### `[remote]` —" Cache Phân tán & Thực thi Từ xa

| Khóa | Kiểu | Mặc định | Mô tả |
| :--- | :--- | :--- | :--- |
| `cache_url` | chuỗi | None | Địa chỉ máy chủ cache từ xa (HTTP/gRPC). |
| `token` | chuỗi | None | Mã xác thực Bearer token cho các thao tác từ xa. |
| `workers` | danh sách chuỗi | `[]` | Danh sách địa chỉ cluster worker từ xa (ví dụ: `["worker1:9000", "worker2:9000"]`). |
| `send_source` | boolean | `false` | Nén và truyền snapshot mã nguồn tới các worker không dùng chung filesystem. |

---

### `[daemon]` —" Dịch vụ IPC Chạy ngầm

| Khóa | Kiểu | Mặc định | Mô tả |
| :--- | :--- | :--- | :--- |
| `port` | số nguyên | `9527` | Cổng loopback TCP dành cho Fish background daemon. |

---

### `[pipelines.<task>]` —" Cấu trúc Đường ống Tác vụ

Cấu hình các phụ thuộc và ranh giới caching giữa các tác vụ trong toàn bộ package:

```toml
[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]

[pipelines.lint]
inputs = ["src/**/*.rs"]
```
