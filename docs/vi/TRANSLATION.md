# Hướng dẫn Dịch thuật Tài liệu

Chúng tôi rất hoan nghênh các đóng góp từ cộng đồng để dịch tài liệu Fish sang **bất kỳ ngôn ngữ nào**. Tài liệu này mô tả quy trình làm việc, chính sách công khai công cụ dịch, quy định linh hoạt cho người đóng góp và các kinh nghiệm thực tiễn.

---

## Phạm vi Áp dụng

Dịch thuật chỉ áp dụng cho **tài liệu** (các tệp markdown trong thư mục `docs/` và các hướng dẫn của dự án).

- **Mã nguồn, bài kiểm thử (tests), tên định danh (identifiers) và thông điệp commit** bắt buộc phải giữ **100% bằng Tiếng Anh**.
- **Các lệnh CLI, tên cờ tham số, đoạn mã nguồn và khóa cấu hình** bên trong tài liệu không được dịch.

---

## Quy định Linh hoạt cho Người Đóng góp Tài liệu

Để việc đóng góp trở nên thuận tiện và dễ dàng nhất:

1. **Hoan nghênh các bản dịch từng phần / Tăng dần (Incremental)**:
   - Bạn không cần phải dịch toàn bộ tài liệu trong một lần.
   - Dịch từng phần nhỏ (như mục *Cài đặt* hoặc *Bắt đầu nhanh*) đều được chào đón.
   - Đối với các phần chưa dịch xong, bạn có thể giữ nguyên văn bản tiếng Anh gốc hoặc để lại chú thích như `<!-- TODO: translate this section -->`.

2. **Không yêu cầu cài đặt Rust trên máy**:
   - Bạn không cần phải clone kho lưu trữ hoặc cài Rust để đóng góp tài liệu.
   - Bạn có thể chỉnh sửa tệp markdown trực tiếp trên **Giao diện Web của GitHub** bằng cách nhấn vào biểu tượng cây bút chì.

3. **Quy trình duyệt nhanh (Fast-Track Review)**:
   - Các Pull Request sửa lỗi chính tả, cải thiện định dạng, cập nhật liên kết và trau chuốt bản dịch sẽ được ưu tiên xem xét và merge nhanh chóng.

---

## Các Ngôn ngữ Được Hỗ trợ

Bản dịch được chấp nhận cho **mọi ngôn ngữ**. Fish duy trì 5 ngôn ngữ cốt lõi ưu tiên cùng với các đóng góp mở cho mọi ngôn ngữ khác:

| Ngôn ngữ | Mã ngôn ngữ | Trạng thái | Vai trò |
| :--- | :--- | :--- | :--- |
| **Tiếng Anh** (English) | `en` | Đang hoạt động | Nguồn chuẩn xác định |
| **Tiếng Trung Giản thể** (简体中文) | `zh-CN` | Mở | Ngôn ngữ cộng đồng cốt lõi |
| **Tiếng Trung Phồn thể** (繁體中文) | `zh-TW` | Mở | Ngôn ngữ cộng đồng cốt lõi |
| **Tiếng Nhật** (日本語) | `ja` | Mở | Ngôn ngữ cộng đồng cốt lõi |
| **Tiếng Việt** | `vi` | Mở | Ngôn ngữ cộng đồng cốt lõi |
| **Tất cả ngôn ngữ khác** (Tây Ban Nha, Pháp, Đức, Hàn, v.v.) | `*` | Mở | Ngôn ngữ cộng đồng |

---

## Chính sách Công cụ & Công khai Dịch máy Bắt buộc

### 1. Cho phép Sử dụng Công cụ Tự động & AI
Người đóng góp có thể sử dụng các trợ lý AI (ChatGPT, Claude, Gemini) và các công cụ dịch máy (DeepL, Google Translate) để dịch nháp hoặc đẩy nhanh tiến độ.

### 2. Bắt buộc Công khai việc Dịch máy
Tính minh bạch là bắt buộc đối với mọi bản dịch. Khi gửi Pull Request, bạn **phải công khai** trong phần mô tả PR xem có sử dụng công cụ dịch tự động hay không.

Vui lòng chọn phân cấp phù hợp trong PR của bạn:
- **Cấp 1 (Tier 1)** - Bản dịch Thủ công Bản ngữ: Do người nói tiếng mẹ đẻ hoặc thông thạo dịch 100% thủ công.
- **Cấp 2 (Tier 2)** - AI / Dịch máy có Người bản ngữ Duyệt: Bản nháp ban đầu tạo bởi AI/máy dịch, đã được người thông thạo kiểm tra và chỉnh sửa kỹ lưỡng.
- **Cấp 3 (Tier 3)** - Bản dịch Nháp AI / Tự động (Cần duyệt): Do công cụ AI/máy dịch tạo ra, chờ cộng đồng người bản ngữ kiểm tra chuyên sâu.

### 3. Ưu tiên Người Bản xứ
Các bản dịch được rà soát hoặc biên soạn bởi người thông thạo tiếng bản xứ sẽ được ưu tiên merge để đảm bảo tính tự nhiên và độ chính xác kỹ thuật cao nhất.

---

## Cấu trúc Thư mục

Bản dịch được sắp xếp trong các thư mục con theo mã ngôn ngữ chuẩn ISO dưới `docs/`:

```text
docs/
├── getting-started.md       # Tiếng Anh (Nguồn gốc)
├── architecture.md          # Tiếng Anh (Nguồn gốc)
├── vi/                      # Tiếng Việt
│   ├── getting-started.md
│   └── architecture.md
├── zh-CN/                   # Tiếng Trung Giản thể
│   ├── getting-started.md
│   └── architecture.md
├── zh-TW/                   # Tiếng Trung Phồn thể
│   ├── getting-started.md
│   └── architecture.md
├── ja/                      # Tiếng Nhật
│   ├── getting-started.md
│   └── architecture.md
└── <lang-code>/             # Bất kỳ ngôn ngữ nào khác (vd: es, fr, de, ko)
    └── getting-started.md
```

---

## Cách Gửi Bản Dịch

### Lựa chọn A: Qua Giao diện Web GitHub (Dễ nhất)
1. Truy cập tệp cần dịch dưới thư mục `docs/` trên GitHub.
2. Nhấn vào biểu tượng **Chỉnh sửa tệp này** (cây bút chì).
3. Lưu các thay đổi vào một nhánh mới và mở Pull Request.

### Lựa chọn B: Qua Dòng lệnh Git
1. Fork kho lưu trữ và clone về máy:
   ```bash
   git clone https://github.com/<your-username>/fish.git
   cd fish
   git checkout -b docs/translate-<lang>-<topic>
   ```
2. Thêm hoặc cập nhật các tệp markdown trong `docs/<lang-code>/`.
3. Commit thay đổi bằng thông điệp tiếng Anh (ví dụ: `docs: translate getting-started to Vietnamese`).
4. Push lên fork của bạn và mở Pull Request vào nhánh `dev`.
