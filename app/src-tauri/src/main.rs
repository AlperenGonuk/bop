// The exe is built with the console subsystem so CLI subcommands (`hatch`, `list` ...) report
// their output and exit code reliably in every shell. PowerShell 5.1 does not wait for a
// "windows" subsystem exe, so `$LASTEXITCODE` stayed empty (code review, finding 5). The pet
// window opens no console: `toggle` starts it with CREATE_NO_WINDOW; when double-clicked in
// Explorer, the console that opens is released right away.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `bop hook [event]`: write the state file without opening a window, then exit.
    if args.get(1).map(String::as_str) == Some("hook") {
        bop_lib::hook_main(args.get(2).map(String::as_str));
        return;
    }
    // `bop list | use <id> | install <folder> | toggle | stop | hatch ...`
    if let Some(code) = bop_lib::cli_main(&args[1..]) {
        std::process::exit(code);
    }
    // Opened without arguments (double-clicked after downloading): put a copy in ~/.bop/bin,
    // where the plugin finds it, and run the pet from there so the download is not kept open.
    // Pets started by `toggle` get `--no-install`; debug builds skip this entirely.
    if args.len() == 1 && !cfg!(debug_assertions) {
        match bop_lib::install_self() {
            Ok(bop_lib::Install::Here) => {}
            Ok(
                bop_lib::Install::Current(app)
                | bop_lib::Install::Updated(app)
                | bop_lib::Install::NewerKept(app),
            ) => {
                match bop_lib::spawn_pet(&app) {
                    Ok(()) => return,
                    Err(e) => bop_lib::set_install_notice(e),
                }
            }
            Err(e) => bop_lib::set_install_notice(format!("Bop could not install itself: {e}")),
        }
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

    /// Releases the console if only this process uses it (launched from Explorer); the window
    /// closes. If launched from a terminal, the terminal's console is left alone.
    pub fn release_own() {
        let mut ids = [0u32; 2];
        // SAFETY: the array and its length are passed correctly; on failure (0) nothing is done.
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
