use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    En,
    Vi,
    ZhCn,
    ZhTw,
    Ja,
}

#[allow(dead_code)]
impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Vi => "Tiếng Việt",
            Self::ZhCn => "简体中文 (Simplified Chinese)",
            Self::ZhTw => "繁體中文 (Traditional Chinese)",
            Self::Ja => "日本語 (Japanese)",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Vi => "vi",
            Self::ZhCn => "zh-cn",
            Self::ZhTw => "zh-tw",
            Self::Ja => "ja",
        }
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "en" | "english" => Ok(Self::En),
            "vi" | "vietnamese" | "tieng-viet" => Ok(Self::Vi),
            "zh" | "zh-cn" | "zh-hans" | "chinese" => Ok(Self::ZhCn),
            "zh-tw" | "zh-hant" | "traditional-chinese" => Ok(Self::ZhTw),
            "ja" | "japanese" | "nihongo" => Ok(Self::Ja),
            _ => Err(format!("Unsupported language code: {s}")),
        }
    }
}

#[allow(dead_code)]
pub struct Messages {
    pub welcome_banner: &'static str,
    pub select_language: &'static str,
    pub install_location: &'static str,
    pub enter_custom_path: &'static str,
    pub path_addition: &'static str,
    pub path_added: &'static str,
    pub path_already_present: &'static str,
    pub installing_binaries: &'static str,
    pub installation_complete: &'static str,
    pub toolchains_header: &'static str,
    pub toolchain_found: &'static str,
    pub toolchain_not_found: &'static str,
    pub uninstall_confirm: &'static str,
    pub uninstall_complete: &'static str,
    pub uninstall_path_removed: &'static str,
    pub next_steps: &'static str,
    pub press_enter_to_exit: &'static str,
}

