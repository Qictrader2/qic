#!/usr/bin/env bash
set -euo pipefail

# twolebot installer
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/sjalq/twolebot/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/sjalq/twolebot/main/install.sh | bash -s -- --dev

REPO="sjalq/twolebot"
REPO_URL="https://github.com/${REPO}.git"
LOCAL_PREFIX="${HOME}/.local"
INSTALL_DIR="${LOCAL_PREFIX}/bin"
BINARY_NAME="twolebot"
DEFAULT_SOURCE_DIR="${HOME}/src/twolebot"
NODE_MIN_MAJOR=18
NODE_VERSION="${NODE_VERSION:-22.13.1}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}==>${NC} $1"; }
warn()    { echo -e "${YELLOW}==>${NC} $1"; }
error()   { echo -e "${RED}Error:${NC} $1" >&2; exit 1; }
step()    { echo -e "${BLUE}-->${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
has_cmd() { command -v "$1" &>/dev/null; }

prepend_path() {
    local dir="$1"
    if [[ -d "$dir" ]] && [[ ":$PATH:" != *":${dir}:"* ]]; then
        export PATH="${dir}:${PATH}"
    fi
}

# ============================================================================
# Platform Detection
# ============================================================================

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

require_cmd() {
    local cmd="$1"
    has_cmd "$cmd" || error "Missing required command: ${cmd}"
}

choose_shell_rc() {
    if [[ -n "${ZSH_VERSION:-}" ]] || [[ "${SHELL:-}" == */zsh ]]; then
        if [[ -f "${HOME}/.zshrc" ]]; then
            echo "${HOME}/.zshrc"
        elif [[ -f "${HOME}/.zprofile" ]]; then
            echo "${HOME}/.zprofile"
        else
            echo "${HOME}/.zshrc"
        fi
        return
    fi

    if [[ -f "${HOME}/.bashrc" ]]; then
        echo "${HOME}/.bashrc"
    elif [[ -f "${HOME}/.bash_profile" ]]; then
        echo "${HOME}/.bash_profile"
    else
        echo "${HOME}/.bashrc"
    fi
}

# ============================================================================
# Path Setup
# ============================================================================

ensure_path() {
    local dir="$1"

    prepend_path "$dir"

    local shell_rc
    shell_rc="$(choose_shell_rc)"

    if [[ -f "$shell_rc" ]] && grep -Fq "$dir" "$shell_rc" 2>/dev/null; then
        return 0
    fi

    echo "" >> "$shell_rc"
    echo "# Added by twolebot installer" >> "$shell_rc"
    echo "export PATH=\"${dir}:\$PATH\"" >> "$shell_rc"

    warn "Added ${dir} to PATH in ${shell_rc}"
    warn "Run: source ${shell_rc} (or restart your terminal)"
}

# ============================================================================
# Node.js + npm (rootless)
# ============================================================================

node_major_version() {
    node --version 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/'
}

node_is_compatible() {
    if ! has_cmd node || ! has_cmd npm; then
        return 1
    fi
    local major
    major="$(node_major_version)"
    [[ "$major" =~ ^[0-9]+$ ]] && [[ "$major" -ge "$NODE_MIN_MAJOR" ]]
}

install_node_rootless() {
    local os arch node_os node_arch archive url tmp_dir extracted install_root
    os="$(detect_os)"
    arch="$(detect_arch)"

    case "$os" in
        linux) node_os="linux" ;;
        macos) node_os="darwin" ;;
        *)     error "Unsupported OS for Node.js install: ${os}" ;;
    esac

    case "$arch" in
        x86_64) node_arch="x64" ;;
        arm64)  node_arch="arm64" ;;
        *)      error "Unsupported architecture for Node.js install: ${arch}" ;;
    esac

    archive="node-v${NODE_VERSION}-${node_os}-${node_arch}.tar.xz"
    url="https://nodejs.org/dist/v${NODE_VERSION}/${archive}"
    install_root="${LOCAL_PREFIX}/opt/node-v${NODE_VERSION}-${node_os}-${node_arch}"

    step "Installing Node.js v${NODE_VERSION} (rootless)..."
    step "Downloading ${archive}..."

    tmp_dir="$(mktemp -d)"
    mkdir -p "${LOCAL_PREFIX}/opt" "$INSTALL_DIR"
    if ! curl -fsSL --retry 3 --retry-delay 1 "$url" -o "${tmp_dir}/${archive}"; then
        rm -rf "$tmp_dir"
        error "Failed to download Node.js from ${url}"
    fi

    tar -xf "${tmp_dir}/${archive}" -C "$tmp_dir"
    extracted="${tmp_dir}/node-v${NODE_VERSION}-${node_os}-${node_arch}"
    [[ -d "$extracted" ]] || { rm -rf "$tmp_dir"; error "Unexpected Node.js archive layout"; }

    rm -rf "$install_root"
    mv "$extracted" "$install_root"

    for bin in node npm npx corepack; do
        if [[ -x "${install_root}/bin/${bin}" ]]; then
            ln -sf "${install_root}/bin/${bin}" "${INSTALL_DIR}/${bin}"
        fi
    done
    rm -rf "$tmp_dir"
    prepend_path "$INSTALL_DIR"
}

