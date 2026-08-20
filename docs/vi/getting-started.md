# Bắt đầu với Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này sẽ hướng dẫn bạn các bước làm quen và bắt đầu sử dụng Fish — hệ thống điều phối biên dịch tốc độ cao, ưu tiên bộ nhớ đệm (cache-first).

## Cài đặt

### Cài đặt 1 dòng lệnh (Khuyên dùng)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### Cài đặt từ mã nguồn

```bash
# Clone kho lưu trữ
git clone https://github.com/requla11/fish.git
cd fish

# Biên dịch và cài đặt
cargo install --path crates/fish-cli
```

### Cài đặt qua Cargo

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## Bắt đầu nhanh

### Biên dịch dự án Rust

```bash
cd your-rust-project
fish build
```

### Biên dịch Monorepo đa ngôn ngữ

```bash
# Clone monorepo mẫu
git clone https://github.com/requla11/fish.git
cd fish/examples/polyglot-demo

# Biên dịch tất cả dịch vụ
fish build

# Xem đồ thị phụ thuộc DAG
fish graph

# Chạy kiểm thử
fish test
```

## Các lệnh cơ bản

### Lệnh biên dịch (Build)

```bash
# Biên dịch toàn bộ workspace
fish build

# Biên dịch một package cụ thể
fish build -p my-package

# Biên dịch với 8 luồng song song
fish build -j 8

# Biên dịch bỏ qua cache
fish build --no-cache

# Biên dịch trong sandbox cách ly
fish build --sandbox

# Giải thích chi tiết lý do rebuild
fish build --explain

# Quy trình tối ưu hóa dựa trên profile (PGO)
fish build --pgo-generate
# ... chạy benchmark / khối lượng công việc ...
fish build --pgo-use
```

### Lệnh đồ thị & Truy vấn (Graph & Query)

```bash
# Truy vấn các phụ thuộc bắc cầu (phong cách Bazel)
fish query "deps(//fish-cli)"

# Truy vấn các phụ thuộc ngược
fish query "rdeps(//fish-graph)"

# Tìm tất cả đường đi giữa hai module
fish query "allpaths(//fish-cli, //fish-core)"

# Lọc phụ thuộc theo regex
fish query "filter('backend', deps(//fish-cli))"

# Trực quan hóa đồ thị
fish graph --format tree
fish graph --format dot
```

### Lệnh Build Daemon

```bash
# Khởi chạy daemon chạy ngầm để tăng tốc độ warm build dưới mili-giây
fish daemon start

# Kiểm tra trạng thái daemon
fish daemon status

# Dừng daemon
fish daemon stop
```

### Lệnh kiểm thử (Test)

```bash
# Chạy tất cả bài test
fish test

# Test một package cụ thể
fish test -p my-package

# Chạy test không dùng cache
fish test --no-cache
```

### Lệnh quản lý Cache

```bash
# Xem thống kê dung lượng cache
fish cache stats

# Dọn dẹp cache cũ
fish cache prune

# Khởi chạy máy chủ cache từ xa
fish cache-server --listen 0.0.0.0:8080
```

### Lệnh biên dịch phân tán (Distributed Build)

```bash
# Khởi chạy một worker
fish worker --listen 0.0.0.0:9000

# Biên dịch sử dụng cụm worker phân tán
fish build --workers worker1:9000,worker2:9000
```

### Lệnh tạo cấu hình CI/CD

```bash
# Tạo workflow cho GitHub Actions
fish ci init --platform github

# Tạo pipeline cho GitLab CI
fish ci init --platform gitlab

# Tạo cấu hình CircleCI
fish ci init --platform circleci

# Tạo cấu hình Bitbucket Pipelines
fish ci init --platform bitbucket

# Tạo cấu hình cho tất cả nền tảng
fish ci init --platform all
```

### Lệnh Plugin

```bash
# Liệt kê các plugin khả dụng
fish plugin list

# Thực thi một lệnh của plugin
fish plugin execute my-plugin build

# Cài đặt một plugin
fish plugin install ./my-plugin
```

## Cấu hình

### Cấu hình Workspace (`fish.toml`)

Fish tự động phát hiện loại dự án dựa trên các tệp manifest. Để tùy chỉnh hành vi thực thi, caching và pipeline, hãy tạo tệp `fish.toml` tại thư mục gốc dự án:

```toml
[build]
backend = "auto"
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
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

Xem thêm tại [Tài liệu cấu hình](configuration.md) để biết đầy đủ tùy chọn.

---

## Bảng điều khiển Web Dashboard & Đo lường Tương tác

Fish tích hợp sẵn bảng điều khiển web đo lường thời gian thực và trực quan hóa DAG với hỗ trợ 5 ngôn ngữ (Tiếng Anh, Tiếng Việt, Tiếng Trung giản thể, Tiếng Trung phồn thể, Tiếng Nhật):

```bash
# Khởi chạy giao diện web tại cổng 3000 và tự động mở trình duyệt
fish ui --port 3000 --open

# Kiểm tra dữ liệu đồ thị dạng JSON
curl http://localhost:3000/api/graph

# Kiểm tra thống kê phần cứng và CAS
curl http://localhost:3000/api/stats
```

---

## Xử lý sự cố thường gặp

### Build thất bại

Nếu quá trình build thất bại:

1. Kiểm tra thông báo lỗi hoặc chạy `fish build --explain` để chẩn đoán nguyên nhân rebuild.
2. Chạy với log chi tiết: `RUST_LOG=debug fish build`
3. Kiểm tra môi trường toolchain: `fish doctor`
4. Thử xóa cache: `fish cache prune`

### Sự cố về Cache

Nếu cache không hoạt động:

1. Kiểm tra thống kê cache: `fish cache stats`
2. Đảm bảo thư mục cache có quyền ghi: `~/.fish/cache`
3. Xóa và xây dựng lại cache: `fish cache prune && fish build`

### Sự cố kết nối Worker

Nếu worker không thể kết nối:

1. Kiểm tra kết nối mạng giữa các máy
2. Đảm bảo worker đang chạy: `fish worker --listen 0.0.0.0:9000`
3. Kiểm tra cấu hình tường lửa và mã token xác thực
4. Xem nhật ký log của worker

## Bước tiếp theo

- Đọc [Hướng dẫn Kiến trúc](architecture.md)
- Xem [Hướng dẫn Phát triển](../development.md)
- Khám phá [Danh mục Lệnh CLI](cli-reference.md)
- Khám phá [Tài liệu Backend ngôn ngữ](backends/)

## Trợ giúp & Hỗ trợ

- [Tài liệu chính thức](../getting-started.md)
- [Hỗ trợ kỹ thuật](../support.md)
- [Báo lỗi GitHub Issues](https://github.com/requla11/fish/issues)
- [Email liên hệ](mailto:foursavage@proton.me)
