---
layout: home

hero:
  name: "Fish"
  text: "Hệ thống điều phối build đa ngôn ngữ tốc độ cao"
  tagline: "Hợp nhất quy trình build trên 11+ ngôn ngữ với lập lịch đồ thị DAG đại số, bộ nhớ đệm CAS xác định và thực thi phân tán."
  image:
    src: /logo.svg
    alt: Fish Logo
  actions:
    - theme: brand
      text: Bắt đầu ngay
      link: /vi/getting-started
    - theme: alt
      text: Kiến trúc hệ thống
      link: /vi/architecture
    - theme: alt
      text: Xem trên GitHub
      link: https://github.com/requla11/fish

features:
  - icon: ⚡
    title: Tối đa hóa hiệu năng & Racing
    details: Tích hợp GNU Jobserver, cơ chế racing song song giữa local và worker từ xa giúp tận dụng triệt để CPU và mạng.
  - icon: 🎯
    title: Hỗ trợ 11+ Ngôn ngữ
    details: Tự động nhận diện cấu hình cho Rust, Go, TypeScript, Python, C/C++, Docker, Java, .NET, Swift, Dart và Zig.
  - icon: 🔒
    title: Cache CAS & Fingerprint chuẩn xác
    details: Sử dụng hàm băm Blake3 đa tầng kết hợp nén ZSTD giúp tái sử dụng kết quả build tức thì chỉ trong vài mili-giây.
  - icon: 🌐
    title: Giao diện Web Dashboard trực quan
    details: Trực quan hóa đồ thị phụ thuộc DAG trong thời gian thực, đo lường dữ liệu build chi tiết đa ngôn ngữ.
  - icon: 🛡️
    title: Chữ ký số Ed25519 & Sandbox
    details: Tự động quét lỗ hổng bảo mật, tạo hóa đơn vật liệu phần mềm SBOM và môi trường cô lập hermetic.
  - icon: 🚀
    title: Bộ máy truy vấn DAG Đại số
    details: Tra cứu quan hệ phụ thuộc (deps, rdeps, path) trên toàn bộ monorepo bằng cú pháp biểu thức mạnh mẽ.
---
