# Câu hỏi Thường gặp & Xử lý Sự cố

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này bao gồm các câu hỏi thường gặp, hướng dẫn chuyển đổi dự án và các bước chẩn đoán xử lý sự cố trong Fish.

---

## Các Câu hỏi Thường gặp (FAQ)

### 1. Fish có thay thế Cargo, npm hay go build không?
Không. Fish là hệ thống **điều phối build (orchestrator)**, không phải là trình biên dịch thay thế. Nó kết nối các chuỗi công cụ hiện có của bạn (Cargo, rustc, Node.js, Go, GCC/Clang, dotnet), phân tích đồ thị phụ thuộc hợp nhất và tăng tốc quá trình build thông qua bộ nhớ đệm hermetic, lập lịch song song và thực thi phân tán.

### 2. Làm thế nào để chuyển đổi monorepo hiện có sang Fish?
Fish tự động phát hiện các dự án từ tệp manifest (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`).
1. Điều hướng tới thư mục gốc dự án của bạn.
2. Chạy `fish build` để Fish tự động khám phá toàn bộ workspace.
3. (Tùy chọn) Tạo tệp `fish.toml` tại thư mục gốc để tùy biến phụ thuộc pipeline và đường dẫn cache.

### 3. Cơ chế hoạt động của CAS Cache trong Fish như thế nào?
Fish tính toán mã băm Blake3 trên các tệp đầu vào, phiên bản toolchain và biến môi trường. Khi một tác vụ hoàn thành, các artifact sinh ra sẽ được nén bằng thuật toán Zstandard và lưu trong kho lưu trữ Content-Addressable Storage (CAS) tại `~/.fish/cache`. Nếu đầu vào không thay đổi, Fish sẽ xuất trực tiếp artifact bằng Copy-on-Write extents hoặc hardlink mà không cần gọi lại trình biên dịch.

---

## Các Tình huống Xử lý Sự cố

### Vấn đề: Target bị build lại ngoài ý muốn
**Giải pháp:**
Sử dụng cờ `--explain` để kiểm tra lý do target bị đánh dấu là dirty:
```bash
fish build --explain
```
Các nguyên nhân phổ biến bao gồm:
- Một tệp mã nguồn vừa được chỉnh sửa (touch).
- Mã băm đầu ra của một phụ thuộc cấp trên bị thay đổi.
- Sự khác biệt về biến môi trường làm vô hiệu hóa cache.

---

### Vấn đề: Sử dụng quá nhiều bộ nhớ RAM khi build song song
**Giải pháp:**
Khi biên dịch đồng thời nhiều crate lớn hoặc module C++, áp lực bộ nhớ có thể dẫn đến hiện tượng swap ổ đĩa. Hãy sử dụng cờ `--ram-limit` hoặc cấu hình `ram_limit` trong `fish.toml`:
```bash
fish build --ram-limit 80
```
Bộ điều tiết tài nguyên của Fish sẽ tự động giảm mức độ đồng thời mỗi khi mức tiêu thụ RAM vượt quá ngưỡng quy định.

---

### Vấn đề: Xung đột cổng Daemon chạy ngầm (`9527`)
**Giải pháp:**
Nếu cổng `9527` đang bị chiếm bởi một tiến trình khác, hãy chỉ định cổng tùy chỉnh:
```bash
fish daemon start --port 9588
```
Hoặc thiết lập biến môi trường:
```bash
export FISH_DAEMON_PORT=9588
```

---

### Vấn đề: Lỗi khóa tệp trên Windows (`os error 5: Access is denied`)
**Giải pháp:**
Trên Windows, việc chạy một file nhị phân trực tiếp từ thư mục `target/debug` sẽ khóa file thực thi trên ổ đĩa. Hãy cài đặt Fish toàn cục vào `%USERPROFILE%\.cargo\bin`:
```bash
cargo install --path crates/fish-cli --force
```
Sau đó gọi lệnh `fish` trực tiếp từ bất kỳ thư mục nào.
