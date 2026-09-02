# LichAI Desktop — Windows #01

Kiến trúc đã chốt:
- Web `https://lich.ai.vn/` là lõi chính.
- Desktop app là tray-first shell nhẹ, kiểu UniKey.
- Click trái tray -> cửa sổ Cài đặt nhỏ.
- Double-click tray -> mở LịchAI bằng trình duyệt mặc định.
- Click phải -> menu đa tầng.
- Tất cả nhóm chức năng vẫn hiện dù chưa đăng nhập.
- Login / Account / feature shortcuts mở Web, không nhúng WebView.
- Đóng cửa sổ Cài đặt chỉ ẩn về tray, không thoát app.
- Có tùy chọn Start with Windows.
- Source dùng Tauri 2 để sau này mở rộng macOS/Linux cùng codebase.

## Menu tray hiện tại
- Mở LịchAI
- Tài khoản
  - Đăng nhập / Đăng ký
  - Tài khoản & bảo mật
- Lịch
  - Hôm nay
  - Lịch tháng
  - Âm lịch
  - Ngày tốt
  - Giờ tốt
- Cá nhân
  - Ghi chú
  - Sự kiện
  - Sinh nhật
  - Kỷ niệm / Ngày giỗ
- Tiện ích
  - Thời tiết
  - Ngày giờ & hướng tốt
  - Học & Nghề
  - Các tiện ích khác
- Cài đặt
- Thoát

## Deep-link web
WIN #01 gửi các lối tắt dưới dạng `https://lich.ai.vn/?desktop=<route>`.
Nếu Web chưa đọc query này thì người dùng vẫn vào homepage bình thường.
Bước sau sẽ map các route vào đúng trang Web mà không viết lại thuật toán ở Desktop.

## Build GitHub
Repo root cần chứa thư mục này đúng cấu trúc. Chạy workflow:
`LichAI Desktop Windows`

Artifact:
`LichAI-Desktop-Windows`

## Chưa làm ở #01
- Đồng bộ trạng thái login thực vào tray label.
- Windows native notification / reminder device registration.
- Icon tray thay đổi thành số ngày.
- Mini calendar flyout riêng khi click trái.
- Code signing Windows installer.
- macOS/Linux jobs.

Các mục này chủ ý để sau khi WIN #01 build PASS, tránh nhét quá nhiều thứ trước khi baseline ổn định.
