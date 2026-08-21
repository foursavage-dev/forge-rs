# Lộ trình Phát triển Fish (Roadmap)

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này trình bày lộ trình phát triển chiến lược của dự án Fish, từ các cột mốc đã hoàn thành đến các mục tiêu ngắn hạn, trung hạn và tầm nhìn dài hạn.

---

## 🎯 Tầm nhìn chiến lược

Fish hướng tới trở thành hệ thống điều phối build đa ngôn ngữ (polyglot) hiệu năng cao nhất, an toàn nhất và thân thiện nhất với lập trình viên, vận hành trên **Kiến trúc Tri-Engine (Rust 75% + Python 15% + Go 10%)**.

---

## 🚀 Cột mốc Hiện tại (v0.2.x) — Đã Hoàn Thành

### Giai đoạn 1: Lõi Thực Thi & Đa Ngôn Ngữ
- [x] **Kiến trúc Tri-Engine**: Lõi tính toán Rust siêu tốc (75%), Trí tuệ nhân tạo Python (15%) và Mạng phân tán Go (10%).
- [x] **11 Backend Ngôn ngữ**: Rust, Go, TypeScript/Node.js, Python, C/C++, Docker, Java, .NET, Swift, Dart, Zig.
- [x] **Hợp đồng Protobuf dùng chung**: Các giao thức `build.proto`, `ai.proto`, `coordinator.proto`.
- [x] **Lưu trữ CAS Blake3 & Thuật toán Nén ZSTD**: Hệ thống Content-Addressable Storage hai pha dọn dẹp.
- [x] **Điều phối GNU Jobserver**: Phân phối token luồng toàn cục chống tràn bộ nhớ CPU/RAM.
- [x] **Tự động sinh CI/CD**: Hỗ trợ GitHub Actions, GitLab CI, CircleCI, Bitbucket Pipelines.
- [x] **Hệ thống Tài liệu 5 Ngôn ngữ**: Triển khai trực tiếp trên GitHub Pages (Anh, Việt, Trung Giản/Phồn thể, Nhật).

---

## ⚡ Mục tiêu Ngắn hạn (v0.3.x) — Trọng tâm: Trải nghiệm & Giao thức

### 1. Tích hợp IDE & Trình soạn thảo
- [x] **Extension VS Code**: Trực quan hóa đồ thị DAG tương tác, chạy tác vụ 1-click và chuẩn đoán lỗi trực tiếp.
- [x] **Plugin JetBrains**: Tích hợp cho CLion, IntelliJ IDEA và Rider với ToolWindow DAG và LSP.
- [x] **Cầu nối LSP (Language Server Protocol)**: Tự động hoàn thiện cú pháp `fish.toml` và kiểm tra lỗi thời gian thực.

### 2. Giao thức IPC & Dịch vụ Tốc độ cao
- [x] **Luồng IPC Daemon**: Giao tiếp JSON-RPC qua Unix domain socket / Windows named pipes với độ trễ dưới 1 mili-giây.
- [x] **Giao thức gRPC REAPI**: Chuẩn Remote Execution API v2 tương thích các hệ thống build phân tán.
- [x] **Theo dõi tệp bằng eBPF**: Ghi nhận chính xác tệp input/output và phát hiện phụ thuộc động ở cấp nhân Linux.

### 3. Công cụ Chuẩn đoán Thông minh
- [x] **Trợ lý Doctor AI tương tác**: Tự động phát hiện lỗi môi trường và gợi ý lệnh sửa tự động (`fish doctor --fix`).
- [x] **Nâng cấp giao diện TUI**: Biểu đồ CPU/RAM thời gian thực và chế độ xem tiến trình dạng Waterfall.

> **Cột mốc v0.3.x hoàn thành (21/08/2026):** Toàn bộ 8/8 hạng mục về Trải nghiệm Lập trình viên & Giao thức Phân tán đã hoàn tất 100% với kiểm thử xanh toàn diện.

---

## 🌟 Mục tiêu Trung hạn (v0.4.x - v0.5.x) — Trọng tâm: Hạ tầng Đám mây & AI

