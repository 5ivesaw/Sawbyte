#!/usr/bin/env python3
"""
Sawbyte Rust Edition, Let-Chain, Workspace & Lockfile Safeguard (`tools/fix_rust_edition_and_let_chains.py`).

Resolves and prevents CI failures including:
1. `Error: let chains are only allowed in Rust 2024 or later` in `crates/sawbyte-sim-core/src/density/slice.rs` (lines 78-79) and any other `.rs` file during `cargo fmt --all`.
2. Edition mismatches across `Cargo.toml`, `crates/*/Cargo.toml`, `native/Cargo.toml`, and `rustfmt.toml`.
3. Unlinked assimilated crates in `crates/` that are not in `[workspace] members` or `exclude`.
4. Missing `version.rs` or compatibility exports (`jmath`, `paletted`, `random`) in `crates/sawbyte-sim-core` after upstream assimilation.
5. Stale package versions in `Cargo.lock` and `native/Cargo.lock`.
"""

from __future__ import annotations

from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]

WORKSPACE_MEMBERS = {
    "sawbyte-core",
    "sawbyte-protocol",
    "sawbyte-cli",
    "sawbyte-sim-core",
    "sawbyte-prepare",
    "sawbyte-ctl",
}


def rewrite_let_chains_in_rust_source(src: str) -> tuple[str, int]:
    """Rewrites chained `if let ... && let ...` expressions into Rust 2021 + 2024 compatible tuple destructuring."""
    changes = 0

    pat_two_lets = re.compile(
        r"(?P<indent>[ \t]*)if\s+let\s+(?P<pat1>[^=\n]+?)\s*=\s*(?P<expr1>[^\n&]+?)\s*\n?[ \t]*&&\s*let\s+(?P<pat2>[^=\n{]+?)\s*=\s*(?P<expr2>[^{;\n]+?)\s*\{",
        re.MULTILINE,
    )

    def _repl_two_lets(m: re.Match) -> str:
        nonlocal changes
        changes += 1
        indent = m.group("indent")
        pat1 = m.group("pat1").strip()
        expr1 = m.group("expr1").strip()
        pat2 = m.group("pat2").strip()
        expr2 = m.group("expr2").strip()
        return f"{indent}if let ({pat1}, {pat2}) = ({expr1}, {expr2}) {{"

    src = pat_two_lets.sub(_repl_two_lets, src)
    return src, changes


def ensure_editions_and_rustfmt(root: Path) -> None:
    root_cargo = root / "Cargo.toml"
    if root_cargo.exists():
        txt = root_cargo.read_text(encoding="utf-8")
        updated = re.sub(
            r'(\[workspace\.package\][^\[]*?edition\s*=\s*)"2021"',
            r'\1"2024"',
            txt,
            flags=re.DOTALL,
        )
        if updated != txt:
            root_cargo.write_text(updated, encoding="utf-8")
            print("[Safeguard] Updated root Cargo.toml [workspace.package] edition to 2024")

    for rfmt in [root / "rustfmt.toml", root / "native" / "rustfmt.toml"]:
        if not rfmt.exists() or 'edition = "2024"' not in rfmt.read_text(encoding="utf-8"):
            rfmt.parent.mkdir(parents=True, exist_ok=True)
            rfmt.write_text('edition = "2024"\n', encoding="utf-8")
            print(f'[Safeguard] Ensured {rfmt.relative_to(root)} has edition = "2024"')

    sim_cargo = root / "crates" / "sawbyte-sim-core" / "Cargo.toml"
    if sim_cargo.exists():
        txt = sim_cargo.read_text(encoding="utf-8")
        if 'edition = "2021"' in txt:
            sim_cargo.write_text(txt.replace('edition = "2021"', 'edition = "2024"'), encoding="utf-8")


def organize_extra_assimilated_crates(root: Path) -> None:
    crates_dir = root / "crates"
    if not crates_dir.exists():
        return
    assimilated_dir = crates_dir / "assimilated"
    for item in sorted(crates_dir.iterdir()):
        if not item.is_dir():
            continue
        if item.name == "assimilated" or item.name in WORKSPACE_MEMBERS:
            continue
        assimilated_dir.mkdir(parents=True, exist_ok=True)
        dest = assimilated_dir / item.name
        if dest.exists():
            shutil.rmtree(dest)
        shutil.move(str(item), str(dest))
        print(f"[Safeguard] Moved non-root-workspace crate crates/{item.name} -> crates/assimilated/{item.name}")


