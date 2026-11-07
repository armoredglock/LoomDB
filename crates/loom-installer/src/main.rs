use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const LOOM_BYTES: &[u8] = include_bytes!("../../../target/debug/loom.exe");
const STUDIO_HTML: &str = include_str!("../../loom-cli/src/studio.html");

fn main() {
    clear_screen();
    println!("========================================================================");
    println!("                     LOOMDB OFFICIAL INSTALLER");
    println!("        The World's Lightest Multi-Modal, AI-Native Database");
    println!("========================================================================\n");

    // --- STEP 1: SERVICE INFORMATION ---
    println!("STAGE 1 of 5: System & Service Information");
    println!("------------------------------------------------------------------------");
    println!("LoomDB is a zero-dependency, single-file multi-modal database:");
    println!("  * Single-File Architecture: Entire DB (Tables, Vectors, WAL) in one file.");
    println!("  * Multi-Modal Engine: SQL Relational, JSON Docs, AI Vectors, Time-Series.");
    println!("  * CLI Command: Run 'loom' from any terminal workspace.");
    println!("  * Background Daemon: Runs automatically on port 8484 with zero idle CPU.");
    println!("\nPress [Enter] to review the User Agreement...");
    let mut dummy = String::new();
    let _ = io::stdin().read_line(&mut dummy);

    // --- STEP 2: USER AGREEMENT ---
    clear_screen();
    println!("STAGE 2 of 5: User License Agreement");
    println!("------------------------------------------------------------------------");
    println!("LoomDB is free and open-source software distributed under dual");
    println!("Apache License, Version 2.0 and the MIT License.\n");
    println!("Summary:");
    println!("  1. Freedom to Use: Free for all commercial and personal projects.");
    println!("  2. No Warranty: Provided 'AS IS' without warranty of any kind.");
    println!("  3. Attribution: Retain original copyright notices when distributing.\n");
    
    print!("Do you accept these terms? (y/N): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    let _ = io::stdin().read_line(&mut choice);
    if !choice.trim().eq_ignore_ascii_case("y") {
        println!("\nInstallation aborted. You must accept the agreement to install LoomDB.");
        return;
    }

    // --- STEP 3: DESTINATION DIRECTORY ---
    clear_screen();
    println!("STAGE 3 of 5: Destination Directory");
    println!("------------------------------------------------------------------------");
    
    let default_dir = get_default_install_dir();
    println!("Default installation directory: {}", default_dir.display());
    print!("Press [Enter] to use default, or enter custom path: ");
    io::stdout().flush().unwrap();
    let mut custom_dir = String::new();
    let _ = io::stdin().read_line(&mut custom_dir);
    
    let install_dir = if custom_dir.trim().is_empty() {
        default_dir
    } else {
        PathBuf::from(custom_dir.trim())
    };

    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("Failed to create destination directory: {}", e);
        return;
    }

    // Extract embedded loom binary
    let exe_name = if cfg!(windows) { "loom.exe" } else { "loom" };
    let target_exe = install_dir.join(exe_name);

    print!("\nExtracting LoomDB binary... ");
    io::stdout().flush().unwrap();
    match File::create(&target_exe) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(LOOM_BYTES) {
                eprintln!("\nFailed to write binary: {}", e);
                return;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target_exe, fs::Permissions::from_mode(0o755));
            }
            println!("[OK]");
        }
        Err(e) => {
            eprintln!("\nError opening file {}: {}", target_exe.display(), e);
            return;
        }
    }

    // Extract studio.html
    let studio_path = install_dir.join("studio.html");
    let _ = fs::write(studio_path, STUDIO_HTML);

    // --- STEP 4: CLI PATH CONFIGURATION ---
    clear_screen();
    println!("STAGE 4 of 5: Terminal CLI Path Configuration");
    println!("------------------------------------------------------------------------");
    println!("Adding {} to User PATH environment variable...", install_dir.display());

    add_to_path(&install_dir);
    println!("[OK] PATH updated. You can run 'loom' from any terminal!");

    // --- STEP 5: SERVICE SETUP ---
    clear_screen();
    println!("STAGE 5 of 5: Background Service Setup");
    println!("------------------------------------------------------------------------");
    println!("Would you like LoomDB to run automatically as a background service?");
    println!("This allows applications to connect at 127.0.0.1:8484.\n");
    print!("Register as background service? (y/N): ");
    io::stdout().flush().unwrap();
    let mut svc_choice = String::new();
    let _ = io::stdin().read_line(&mut svc_choice);

    if svc_choice.trim().eq_ignore_ascii_case("y") {
        println!("Configuring background service...");
        let _ = Command::new(&target_exe)
            .args(["service", "install"])
            .status();
        println!("[OK] Service configured.");
    } else {
        println!("Skipped service registration. You can start it anytime using: loom server");
    }

    // --- FINISH ---
    clear_screen();
    println!("========================================================================");
    println!("                     INSTALLATION COMPLETE");
    println!("========================================================================\n");
    println!("LoomDB has been successfully installed on your machine.");
    println!("Binary location: {}", target_exe.display());
    println!("\nQuick Terminal Commands:");
    println!("  * loom             Launch the interactive SQL / LoomQL shell");
    println!("  * loom server      Start standalone server with Web Studio");
    println!("  * loom stats       Inspect container pages and memory consumption\n");

    print!("Launch the LoomDB interactive workspace now? (y/N): ");
    io::stdout().flush().unwrap();
    let mut run_choice = String::new();
    let _ = io::stdin().read_line(&mut run_choice);

    if run_choice.trim().eq_ignore_ascii_case("y") {
        let _ = Command::new(&target_exe).args(["repl"]).status();
    } else {
        println!("\nThank you for choosing LoomDB! Open any terminal and run 'loom' to begin.");
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

fn get_default_install_dir() -> PathBuf {
    if cfg!(windows) {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_app_data).join("LoomDB");
        }
        PathBuf::from("C:\\Program Files\\LoomDB")
    } else {
        if let Ok(home) = std::env::var("HOME") {
            let user_bin = PathBuf::from(home).join(".loom").join("bin");
            return user_bin;
        }
        PathBuf::from("/usr/local/bin")
    }
}

fn add_to_path(install_dir: &Path) {
    if cfg!(windows) {
        let dir_str = install_dir.to_string_lossy().to_string();
        let ps_cmd = format!(
            "$p = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($p -split ';' -notcontains '{}') {{ [Environment]::SetEnvironmentVariable('Path', \"$p;{}\", 'User') }}",
            dir_str, dir_str
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .status();
    } else {
        // Unix shell configuration
        if let Ok(home) = std::env::var("HOME") {
            let zshrc = PathBuf::from(&home).join(".zshrc");
            let bashrc = PathBuf::from(&home).join(".bashrc");
            let export_line = format!("\nexport PATH=\"$PATH:{}\"\n", install_dir.display());

            if zshrc.exists() {
                let _ = fs::OpenOptions::new().append(true).open(&zshrc)
                    .and_then(|mut f| f.write_all(export_line.as_bytes()));
            }
            if bashrc.exists() {
                let _ = fs::OpenOptions::new().append(true).open(&bashrc)
                    .and_then(|mut f| f.write_all(export_line.as_bytes()));
            }
        }
    }
}