ensure_node_npm() {
    prepend_path "$INSTALL_DIR"

    if node_is_compatible; then
        success "node $(node --version), npm $(npm --version)"
        return 0
    fi

    if has_cmd node; then
        warn "Detected node $(node --version), but twolebot requires node >= ${NODE_MIN_MAJOR}."
    else
        warn "Node.js/npm not found."
    fi

    install_node_rootless

    if ! node_is_compatible; then
        error "Node.js install did not provide node >= ${NODE_MIN_MAJOR} with npm."
    fi

    success "node $(node --version), npm $(npm --version)"
}

# ============================================================================
# Dev Tool Installers (user-local)
# ============================================================================

ensure_rust() {
    prepend_path "${HOME}/.cargo/bin"
    if has_cmd rustc; then
        success "rust $(rustc --version | cut -d' ' -f2)"
        return 0
    fi

    require_cmd curl
    step "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

    # shellcheck disable=SC1090
    source "${HOME}/.cargo/env"
    prepend_path "${HOME}/.cargo/bin"
    success "rust $(rustc --version | cut -d' ' -f2)"
}

ensure_lamdera() {
    prepend_path "$INSTALL_DIR"
    if has_cmd lamdera; then
        success "lamdera $(lamdera --version 2>/dev/null | head -1 || echo 'installed')"
        return 0
    fi

    require_cmd curl
    step "Installing Lamdera..."

    local os arch lamdera_url
    os="$(detect_os)"
    arch="$(detect_arch)"

    case "${os}-${arch}" in
        macos-arm64)  lamdera_url="https://static.lamdera.com/bin/lamdera-1.3.2-macos-arm64" ;;
        macos-x86_64) lamdera_url="https://static.lamdera.com/bin/lamdera-1.3.2-macos-x86_64" ;;
        linux-x86_64) lamdera_url="https://static.lamdera.com/bin/lamdera-1.3.2-linux-x86_64" ;;
        linux-arm64)  lamdera_url="https://static.lamdera.com/bin/lamdera-1.3.2-linux-arm64" ;;
        *)            error "No Lamdera binary for ${os}-${arch}" ;;
    esac

    mkdir -p "$INSTALL_DIR"
    curl -fsSL "$lamdera_url" -o "${INSTALL_DIR}/lamdera"
    chmod +x "${INSTALL_DIR}/lamdera"
    prepend_path "$INSTALL_DIR"
    success "lamdera installed to ${INSTALL_DIR}"
}

ensure_claude_cli() {
    prepend_path "$INSTALL_DIR"
    if has_cmd claude; then
        success "claude $(claude --version 2>/dev/null | head -1 || echo 'installed')"
        return 0
    fi

    ensure_node_npm
    step "Installing Claude CLI to ${LOCAL_PREFIX}..."

    if ! npm install -g --prefix "${LOCAL_PREFIX}" @anthropic-ai/claude-code; then
        warn "User-local install failed; trying global npm install..."
        npm install -g @anthropic-ai/claude-code
    fi

    prepend_path "$INSTALL_DIR"
    has_cmd claude || error "Claude CLI install finished but 'claude' is not in PATH."
    success "claude $(claude --version 2>/dev/null | head -1 || echo 'installed')"
}

# ============================================================================
# System Prereqs (source/dev mode)
# ============================================================================

run_as_root() {
    if [[ "${EUID}" -eq 0 ]]; then
        "$@"
    else
        sudo "$@"
    fi
}

ensure_sudo_session() {
    if [[ "${EUID}" -eq 0 ]]; then
        return 0
    fi
    has_cmd sudo || error "Source install needs system packages, but 'sudo' is not available."
    step "Requesting sudo access for system package installation..."
    sudo -v || error "Could not obtain sudo credentials. Re-run after 'sudo -v' succeeds."
}

