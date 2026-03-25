use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("build") => build()?,
        Some("run") => run()?,
        Some("help") | Some("-h") | Some("--help") => print_help(),
        _ => {
            print_help();
            return Err("Unknown or missing command.".into());
        }
    }
    Ok(())
}

fn build() -> Result<(), DynError> {
    let bootloader_path = project_root().join("bootloader");
    let efi_dst = project_root().join("esp/EFI/BOOT/BOOTX64.EFI");
    let efi_src = project_root().join("target/x86_64-unknown-uefi/debug/bootloader.efi");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(&cargo)
        .current_dir(&bootloader_path)
        .arg("build")
        .status()?;

    if !status.success() {
        return Err("cargo build failed.".into());
    }

    fs::create_dir_all(efi_dst.parent().unwrap())?;
    fs::copy(&efi_src, &efi_dst)?;
    eprintln!("Copied {} -> {}", efi_src.display(), efi_dst.display());

    Ok(())
}

fn run() -> Result<(), DynError> {
    let ovmf_vars_src =
        project_root().join("edk2/Build/OvmfX64/DEBUG_GCC/FV/OVMF_VARS.fd");
    let ovmf_vars_dst = project_root().join("OVMF_VARS.fd");
    let ovmf_code = project_root().join("edk2/Build/OvmfX64/DEBUG_GCC/FV/OVMF_CODE.fd");

    // OVMF_VARS.fd は QEMU が書き込む可能性があるため毎回新鮮なコピーを用意する
    fs::copy(&ovmf_vars_src, &ovmf_vars_dst)?;

    let status = Command::new("qemu-system-x86_64")
        .current_dir(project_root())
        .args([
            "-drive",
            &format!(
                "if=pflash,format=raw,readonly=on,file={}",
                ovmf_code.display()
            ),
            "-drive",
            &format!("if=pflash,format=raw,file={}", ovmf_vars_dst.display()),
            "-drive",
            "format=raw,file=fat:rw:esp",
            "-net",
            "none",
            "-nographic",
            "-debugcon",
            "file:debug.log",
            "-global",
            "isa-debugcon.iobase=0x402",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("QEMU exited with non-zero status.".into());
    }

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn print_help() {
    eprintln!(
        "Usage: cargo xtask <command>

Commands:
  build    ブートローダーをビルドして esp/EFI/BOOT/BOOTX64.EFI に配置する
  run      OVMF_VARS.fd をコピーし QEMU でブートローダーを起動する
  help     このヘルプを表示する"
    );
}