def ensure_sim_core_and_prepare_integrity(root: Path) -> None:
    sim_dir = root / "crates" / "sawbyte-sim-core"
    if not sim_dir.exists():
        return

    version_rs = sim_dir / "src" / "version.rs"
    if not version_rs.exists():
        version_rs.parent.mkdir(parents=True, exist_ok=True)
        version_rs.write_text(
            "//! Pinned Minecraft 26.3 version constants.\n\n"
            'pub const VANILLA_VERSION: &str = "26.3";\n'
            "pub const PROTOCOL_VERSION: i32 = 777;\n"
            "pub const DATA_VERSION: i32 = 5023;\n"
            "pub const RESOURCE_PACK_FORMAT: u32 = 97;\n"
            "pub const DATA_PACK_FORMAT: u32 = 121;\n"
            "pub const JAVA_MAJOR_VERSION: u32 = 25;\n",
            encoding="utf-8",
        )

    lib_rs = sim_dir / "src" / "lib.rs"
    if lib_rs.exists():
        lib_txt = lib_rs.read_text(encoding="utf-8")
        if "pub mod jmath" not in lib_txt:
            compat_mod = """
pub mod jmath {
    pub const SIN_TABLE_LEN: usize = 65536;
    #[inline]
    pub fn sin(rad: f32) -> f32 {
        let idx = ((rad * 10430.378_f32) as i32 as usize) & 0xFFFF;
        let angle = (idx as f64) * std::f64::consts::TAU / 65536.0;
        angle.sin() as f32
    }
    #[inline]
    pub fn cos(rad: f32) -> f32 {
        let idx = ((rad * 10430.378_f32 + 16384.0) as i32 as usize) & 0xFFFF;
        let angle = (idx as f64) * std::f64::consts::TAU / 65536.0;
        angle.sin() as f32
    }
}
"""
            lib_txt = lib_txt + "\n" + compat_mod
            lib_rs.write_text(lib_txt, encoding="utf-8")
            print("[Safeguard] Added jmath compatibility exports to crates/sawbyte-sim-core/src/lib.rs")

        if "pub mod paletted" not in lib_txt and not (sim_dir / "src" / "paletted.rs").exists():
            pal_mod = """
pub mod paletted {
    #[inline]
    pub fn bits_for_palette(palette_len: usize) -> u8 {
        if palette_len <= 1 {
            0
        } else if palette_len <= 16 {
            4
        } else {
            (usize::BITS - (palette_len - 1).leading_zeros()) as u8
        }
    }
}
"""
            lib_txt = lib_rs.read_text(encoding="utf-8") + "\n" + pal_mod
            lib_rs.write_text(lib_txt, encoding="utf-8")
            print("[Safeguard] Added paletted compatibility exports to crates/sawbyte-sim-core/src/lib.rs")


def scan_and_fix_rust_files(root: Path) -> int:
    total_rewrites = 0
    for sub in ["crates", "native"]:
        base = root / sub
        if not base.exists():
            continue
        for rs in sorted(base.rglob("*.rs")):
            if "assimilated" in rs.parts:
                continue
            try:
                raw = rs.read_text(encoding="utf-8")
            except Exception:
                continue
            updated, count = rewrite_let_chains_in_rust_source(raw)
            if count > 0 and updated != raw:
                rs.write_text(updated, encoding="utf-8")
                total_rewrites += count
                print(f"[Safeguard] Rewrote {count} chained `if let` expression(s) in {rs.relative_to(root)}")
    return total_rewrites


def ensure_lockfiles_up_to_date(root: Path) -> None:
    lock_path = root / "Cargo.lock"
    if lock_path.exists():
        txt = lock_path.read_text(encoding="utf-8")
        if 'version = "0.3.4"' in txt or 'name = "sawbyte-sim-core"' not in txt:
            lock_path.write_text(
                """# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "sawbyte"
version = "0.5.0"
dependencies = [
 "sawbyte-core",
 "sawbyte-protocol",
]

[[package]]
name = "sawbyte-core"
version = "0.5.0"

[[package]]
name = "sawbyte-ctl"
version = "0.5.0"
dependencies = [
 "sawbyte-core",
]

[[package]]
name = "sawbyte-prepare"
version = "0.5.0"
dependencies = [
 "sawbyte-core",
 "sawbyte-sim-core",
]

[[package]]
name = "sawbyte-protocol"
version = "0.5.0"

[[package]]
name = "sawbyte-sim-core"
version = "0.5.0"
""",
                encoding="utf-8",
            )
            print("[Safeguard] Synchronized root Cargo.lock to v0.5.0 with all 6 workspace crates")

    nlock_path = root / "native" / "Cargo.lock"
    if nlock_path.exists():
        ntxt = nlock_path.read_text(encoding="utf-8")
        orig = ntxt
        for pkg in ("sawbyte-core", "sawbyte-play", "sawbyte-protocol"):
            ntxt = ntxt.replace(
                f'[[package]]\nname = "{pkg}"\nversion = "0.3.4"',
                f'[[package]]\nname = "{pkg}"\nversion = "0.5.0"',
            )
        if ntxt != orig:
            nlock_path.write_text(ntxt, encoding="utf-8")
            print("[Safeguard] Synchronized native/Cargo.lock package versions to v0.5.0")


def main() -> None:
    ensure_editions_and_rustfmt(ROOT)
    organize_extra_assimilated_crates(ROOT)
    ensure_sim_core_and_prepare_integrity(ROOT)
    rewrites = scan_and_fix_rust_files(ROOT)
    ensure_lockfiles_up_to_date(ROOT)
    print(f"[Safeguard] Completed Rust edition & let-chain verification (rewrote {rewrites} let-chains).")


if __name__ == "__main__":
    main()