detect_linux_pkg_manager() {
    if has_cmd apt-get; then
        echo "apt"
    elif has_cmd dnf; then
        echo "dnf"
    elif has_cmd pacman; then
        echo "pacman"
    elif has_cmd zypper; then
        echo "zypper"
    elif has_cmd apk; then
        echo "apk"
    else
        echo "unknown"
    fi
}

install_linux_system_packages() {
    local manager="$1"
    shift
    local logical_pkgs=("$@")
    local os_pkgs=()

    for logical in "${logical_pkgs[@]}"; do
        case "${manager}:${logical}" in
            apt:compiler)    os_pkgs+=(build-essential) ;;
            apt:git)         os_pkgs+=(git) ;;
            apt:curl)        os_pkgs+=(curl ca-certificates) ;;
            apt:tar)         os_pkgs+=(tar) ;;
            apt:xz)          os_pkgs+=(xz-utils) ;;

            dnf:compiler)    os_pkgs+=(gcc gcc-c++ make) ;;
            dnf:git)         os_pkgs+=(git) ;;
            dnf:curl)        os_pkgs+=(curl ca-certificates) ;;
            dnf:tar)         os_pkgs+=(tar) ;;
            dnf:xz)          os_pkgs+=(xz) ;;

            pacman:compiler) os_pkgs+=(base-devel) ;;
            pacman:git)      os_pkgs+=(git) ;;
            pacman:curl)     os_pkgs+=(curl ca-certificates) ;;
            pacman:tar)      os_pkgs+=(tar) ;;
            pacman:xz)       os_pkgs+=(xz) ;;

            zypper:compiler) os_pkgs+=(gcc gcc-c++ make) ;;
            zypper:git)      os_pkgs+=(git) ;;
            zypper:curl)     os_pkgs+=(curl ca-certificates) ;;
            zypper:tar)      os_pkgs+=(tar) ;;
            zypper:xz)       os_pkgs+=(xz) ;;

            apk:compiler)    os_pkgs+=(build-base) ;;
            apk:git)         os_pkgs+=(git) ;;
            apk:curl)        os_pkgs+=(curl ca-certificates) ;;
            apk:tar)         os_pkgs+=(tar) ;;
            apk:xz)          os_pkgs+=(xz) ;;
        esac
    done

    [[ ${#os_pkgs[@]} -gt 0 ]] || return 0
    ensure_sudo_session

    step "Installing system packages via ${manager}: ${os_pkgs[*]}"
    case "$manager" in
        apt)
            run_as_root apt-get update
            run_as_root apt-get install -y "${os_pkgs[@]}"
            ;;
        dnf)
            run_as_root dnf install -y "${os_pkgs[@]}"
            ;;
        pacman)
            run_as_root pacman -Sy --noconfirm "${os_pkgs[@]}"
            ;;
        zypper)
            run_as_root zypper --non-interactive install "${os_pkgs[@]}"
            ;;
        apk)
            run_as_root apk add --no-cache "${os_pkgs[@]}"
            ;;
        *)
            error "No supported Linux package manager found."
            ;;
    esac
}

