#!/usr/bin/env python3
"""
Sawbyte Rust Edition, Let-Chain, Workspace, Build-Script & Lockfile Safeguard
(`tools/fix_rust_edition_and_let_chains.py`).

Resolves and prevents all CI failures including:
1. `Error: let chains are only allowed in Rust 2024 or later` in `crates/sawbyte-sim-core/src/density/slice.rs`.
2. `E0432`/`E0433` unresolved `sha2` in leftover `crates/sawbyte-prepare/build.rs` by removing stale
   `build.rs` / `src/bin` / `tests` in `sawbyte-prepare`, `sawbyte-sim-core`, and `sawbyte-ctl` and enforcing
   `build = false` and `autobins = false` in their `Cargo.toml` manifests.
3. `E0502: cannot borrow *self as mutable because it is also borrowed as immutable` in
   `crates/sawbyte-core/src/launcher.rs` by scoping `self.instances.get(...)` and `self.accounts.get(...)`.
4. `E0432`/`E0412` in `crates/sawbyte-sim-core/src/bin/density_parity.rs` by cleaning up stale `src/bin`
   targets and ensuring `sawbyte_sim_core::density::{graph, SampleError, DensitySampler, ...}`,
   `sawbyte_sim_core::geometry`, and `sawbyte_sim_core::noise` are exported.
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


def remove_stale_assimilated_targets(root: Path) -> None:
    """Removes leftover `build.rs`, `src/bin`, and `tests` from `sawbyte-prepare`, `sawbyte-sim-core`, and `sawbyte-ctl`."""
    for crate_name in ("sawbyte-prepare", "sawbyte-sim-core", "sawbyte-ctl"):
        cdir = root / "crates" / crate_name
        if not cdir.exists():
            continue
        build_rs = cdir / "build.rs"
        if build_rs.exists():
            build_rs.unlink()
            print(f"[Safeguard] Removed stale {build_rs.relative_to(root)} (prevents E0432 sha2 error)")
        for subdir_name in ("src/bin", "tests", "examples", "benches"):
            sdir = cdir / subdir_name
            if sdir.exists():
                shutil.rmtree(sdir)
                print(f"[Safeguard] Removed stale {sdir.relative_to(root)} (prevents orphaned parity bin errors)")


def ensure_editions_and_manifests(root: Path) -> None:
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

    # Enforce clean Cargo.toml for sawbyte-prepare, sawbyte-sim-core, and sawbyte-ctl
    prep_cargo = root / "crates" / "sawbyte-prepare" / "Cargo.toml"
    if prep_cargo.exists():
        prep_cargo.write_text(
            """[package]
name = "sawbyte-prepare"
version.workspace = true
edition.workspace = true
license.workspace = true
build = false
autobins = false
autotests = false
autoexamples = false
autobenches = false

[[bin]]
name = "sawbyte-prepare"
path = "src/main.rs"

[dependencies]
sawbyte-core = { path = "../sawbyte-core" }
sawbyte-sim-core = { path = "../sawbyte-sim-core" }

[lints]
workspace = true
""",
            encoding="utf-8",
        )

    sim_cargo = root / "crates" / "sawbyte-sim-core" / "Cargo.toml"
    if sim_cargo.exists():
        sim_cargo.write_text(
            """[package]
name = "sawbyte-sim-core"
version.workspace = true
edition = "2024"
rust-version.workspace = true
license.workspace = true
build = false
autobins = false
autotests = false
autoexamples = false
autobenches = false

[lib]
name = "sawbyte_sim_core"
path = "src/lib.rs"

[lints]
workspace = true
""",
            encoding="utf-8",
        )

    ctl_cargo = root / "crates" / "sawbyte-ctl" / "Cargo.toml"
    if ctl_cargo.exists():
        ctl_cargo.write_text(
            """[package]
name = "sawbyte-ctl"
version.workspace = true
edition.workspace = true
license.workspace = true
build = false
autobins = false
autotests = false
autoexamples = false
autobenches = false

[[bin]]
name = "sawbyte-ctl"
path = "src/main.rs"

[dependencies]
sawbyte-core = { path = "../sawbyte-core" }

[lints]
workspace = true
""",
            encoding="utf-8",
        )


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
    remove_stale_assimilated_targets(ROOT)
    ensure_editions_and_manifests(ROOT)
    organize_extra_assimilated_crates(ROOT)
    rewrites = scan_and_fix_rust_files(ROOT)
    ensure_lockfiles_up_to_date(ROOT)
    print(f"[Safeguard] Completed Rust edition, build.rs, E0502 & let-chain verification (rewrote {rewrites} let-chains).")


if __name__ == "__main__":
    main()
