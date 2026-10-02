// Exe konsol alt sistemiyle derlenir: CLI alt komutlarının (`hatch`, `list` ...) çıktısı ve çıkış
// kodu her kabukta güvenilir gelsin. PowerShell 5.1 "windows" alt sistemli bir exe'yi beklemiyor,
// `$LASTEXITCODE` boş kalıyordu (kod incelemesi, bulgu 5). Pet penceresi konsol açmaz: `toggle`
// onu CREATE_NO_WINDOW ile başlatır; Gezgin'den çift tıklanınca açılan konsol hemen bırakılır.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `bop hook [olay]`: pencere açmadan durum dosyasını yaz ve çık.
    if args.get(1).map(String::as_str) == Some("hook") {
        bop_lib::hook_main(args.get(2).map(String::as_str));
        return;
    }
    // `bop list | use <id> | install <klasör> | toggle | stop | hatch ...`
    if let Some(code) = bop_lib::cli_main(&args[1..]) {
        std::process::exit(code);
    }
    console::release_own();
    bop_lib::run()
}

#[cfg(windows)]
mod console {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetConsoleProcessList(list: *mut u32, count: u32) -> u32;
        fn FreeConsole() -> i32;
    }

    /// Konsolu yalnız bu süreç kullanıyorsa (Gezgin'den açıldı) bırakır; pencere kapanır.
    /// Bir terminalden açıldıysa terminalin konsoluna dokunulmaz.
    pub fn release_own() {
        let mut ids = [0u32; 2];
        // SAFETY: dizi ve boyutu doğru verilir; başarısızlıkta (0) hiçbir şey yapılmaz.
        unsafe {
            if GetConsoleProcessList(ids.as_mut_ptr(), ids.len() as u32) == 1 {
                FreeConsole();
            }
        }
    }
}

#[cfg(not(windows))]
mod console {
    pub fn release_own() {}
}