impl Messages {
    pub fn get(lang: Language) -> Self {
        match lang {
            Language::En => Self {
                welcome_banner: "=== Welcome to Fish Build System Installer ===",
                select_language: "Please select your language:",
                install_location: "Installation directory:",
                enter_custom_path: "Enter custom installation directory (or press Enter for default):",
                path_addition: "Adding Fish to User PATH environment variable...",
                path_added: "Successfully added Fish to PATH.",
                path_already_present: "Fish is already in your PATH.",
                installing_binaries: "Copying Fish executable files...",
                installation_complete: "Installation finished successfully!",
                toolchains_header: "Detecting Installed Language Toolchains:",
                toolchain_found: "Detected",
                toolchain_not_found: "Not found",
                uninstall_confirm: "Are you sure you want to uninstall Fish? (y/N):",
                uninstall_complete: "Fish has been successfully uninstalled.",
                uninstall_path_removed: "Removed Fish from User PATH environment variable.",
                next_steps: "Open a new terminal and run 'fish --help' to start building!",
                press_enter_to_exit: "Press Enter to exit...",
            },
            Language::Vi => Self {
                welcome_banner: "=== Chào mừng đến với Trình Cài Đặt Fish Build System ===",
                select_language: "Vui lòng chọn ngôn ngữ của bạn:",
                install_location: "Thư mục cài đặt:",
                enter_custom_path: "Nhập đường dẫn cài đặt tùy chỉnh (hoặc nhấn Enter để dùng mặc định):",
                path_addition: "Đang thêm Fish vào biến môi trường PATH của người dùng...",
                path_added: "Đã thêm Fish vào PATH thành công.",
                path_already_present: "Fish đã có sẵn trong PATH.",
                installing_binaries: "Đang sao chép các tệp thực thi Fish...",
                installation_complete: "Cài đặt đã hoàn tất thành công!",
                toolchains_header: "Kiểm tra các Toolchain ngôn ngữ đã cài đặt trên máy:",
                toolchain_found: "Đã phát hiện",
                toolchain_not_found: "Chưa cài đặt",
                uninstall_confirm: "Bạn có chắc chắn muốn gỡ cài đặt Fish? (y/N):",
                uninstall_complete: "Fish đã được gỡ cài đặt hoàn toàn.",
                uninstall_path_removed: "Đã xóa Fish khỏi biến môi trường PATH của người dùng.",
                next_steps: "Hãy mở cửa sổ Terminal mới và gõ lệnh 'fish --help' để bắt đầu!",
                press_enter_to_exit: "Nhấn Enter để thoát...",
            },
            Language::ZhCn => Self {
                welcome_banner: "=== 欢迎使用 Fish 构建系统安装程序 ===",
                select_language: "请选择您的语言：",
                install_location: "安装目录：",
                enter_custom_path: "输入自定义安装路径（直接按回车使用默认路径）：",
                path_addition: "正在将 Fish 添加到用户 PATH 环境变量...",
                path_added: "已成功将 Fish 添加到 PATH。",
                path_already_present: "Fish 已存在于您的 PATH 中。",
                installing_binaries: "正在复制 Fish 可执行文件...",
                installation_complete: "安装已成功完成！",
                toolchains_header: "正在检测已安装的语言工具链：",
                toolchain_found: "已检测到",
                toolchain_not_found: "未找到",
                uninstall_confirm: "您确定要卸载 Fish 吗？(y/N):",
                uninstall_complete: "Fish 已成功卸载。",
                uninstall_path_removed: "已从用户 PATH 环境变量中移除 Fish。",
                next_steps: "打开新的终端并运行 'fish --help' 即可开始！",
                press_enter_to_exit: "按回车键退出...",
            },
            Language::ZhTw => Self {
                welcome_banner: "=== 歡迎使用 Fish 構建系統安裝程式 ===",
                select_language: "請選擇您的語言：",
                install_location: "安裝目錄：",
                enter_custom_path: "輸入自訂安裝路徑（直接按 Enter 使用預設路徑）：",
                path_addition: "正在將 Fish 新增至使用者 PATH 環境變數...",
                path_added: "已成功將 Fish 新增至 PATH。",
                path_already_present: "Fish 已存在於您的 PATH 中。",
                installing_binaries: "正在複製 Fish 執行檔...",
                installation_complete: "安裝已成功完成！",
                toolchains_header: "正在檢測已安裝的語言工具鏈：",
                toolchain_found: "已檢測到",
                toolchain_not_found: "未找到",
                uninstall_confirm: "您確定要解除安裝 Fish 嗎？(y/N):",
                uninstall_complete: "Fish 已成功解除安裝。",
                uninstall_path_removed: "已從使用者 PATH 環境變數中移除 Fish。",
                next_steps: "開啟新的終端機並執行 'fish --help' 即可開始！",
                press_enter_to_exit: "按 Enter 鍵結束...",
            },
            Language::Ja => Self {
                welcome_banner: "=== Fish ビルドシステム インストーラーへようこそ ===",
                select_language: "言語を選択してください：",
                install_location: "インストール先ディレクトリ：",
                enter_custom_path: "カスタムインストール先を入力してください（既定値を使用する場合はEnter）：",
                path_addition: "Fish をユーザー PATH 環境変数に追加しています...",
                path_added: "Fish が PATH に正常に追加されました。",
                path_already_present: "Fish はすでに PATH に設定されています。",
                installing_binaries: "Fish 実行可能ファイルを配置しています...",
                installation_complete: "インストールが正常に完了しました！",
                toolchains_header: "インストール済みの言語ツールチェーンを検出しています：",
                toolchain_found: "検出済み",
                toolchain_not_found: "未検出",
                uninstall_confirm: "Fish をアンインストールしてもよろしいですか？ (y/N):",
                uninstall_complete: "Fish のアンインストールが完了しました。",
                uninstall_path_removed: "ユーザー PATH 環境変数から Fish を削除しました。",
                next_steps: "新しいターミナルを開き、'fish --help' を実行して開始してください！",
                press_enter_to_exit: "Enter キーを押して終了します...",
            },
        }
    }
}
