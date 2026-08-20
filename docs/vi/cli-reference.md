# Danh mục Lệnh Fish CLI

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu tham khảo toàn diện về tất cả các lệnh và tùy chọn của giao diện dòng lệnh Fish.

---

## Tùy chọn Toàn cục (Global Options)

- `--experimental`: Bật các tính năng thử nghiệm.
- `-v, --verbose`: Bật đầu ra chẩn đoán chi tiết.
- `-j, --jobs <N>`: Số lượng luồng worker song song tối đa.
- `--no-cache`: Bỏ qua cả cache cục bộ và cache từ xa.
- `--cache-dir <PATH>`: Chỉ định đường dẫn thư mục cache cục bộ.
- `--explain`: In lý do chi tiết tại sao các target bị rebuild.
- `--pgo-generate`: Thêm công cụ đo lường vào file nhị phân cho Profile-Guided Optimization.
- `--pgo-use`: Biên dịch file nhị phân sử dụng dữ liệu PGO đã thu thập.

---

## Các Lệnh Chính

### `fish init`
Khởi tạo cấu hình Fish và quét workspace để tạo định nghĩa tác vụ đa ngôn ngữ (`fish.yaml`).

```bash
fish init [--force]
```

---

### `fish ui`
Khởi chạy Web Dashboard thời gian thực và trình trực quan hóa đồ thị SVG DAG với hỗ trợ 5 ngôn ngữ (Tiếng Anh, Tiếng Việt, Tiếng Trung giản thể, Tiếng Trung phồn thể, Tiếng Nhật).

```bash
fish ui [--port <PORT>] [--open]
```

---

### `fish build`
Thực thi các tác vụ build cho các package trong workspace.

```bash
fish build [OPTIONS]
```

**Các cờ phổ biến:**
- `-p, --package <NAME>`: Biên dịch một package cụ thể.
- `--explain`: Chẩn đoán nguyên nhân các package bị build lại.
- `--profile [FILE]`: Tạo tệp hồ sơ hiệu năng Chrome trace JSON.
- `--sandbox`: Chạy trong môi trường sandbox cách ly.
- `--ram-limit <PCT>`: Điều tiết mức độ đồng thời khi bộ nhớ vượt ngưỡng phần trăm cho phép.

---

### `fish check`
Kiểm tra kiểu dữ liệu và phân tích tĩnh mà không liên kết (link) toàn bộ artifact.

```bash
fish check [OPTIONS]
```

---

### `fish test`
Thực thi các bộ kiểm thử trên tất cả các package trong workspace.

```bash
fish test [OPTIONS]
```

---

### `fish run`
Biên dịch và chạy một target thực thi nhị phân cụ thể.

```bash
fish run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
```

---

### `fish query <EXPR>`
Đánh giá các truy vấn đại số trên đồ thị phụ thuộc của workspace.

```bash
fish query "<EXPRESSION>"
```

**Các hàm được hỗ trợ:**
- `deps(//pkg)`: Tất cả các phụ thuộc bắc cầu của `//pkg`.
- `rdeps(//pkg)`: Tất cả các phụ thuộc ngược bắc cầu của `//pkg`.
- `allpaths(//from, //to)`: Tất cả đường đi giữa `//from` và `//to`.
- `somepath(//from, //to)`: Đường đi ngắn nhất giữa `//from` và `//to`.
- `filter('pattern', expr)`: Lọc các package khớp theo từ khóa hoặc mẫu regex.

**Ví dụ:**
```bash
# Tìm tất cả những gì cần thiết để biên dịch fish-cli
fish query "deps(//fish-cli)"

# Tìm tất cả các crate bị ảnh hưởng khi fish-graph thay đổi
fish query "rdeps(//fish-graph)"

# Tìm chuỗi phụ thuộc ngắn nhất giữa app và util
fish query "somepath(//app, //util)"
```

---

### `fish daemon`
Quản lý build daemon chạy ngầm để tăng tốc độ phân giải đồ thị ấm.

```bash
# Khởi động daemon
fish daemon start [--port 9527]

# Kiểm tra trạng thái daemon
fish daemon status [--port 9527]

# Dừng daemon
fish daemon stop [--port 9527]
```

---

### `fish graph`
In hoặc xuất đồ thị phụ thuộc của dự án.

```bash
fish graph [--format <tree|dot|json>]
```

---

### `fish affected`
Xác định và chỉ thực thi các tác vụ trên các package bị thay đổi kể từ một mốc Git commit.

```bash
fish affected --since <GIT_REF> [--mode <build|check|test>]
```

---

### `fish cache`
Quản lý kho lưu trữ Content-Addressable Storage (CAS) và dấu vân tay cục bộ.

```bash
# Hiển thị dung lượng và số lượng đối tượng trong cache
fish cache stats

# Dọn dẹp dấu vân tay cũ và các artifact mồ côi
fish cache prune

# Kiểm tra kho lưu trữ CAS
fish cache cas stats
fish cache cas list
```

---

### `fish doctor`
Kiểm tra tính sẵn sàng của các toolchain, trình biên dịch, trình liên kết và các phụ thuộc hệ thống.

```bash
fish doctor [--fix] [--ai]
```

---

### `fish ci init` / `fish ci export`
Tự động sinh cấu hình CI workflow cho nhiều nền tảng khác nhau.

```bash
fish ci init --platform <github|gitlab|circleci|bitbucket|all>
```
