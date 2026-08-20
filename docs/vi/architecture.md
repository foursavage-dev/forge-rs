# Kiến trúc hệ thống Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Fish là hệ thống điều phối build đa ngôn ngữ (polyglot build orchestrator), ứng dụng kiến trúc Tri-Engine hiện đại để đạt hiệu năng tối đa, khả năng phân tán mạnh mẽ và năng lực trí tuệ nhân tạo.

## Kiến trúc Tri-Engine (Rust + Python + Go)

```
┌─────────────────────────────────────────────────────────────┐
│                       Giao diện CLI                         │
│                    (crates/fish-cli)                        │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌──────────────────────────────┐ ┌────────────────────────────┐
│   Lõi Thực Thi Rust (75%)   │ │  Dịch vụ Mạng Go (10%)    │
│  - fish-core, fish-graph     │ │  - fish-coordinator       │
│  - fish-executor, scheduler  │ │  - fish-worker-gateway    │
│  - fish-cache, fish-cas      │ │  - fish-network, migrator │
└──────────────┬───────────────┘ └────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Dịch vụ AI Python (15%)                    │
│   - fish_ai_analyzer   - fish_optimizer                     │
│   - fish_analytics     - fish_recommender                   │
└─────────────────────────────────────────────────────────────┘
```

### 1. Lõi thực thi Rust (75%)
- **`fish-core`**: Quản lý workspace, phân tích manifest và lọc tệp chi tiết.
- **`fish-graph`**: Đồ thị có hướng không chu trình (DAG), sắp xếp topological và truy vấn đại số.
- **`fish-executor`**: Thực thi tiến trình, cô lập sandbox và điều phối middleware.
- **`fish-scheduler`**: Tận dụng GNU Jobserver, phân luồng song song thông minh.
- **`fish-cache` & `fish-cas`**: Fingerprint đa tầng Blake3 và lưu trữ artifact nén ZSTD.

### 2. Dịch vụ AI Python (15%)
- **`fish_ai_analyzer`**: Phân tích log lỗi build, bóc tách nguyên nhân và đề xuất cách sửa.
- **`fish_optimizer`**: Tối ưu đường găng (critical path) trên đồ thị và phân bổ bộ nhớ.
- **`fish_analytics`**: Đo lường hiệu suất build, tỷ lệ trúng cache và phát hiện điểm nghẽn.
- **`fish_recommender`**: Đề xuất các target cần build dựa trên git diff và phát hiện flaky test.

### 3. Dịch vụ Mạng Go (10%)
- **`fish-coordinator`**: Đăng ký nút worker, theo dõi heartbeat và điều phối tác vụ phân tán.
- **`fish-worker-gateway`**: Reverse proxy hiệu năng cao, cân bằng tải Least-Loaded.
- **`fish-network`**: Quản lý connection pool và mTLS bảo mật cao.
- **`fish-db-migrator`**: Quản lý phiên bản và migration cho cơ sở dữ liệu telemetry.
