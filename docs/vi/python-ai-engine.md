# Bộ Máy Trí Tuệ Nhân Tạo Python (`py/`)

Fish tích hợp bộ máy học máy dự đoán thông minh viết bằng Python 3.11+.

## Các tính năng AI
- **Dự đoán thời gian build (`py/src/fish_ai/predictor.py`)**: Ước tính thời lượng biên dịch dựa trên lịch sử và đặc trưng mã nguồn AST.
- **Cách ly Flaky Test (`py/src/fish_ai/flaky.py`)**: Tự động phát hiện và cách ly các bài kiểm thử không tất định.
- **Pre-warming Phỏng đoán (`py/src/fish_ai/speculative.py`)**: Dự đoán các gói bị ảnh hưởng dựa trên git diff để pre-compile trước.
- **Tự sửa lỗi AST (`py/src/fish_ai/remediation.py`)**: Gợi ý và tự động sửa các lỗi cú pháp và thiếu import.

## Chạy kiểm thử Python AI
```bash
cd py
python -m unittest discover tests
```
