use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

mod i18n;
mod install;
mod path_env;
mod toolchain_detect;

use i18n::{Language, Messages};

#[derive(Parser, Debug)]
#[command(name = "forge-setup", version, about = "Forge Build System Installer")]
struct CliArgs {
    #[arg(short, long, help = "Installation destination directory")]
    dir: Option<PathBuf>,

    #[arg(short, long, help = "Language code: en, vi, zh-cn, zh-tw, ja")]
    lang: Option<String>,

    #[arg(short = 'y', long, help = "Run silently in non-interactive mode")]
    silent: bool,

    #[arg(long, help = "Uninstall Forge from the system")]
    uninstall: bool,

    #[arg(long, help = "Only scan and report installed toolchains")]
    check_toolchains: bool,
}

fn prompt_line(prompt: &str) -> String {
    print!("{prompt} ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let _ = handle.read_line(&mut line);
    line.trim().to_string()
}

fn select_language_interactive() -> Language {
    println!("\nSelect your language / Chọn ngôn ngữ / 选择语言 / 言語を選択:");
    let languages = [
        Language::En,
        Language::Vi,
        Language::ZhCn,
        Language::ZhTw,
        Language::Ja,
    ];
    for (idx, lang) in languages.iter().enumerate() {
        println!("  {}. {}", idx + 1, lang.display_name());
    }

    let choice = prompt_line("Enter number [1-5]:");
    match choice.trim() {
        "2" => Language::Vi,
        "3" => Language::ZhCn,
        "4" => Language::ZhTw,
        "5" => Language::Ja,
        _ => Language::En,
    }
}

fn main() {
    let args = CliArgs::parse();

    let lang = match &args.lang {
        Some(code) => Language::from_str(code).unwrap_or(Language::En),
        None => {
            if args.silent || args.check_toolchains {
                Language::En
            } else {
                select_language_interactive()
            }
        }
    };

    let msg = Messages::get(lang);

    if args.check_toolchains {
        println!("\n{}", msg.toolchains_header);
        let statuses = toolchain_detect::scan_toolchains();
        for s in statuses {
            if s.detected {
                let ver = s.version.unwrap_or_default();
                println!(
                    "  [OK] {:<20} -> {} ({})",
                    s.language, msg.toolchain_found, ver
                );
            } else {
                println!("  [--] {:<20} -> {}", s.language, msg.toolchain_not_found);
            }
        }
        return;
    }

    let install_dir = args.dir.unwrap_or_else(install::get_default_install_dir);

    if args.uninstall {
        if !args.silent {
            let confirm = prompt_line(msg.uninstall_confirm);
            if !confirm.eq_ignore_ascii_case("y") && !confirm.eq_ignore_ascii_case("yes") {
                println!("Aborted.");
                return;
            }
        }

        match install::perform_uninstallation(&install_dir) {
            Ok(()) => {
                println!("\n{}", msg.uninstall_complete);
                println!("{}", msg.uninstall_path_removed);
            }
            Err(e) => {
                eprintln!("\nError during uninstallation: {e}");
            }
        }
        return;
    }

    println!("\n{}", msg.welcome_banner);
    println!("{} {}", msg.install_location, install_dir.display());

    let final_install_dir = if args.silent {
        install_dir
    } else {
        let custom = prompt_line(msg.enter_custom_path);
        if custom.is_empty() {
            install_dir
        } else {
            PathBuf::from(custom)
        }
    };

    println!("\n{}", msg.installing_binaries);
    println!("{}", msg.path_addition);
    let source_bin = install::find_source_forge_binary();

    match install::perform_installation(&final_install_dir, source_bin.as_deref()) {
        Ok(()) => {
            println!("{}", msg.installation_complete);
            println!("{}", msg.path_added);

            println!("\n{}", msg.toolchains_header);
            let statuses = toolchain_detect::scan_toolchains();
            for s in statuses {
                if s.detected {
                    let ver = s.version.unwrap_or_default();
                    println!("  [OK] {:<20} -> {}", s.language, ver);
                } else {
                    println!("  [--] {:<20} -> {}", s.language, msg.toolchain_not_found);
                }
            }

            println!("\n{}", msg.next_steps);
        }
        Err(e) => {
            eprintln!("\nInstallation failed: {e}");
        }
    }

    if !args.silent {
        prompt_line(&format!("\n{}", msg.press_enter_to_exit));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_parsing() {
        assert_eq!(Language::from_str("vi").unwrap(), Language::Vi);
        assert_eq!(Language::from_str("en").unwrap(), Language::En);
        assert_eq!(Language::from_str("zh-cn").unwrap(), Language::ZhCn);
        assert_eq!(Language::from_str("zh-tw").unwrap(), Language::ZhTw);
        assert_eq!(Language::from_str("ja").unwrap(), Language::Ja);
    }

    #[test]
    fn test_messages_retrieval() {
        for lang in [
            Language::En,
            Language::Vi,
            Language::ZhCn,
            Language::ZhTw,
            Language::Ja,
        ] {
            let msg = Messages::get(lang);
            assert!(!msg.welcome_banner.is_empty());
            assert!(!msg.installation_complete.is_empty());
        }
    }

    #[test]
    fn test_default_install_dir() {
        let dir = install::get_default_install_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