### 1. Hạ tầng Phân tán Chuẩn Cloud-Native
- [ ] **Kubernetes Operator (Go)**: Tự động co giãn (autoscaling) cụm worker pods theo tải thực tế.
- [ ] **Tối ưu hóa Spot Instance**: Di chuyển tác vụ an toàn khi nút đám mây bị thu hồi.
- [ ] **Đồng bộ Cache Đa Vùng (P2P CAS)**: Tái tạo và chia sẻ cache L2 phân tán theo địa lý.

### 2. Trí tuệ Nhân tạo & Dự đoán Tác vụ
- [x] **Mô hình AI Dự đoán Thời gian Build**: Dự báo thời gian hoàn thành dựa trên độ phức tạp AST và dữ liệu lịch sử.
- [x] **Cách ly Flaky Test Tự động**: Phát hiện và cô lập các bài kiểm thử không tất định.
- [x] **Pre-warming Thông minh**: Tự động dự đoán và biên dịch trước các gói có khả năng bị sửa đổi.

### 3. Đo lường Hiệu suất & Hợp tác Nhóm
- [ ] **Tích hợp OpenTelemetry**: Truy vết phân tán toàn diện trên từng bước build và nút mạng.
- [ ] **Bảng điều khiển Web Dashboard**: Đo lường tốc độ build của nhóm và tỷ lệ tiết kiệm thời gian.
- [ ] **Tính toán Chi phí Đám mây**: Báo cáo chi phí điện toán và lưu trữ tiết kiệm được.

### 4. Hệ sinh thái Plugin
- [ ] **WebAssembly Plugin Engine**: Mở rộng công cụ qua Wasm/WASI an toàn với Extism.
- [ ] **Chợ Plugin (Marketplace Registry)**: Hệ thống phân phối plugin có chữ ký số xác thực.

---

## 🏰 Tầm nhìn Dài hạn (v1.0+) — Trọng tâm: Doanh nghiệp & Zero-Trust

### 1. Bảo mật Cấp Doanh nghiệp & Zero-Trust
- [ ] **Cách ly Phần cứng MicroVM**: Chạy build trong máy ảo siêu nhẹ Firecracker / Cloud-Hypervisor.
- [ ] **Xác thực Doanh nghiệp (SSO / OIDC & RBAC)**: Quản lý quyền truy cập và lưu vết kiểm toán chi tiết.
- [ ] **Chứng thực Chuỗi cung ứng SLSA L3**: Tạo bản khai in-toto attestation và SBOM không thể giả mạo.

### 2. Biên dịch Phân tán Toàn cầu
- [ ] **Bộ nhớ đệm AST Cấp Hàm**: Tái sử dụng kết quả biên dịch ở cấp độ hàm xuyên suốt các ngôn ngữ.
- [ ] **Mạng Lưới Chia sẻ P2P Toàn cầu**: Phân phối artifacts dạng BitTorrent cho các trang trại máy chủ CI lớn.
- [ ] **Tác tử Tự động Tối ưu Liên tục**: AI liên tục tinh chỉnh cờ biên dịch để đạt tốc độ tối đa.

---

## 📅 Dự kiến Tiến độ

| Phiên bản | Trọng tâm chính | Dự kiến | Trạng thái |
| :--- | :--- | :--- | :--- |
| **v0.2.x** | Lõi Tri-Engine, 11 Backends, CAS, Tài liệu 5 Ngôn ngữ | Q3 2026 | ✅ Hoàn thành |
| **v0.3.x** | Plugin IDE, Giao thức IPC, eBPF Tracing, LSP | Hiện tại | ✅ Hoàn thành |
| **v0.4.x - v0.5.x** | K8s Operator, Dự đoán ML, OpenTelemetry, Wasm | Q1 - Q2 2027 | 🟡 Tiếp theo |
| **v1.0** | MicroVM Sandbox, Doanh nghiệp SSO, P2P Mesh, SLSA L3 | Q3 2027+ | ⚪ Tầm nhìn |
