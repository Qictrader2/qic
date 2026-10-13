"""Install canonical local entrypoints without discarding existing files."""
from pathlib import Path
from datetime import datetime, timezone
import os
import plistlib
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def main():
    if sys.platform != "darwin":
        raise SystemExit("The scheduler is owned by the Mac mini; use the runner directly on other hosts")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    backup = ROOT / ".scratch/ticket-preparation" / f"installation-backup-{stamp}"
    links = {
        ".claude/scripts/qic_prepare.py": "ops/ticket-preparation/qic_prepare.py",
        ".claude/commands/references/qic-preparation.md": "ops/ticket-preparation/contract.md",
        ".claude/agents/qic-ticket-planner.md": "ops/ticket-preparation/agents/qic-ticket-planner.md",
        ".claude/agents/qic-design-expert.md": "ops/ticket-preparation/agents/qic-design-expert.md",
    }
    for relative, target in links.items():
        path = ROOT / relative
        canonical = ROOT / target
        if path.is_symlink() and path.resolve() == canonical:
            continue
        backup_path = backup / relative
        backup_path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists() or path.is_symlink():
            path.rename(backup_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.symlink_to(os.path.relpath(canonical, path.parent))
    plist = Path.home() / "Library/LaunchAgents/com.starbird.qic-prepare.plist"
    config = plistlib.loads(plist.read_bytes())
    backup.mkdir(parents=True, exist_ok=True)
    shutil.copy2(plist, backup / plist.name)
    config["ProgramArguments"][1] = str(ROOT / "ops/ticket-preparation/qic_prepare.py")
    config["StartInterval"] = 900
    plist.write_bytes(plistlib.dumps(config))
    domain = f"gui/{os.getuid()}"
    service = domain + "/com.starbird.qic-prepare"
    # Verify registration before unloading; a failed command is surfaced.
    subprocess.run(["launchctl", "print", service], check=True, capture_output=True)
    subprocess.run(["launchctl", "bootout", service], check=True)
    try:
        subprocess.run(["launchctl", "bootstrap", domain, str(plist)], check=True)
    except subprocess.CalledProcessError:
        shutil.copy2(backup / plist.name, plist)
        subprocess.run(["launchctl", "bootstrap", domain, str(plist)], check=True)
        raise
    print(f"Installed canonical preparation tooling; retained files at {backup}")


if __name__ == "__main__":
    main()
