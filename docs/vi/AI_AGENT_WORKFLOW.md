# Hướng dẫn Quy trình Làm việc cho AI Agent trong Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này cung cấp hướng dẫn chi tiết từng bước cho các AI coding agent khi làm việc trong hệ thống điều phối biên dịch Fish, đảm bảo giảm thiểu lỗi, vận hành trơn tru và đóng góp mã nguồn chất lượng cao.

## 🎯 Tổng quan

Quy trình làm việc toàn diện này được thiết kế để:
- Giảm thiểu tối đa việc phát sinh lỗi và bug.
- Đảm bảo dự án luôn vận hành trơn tru.
- Duy trì tính nhất quán và chất lượng mã nguồn cao.
- Tuân thủ các quy chuẩn kỹ thuật đặc thù của dự án.
- Tích hợp liền mạch với kiến trúc hiện có.

---

## 📖 Giai đoạn 1: Thu thập Ngữ cảnh Trước khi Làm việc

### Bước 1.1: Đọc các tài liệu thiết yếu (Bắt buộc)

**Thứ tự đọc ưu tiên:**
1. **README.md** - Tổng quan dự án, hướng dẫn bắt đầu nhanh, các lệnh cơ bản.
2. **Cargo.toml** - Cấu trúc workspace, các phụ thuộc, yêu cầu MSRV (Rust 1.88+).
3. **ARCHITECTURE.md** - Kiến trúc hệ thống và trách nhiệm của từng thành phần.
4. **DEVELOPMENT.md** - Quy trình thiết lập môi trường phát triển cục bộ.

### Bước 1.2: Đọc tài liệu chuyên biệt theo nhiệm vụ

| Loại nhiệm vụ | Tài liệu bổ sung cần đọc |
|---|---|
| Language backend | `crates/fish-backend-rust/` (làm mẫu), phần Backend trong `ARCHITECTURE.md` |
| Thay đổi Scheduler | Các tệp mã nguồn trong `crates/fish-scheduler/` |
| Cải tiến Cache | Các tệp mã nguồn trong `crates/fish-cache/` và `crates/fish-cas/` |
| Thay đổi CLI | Các tệp mã nguồn trong `crates/fish-cli/` |
| Tính năng Bảo mật | `crates/fish-security/` và `crates/fish-signing/` |
| Biên dịch phân tán | `crates/fish-worker/` và `crates/fish-remote-cache/` |

### Bước 1.3: Kiểm tra trạng thái hiện tại của dự án

```bash
git status
git branch
cargo check --workspace
```

**Các điểm kiểm tra quan trọng:**
- Đảm bảo bạn đang làm việc trên nhánh `dev` (không bao giờ commit trực tiếp lên `main`).
- Thư mục làm việc sạch sẽ trước khi chỉnh sửa.
- Toàn bộ bài test hiện tại phải vượt qua trước khi bắt đầu viết code mới.

---

## 🎯 Giai đoạn 2: Phân tích Nhiệm vụ & Lập Kế hoạch

### Bước 2.1: Hiểu rõ yêu cầu bài toán
- Vấn đề cụ thể cần giải quyết là gì?
- Những crate / module nào sẽ bị ảnh hưởng?
- Có mẫu thiết kế (pattern) nào có sẵn cần tuân thủ không?
- Tác dụng phụ tiềm ẩn là gì?

### Bước 2.2: Lập kế hoạch thay đổi
- Liệt kê danh sách tệp cần tạo mới hoặc sửa đổi.
- Thiết kế giao diện API tương thích ngược.
- Lên kế hoạch viết bài kiểm thử unit tests đi kèm.

---

## 💻 Giai đoạn 3: Thực hiện Mã hóa (Coding)

### Bước 3.1: Quy chuẩn Mã nguồn
- Tuân thủ Rust 2024 edition.
- Sử dụng `anyhow` cho xử lý lỗi ứng dụng, `thiserror` cho định nghĩa lỗi tùy chỉnh trong thư viện.
- Sử dụng `async/await` hợp lý với `tokio`.
- **Tuyệt đối không để lại comment thừa trong code theo yêu cầu dự án**.
- Toàn bộ tên hàm, biến, struct, commit message phải bằng **Tiếng Anh**.

### Bước 3.2: Viết Kiểm thử Song hành
- Mỗi tính năng mới hoặc bản sửa lỗi bắt buộc phải có unit test đi kèm.
- Kiểm tra các trường hợp biên và xử lý lỗi không mong muốn.

---

## 🔍 Giai đoạn 4: Xác minh & Kiểm thử Chất lượng

```bash
# 1. Kiểm tra format
cargo fmt --all -- --check

# 2. Kiểm tra cảnh báo linter
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Chạy toàn bộ test suite
cargo test --workspace
```

---

## 🚀 Giai đoạn 5: Commit & Đóng góp
- Đảm bảo commit trên nhánh `dev`.
- Viết thông điệp commit rõ ràng bằng Tiếng Anh theo chuẩn Conventional Commits (`feat: ...`, `fix: ...`, `docs: ...`).