ensure_macos_dev_tools() {
    local missing=("$@")
    [[ ${#missing[@]} -eq 0 ]] && return 0

    if ! xcode-select -p >/dev/null 2>&1; then
        warn "Source install requires Xcode Command Line Tools (git + compiler)."
        step "Starting 'xcode-select --install'..."
        xcode-select --install >/dev/null 2>&1 || true
        error "Complete Xcode Command Line Tools installation, then re-run install.sh --dev"
    fi

    has_cmd git && (has_cmd clang || has_cmd gcc) && return 0
    error "Required dev tools (git + clang/gcc) are still missing. Install Xcode Command Line Tools and retry."
}

ensure_source_system_prereqs() {
    local missing=()
    local os
    os="$(detect_os)"

    has_cmd git || missing+=(git)
    (has_cmd clang || has_cmd gcc) || missing+=(compiler)
    has_cmd curl || missing+=(curl)
    has_cmd tar || missing+=(tar)
    has_cmd xz || missing+=(xz)

    if [[ ${#missing[@]} -eq 0 ]]; then
        return 0
    fi

    info "Checking source-build system prerequisites..."
    if [[ "$os" == "linux" ]]; then
        local manager
        manager="$(detect_linux_pkg_manager)"
        [[ "$manager" != "unknown" ]] || error "Unsupported Linux distro (no apt/dnf/pacman/zypper/apk found)."
        install_linux_system_packages "$manager" "${missing[@]}"
    else
        ensure_macos_dev_tools "${missing[@]}"
    fi

    # Final verification
    has_cmd git || error "git is still missing after install."
    (has_cmd clang || has_cmd gcc) || error "compiler (clang/gcc) is still missing after install."
    has_cmd curl || error "curl is still missing after install."
    has_cmd tar || error "tar is still missing after install."
    has_cmd xz || warn "xz command not found (tar may still handle .tar.xz archives)."
}

# ============================================================================
# Binary Install (runtime)
# ============================================================================

get_latest_version() {
    local latest_url
    latest_url="$(curl -fsSI "https://github.com/${REPO}/releases/latest" | tr -d '\r' | awk 'tolower($1)=="location:" {print $2}')"
    [[ -n "$latest_url" ]] || error "Could not resolve latest release URL."
    basename "$latest_url"
}

install_binary() {
    require_cmd curl
    require_cmd tar

    local os arch platform version archive_name url tmp_dir
    os="$(detect_os)"
    arch="$(detect_arch)"
    platform="${os}-${arch}"

    info "Installing twolebot runtime (binary + CLI dependencies)..."
    step "Platform: ${platform}"

    step "Fetching latest version..."
    version="$(get_latest_version)"
    [[ -n "$version" ]] || error "Could not determine latest version."
    step "Version: ${version}"

    archive_name="twolebot-${platform}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"

    step "Downloading ${archive_name}..."
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    if ! curl -fsSL "$url" -o "${tmp_dir}/${archive_name}"; then
        error "Failed to download ${archive_name} for ${platform}."
    fi

    step "Extracting..."
    tar -xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

    step "Installing binary to ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR"
    mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Install frontend files
    local data_dir
    if [[ "$os" == "macos" ]]; then
        data_dir="${HOME}/Library/Application Support/twolebot"
    else
        data_dir="${XDG_DATA_HOME:-${HOME}/.local/share}/twolebot"
    fi

    if [[ -d "${tmp_dir}/frontend" ]]; then
        step "Installing frontend to ${data_dir}/frontend/dist..."
        mkdir -p "${data_dir}/frontend/dist"
        cp -r "${tmp_dir}/frontend/"* "${data_dir}/frontend/dist/"
    fi

    info "Installing runtime dependencies in user space..."
    ensure_node_npm
    ensure_claude_cli
    ensure_path "$INSTALL_DIR"

    echo ""
    success "twolebot ${version} installed."
    echo "  Run: ${BOLD}twolebot${NC}"
    echo "  Then run: ${BOLD}claude login${NC} (if needed)"
    echo ""
}

# ============================================================================
# Source Install (dev)
# ============================================================================

install_source() {
    info "Installing twolebot from source (dev mode)..."
    echo ""

    ensure_source_system_prereqs

    # Determine source directory
    local source_dir=""
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    if [[ -f "${script_dir}/Cargo.toml" ]] && grep -q "name = \"twolebot\"" "${script_dir}/Cargo.toml" 2>/dev/null; then
        source_dir="$script_dir"
        step "Using current directory: ${source_dir}"
    elif [[ -t 0 ]]; then
        echo -n "Source directory [${DEFAULT_SOURCE_DIR}]: "
        read -r source_dir
        source_dir="${source_dir:-$DEFAULT_SOURCE_DIR}"
    else
        source_dir="$DEFAULT_SOURCE_DIR"
    fi

    echo ""
    info "Installing dev/runtime tools (user-local)..."
    ensure_rust
    ensure_lamdera
    ensure_node_npm
    ensure_claude_cli

    echo ""
    info "Getting source code..."
    if [[ -d "$source_dir/.git" ]]; then
        step "Checking for updates..."
        git -C "$source_dir" pull --ff-only 2>/dev/null || step "Skipping pull (offline or no remote access)"
    else
        step "Cloning to ${source_dir}..."
        mkdir -p "$(dirname "$source_dir")"
        git clone "$REPO_URL" "$source_dir"
    fi

    echo ""
    info "Building..."
    cd "$source_dir"
    ./compile.sh

    echo ""
    info "Installing binary symlink..."
    mkdir -p "$INSTALL_DIR"
    ln -sf "${source_dir}/target/debug/twolebot" "${INSTALL_DIR}/twolebot"
    step "Symlinked to ${INSTALL_DIR}/twolebot"

    ensure_path "$INSTALL_DIR"
    ensure_path "${HOME}/.cargo/bin"

    echo ""
    echo -e "${GREEN}╭─────────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${GREEN}│${NC}  ${BOLD}✓ Dev installation complete${NC}                                   ${GREEN}│${NC}"
    echo -e "${GREEN}│${NC}  Source: ${source_dir}"
    echo -e "${GREEN}│${NC}  Binary: ${INSTALL_DIR}/twolebot"
    echo -e "${GREEN}│${NC}                                                                 ${GREEN}│${NC}"
    echo -e "${GREEN}│${NC}  ${BOLD}Next steps:${NC}                                                   ${GREEN}│${NC}"
    echo -e "${GREEN}│${NC}  1. Restart terminal (or source your shell rc)                 ${GREEN}│${NC}"
    echo -e "${GREEN}│${NC}  2. Run: ${BOLD}claude login${NC}  (if not authenticated)                 ${GREEN}│${NC}"
    echo -e "${GREEN}│${NC}  3. Run: ${BOLD}twolebot${NC}                                               ${GREEN}│${NC}"
    echo -e "${GREEN}╰─────────────────────────────────────────────────────────────────╯${NC}"
    echo ""
}

# ============================================================================
# Uninstall
# ============================================================================

uninstall() {
    info "Uninstalling twolebot..."

    if [[ -L "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        local link_target source_dir=""
        link_target="$(readlink "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true)"
        if [[ -n "$link_target" ]]; then
            if [[ "$link_target" != /* ]]; then
                link_target="$(cd "$(dirname "${INSTALL_DIR}/${BINARY_NAME}")" && cd "$(dirname "${link_target}")" && pwd)/$(basename "${link_target}")"
            fi
            source_dir="${link_target%/target/debug/twolebot}"
        fi

        rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        success "Removed symlink"

        if [[ -n "$source_dir" ]] && [[ "$source_dir" != "$link_target" ]]; then
            echo ""
            echo -n "Also remove source directory ${source_dir}? [y/N]: "
            read -r remove_source
            if [[ "${remove_source,,}" == "y" ]]; then
                rm -rf "$source_dir"
                success "Removed source directory"
            fi
        fi
    elif [[ -f "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        success "Removed binary"
    else
        warn "Binary not found at ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    local data_dir
    if [[ "$(detect_os)" == "macos" ]]; then
        data_dir="${HOME}/Library/Application Support/twolebot"
    else
        data_dir="${XDG_DATA_HOME:-${HOME}/.local/share}/twolebot"
    fi

    if [[ -d "$data_dir" ]]; then
        echo ""
        echo -n "Remove data directory ${data_dir}? [y/N]: "
        read -r remove_data
        if [[ "${remove_data,,}" == "y" ]]; then
            rm -rf "$data_dir"
            success "Removed data directory"
        fi
    fi

    echo ""
    success "Uninstall complete"
    echo "Note: Node.js/Claude CLI/Rust/Lamdera were left installed in user directories."
    echo ""
}

# ============================================================================
# Main
# ============================================================================

show_banner() {
    echo ""
    echo -e "${BOLD}  twolebot installer${NC}"
    echo ""
}

show_menu() {
    echo "  [1] Runtime install (binary + local deps, no sudo)"
    echo "  [2] Dev install (source build; may need sudo for system packages)"
    echo ""
    echo -n "  Choice [1]: "
    read -r choice

    case "${choice:-1}" in
        1) install_binary ;;
        2) install_source ;;
        *) install_binary ;;
    esac
}

main() {
    show_banner

    local mode=""
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --runtime|runtime|--binary|binary|-b) mode="binary"; shift ;;
            --dev|dev|--source|source|-s)         mode="source"; shift ;;
            --uninstall|-u)                       mode="uninstall"; shift ;;
            --help|-h)
                echo "Usage: install.sh [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --runtime, --binary, -b   Runtime install (default)"
                echo "  --dev, --source, -s       Source/dev install"
                echo "  --uninstall, -u           Remove twolebot"
                echo "  --help, -h                Show this help"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done

    case "$mode" in
        source)    install_source ;;
        binary)    install_binary ;;
        uninstall) uninstall ;;
        *)
            if [[ -t 0 ]]; then
                show_menu
            else
                install_binary
            fi
            ;;
    esac
}

main "$@"
