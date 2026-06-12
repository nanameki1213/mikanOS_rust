# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
# ブートローダーをビルドして esp/EFI/BOOT/BOOTX64.EFI に配置する
cargo xtask build

# QEMU で起動する（-nographic、シリアルコンソール経由）
cargo xtask run
```

`xtask build` は内部で `bootloader/` ディレクトリに対して `cargo build` を実行し、
生成された `target/x86_64-unknown-uefi/debug/bootloader.efi` を `esp/EFI/BOOT/BOOTX64.EFI` にコピーする。

QEMU は `-nographic` で起動するため、入出力は端末上のシリアルコンソールになる。
デバッグ出力（iobase=0x402）は `debug.log` に書き出される。

OVMF ファームウェアは `edk2/Build/OvmfX64/DEBUG_GCC/FV/` 以下の
`OVMF_CODE.fd` / `OVMF_VARS.fd` を使用する。`OVMF_VARS.fd` は毎回プロジェクトルートへコピーされる。

## Architecture

### Workspace 構成

```
mikanOS_rust/
├── bootloader/   # UEFI ブートローダー（#[no_std], x86_64-unknown-uefi）
├── xtask/        # ビルド・実行を自動化する cargo xtask
├── esp/          # FAT イメージとして QEMU に渡す仮想ディスク
│   └── EFI/BOOT/BOOTX64.EFI
└── edk2/         # UEFI 仕様・OVMF ファームウェアの参照実装（変更しない）
```

### bootloader

- **ターゲット**: `x86_64-unknown-uefi`（`bootloader/.cargo/config.toml` で固定）
- **エントリポイント**: `efi_main(image_handle: Handle, system_table: *mut SystemTable) -> Status`
- **`src/uefi.rs`**: UEFI インターフェースを手書きで定義するモジュール。外部クレートに依存せず、EDK2 の `MdePkg/Include/Uefi/` を正とする。
  - `Status` (`#[repr(usize)]` enum) — ステータスコード
  - `BootServices` / `RuntimeServices` — 全フィールドを関数ポインタで定義
  - `SystemTable` — UEFI System Table
  - `SimpleTextInputProtocol` / `SimpleTextOutputProtocol` — コンソール I/O

### UEFI 固有の注意点

- 構造体はすべて `#[repr(C)]` が必須。`firmware_revision: u32` と次の pointer フィールドの間には 4 バイトのパディングが C ABI 準拠で自動挿入される。

### xtask

`xtask/src/main.rs` にサブコマンド `build` / `run` を実装する。
新しいビルドステップはここに追加する。
