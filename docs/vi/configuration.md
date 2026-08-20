# Tài liệu Cấu hình (`fish.toml`)

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Fish được cấu hình thông qua tệp `fish.toml` đặt tại thư mục gốc của workspace hoặc từng package con.

## Cấu trúc mẫu

```toml
[workspace]
name = "my-monorepo"
members = [
    "packages/*",
    "apps/*"
]

[cache]
enabled = true
storage_dir = "~/.fish/cache"
max_size_gb = 50
compression = "zstd"

[scheduler]
max_jobs = 8
memory_limit_mb = 8192
strategy = "critical-path"

[ai]
enabled = true
endpoint = "stdio"
auto_suggest = true
```

## Các trường cấu hình

### `[workspace]`
- `name`: Tên định danh của workspace monorepo.
- `members`: Danh sách glob đường dẫn tới các gói thành viên.

### `[cache]`
- `enabled`: Bật hoặc tắt bộ nhớ đệm fingerprint.
- `storage_dir`: Đường dẫn thư mục lưu trữ Content-Addressable Storage (CAS).
- `max_size_gb`: Giới hạn dung lượng tối đa cho cache L1 trước khi thực hiện dọn dẹp hai pha.
- `compression`: Thuật toán nén artifact (`zstd`, `none`).

### `[scheduler]`
- `max_jobs`: Số lượng tác vụ song song tối đa (mặc định bằng số CPU cores).
- `strategy`: Chiến lược lập lịch (`critical-path`, `fifo`, `least-loaded`).
