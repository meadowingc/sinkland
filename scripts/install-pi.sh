#!/usr/bin/env bash
set -Eeuo pipefail

INSTALL_ROOT=/opt/sinkland
CONFIG_DIR=/etc/sinkland
UNIT_DIR=/etc/systemd/system
LOCK_FILE=/run/lock/sinkland-install.lock
REPOSITORY=meadowingc/sinkland
ASSET=sinkland-linux-arm64.tar.gz
MANAGED_MARKER="# Managed by sinkland install-pi.sh"

die() { printf 'Error: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
Usage: sudo bash scripts/install-pi.sh [options]

Install or update Sinkland and its dedicated Cloudflare Tunnel service.
Requires 64-bit ARM Linux, systemd, and an already installed cloudflared.

  --hostname HOST    Public hostname (default: sinkland.meadow.cafe).
                     This is a reminder/settings value, NOT a Cloudflare API change.
                     Configure HOST -> http://127.0.0.1:43796 in the tunnel dashboard.
  --token-file PATH  Read the tunnel token from a file (never pass the token itself).
                     First install prompts privately if this option is omitted.
                     Updates reuse the saved token.
  --cloudflared PATH Real cloudflared executable to copy for the service.
                     For mise: --cloudflared "$(mise which cloudflared)"
                     Default: sudo's PATH, then the previously installed copy.
  --version TAG      Install a specific vX.Y.Z release instead of GitHub's latest.
  --help             Show this help.

Existing hostname, token, and /etc/sinkland/sinkland.env are preserved on updates.
The existing cloudflared.service, if any, is left alone.
EOF
}

require_root() {
    [[ $EUID -eq 0 ]] || die "Run this script with sudo."
}

check_architecture() {
    [[ $(uname -s) == Linux && $(uname -m) == aarch64 && $(getconf LONG_BIT) == 64 ]] ||
        die "This release requires ARM64 Linux (aarch64, 64-bit Raspberry Pi OS)."
}

download() {
    curl --fail --silent --show-error --location --retry 3 \
        --connect-timeout 15 --max-time 300 --proto '=https' --proto-redir '=https' \
        "$1" --output "$2"
}

wait_http() {
    local url=$1 _attempt
    for _attempt in {1..30}; do
        if curl --noproxy '*' --fail --silent --show-error --max-time 2 "$url" > /dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    printf 'Readiness check failed: %s\n' "$url" >&2
    return 1
}

switch_release() {
    ln -sfn "$1" "$INSTALL_ROOT/current.next"
    mv -Tf "$INSTALL_ROOT/current.next" "$INSTALL_ROOT/current"
}

cleanup() {
    local status=$?
    trap - EXIT
    if [[ ${switched:-false} == true && $status -ne 0 ]]; then
        printf 'Sinkland failed to start; rolling back the application release.\n' >&2
        if [[ -n ${previous:-} ]]; then
            switch_release "$previous"
            systemctl restart sinkland.service ||
                printf 'Rollback restart failed; inspect journalctl -u sinkland.service.\n' >&2
        else
            systemctl disable --now sinkland.service sinkland-cloudflared.service ||
                printf 'Could not disable the incomplete installation services.\n' >&2
            rm -f "$INSTALL_ROOT/current"
        fi
    fi
    [[ -z ${work:-} ]] || rm -rf -- "$work"
    [[ -z ${staging:-} ]] || rm -rf -- "$staging"
    exit "$status"
}

extract_release() {
    local archive=$1 destination=$2
    # Validate before root extracts anything; only plain runtime files are expected.
    python3 - "$archive" <<'PY'
import pathlib
import sys
import tarfile

with tarfile.open(sys.argv[1]) as archive:
    for member in archive.getmembers():
        path = pathlib.PurePosixPath(member.name)
        if (path.is_absolute() or ".." in path.parts
                or not (member.isfile() or member.isdir())
                or (path.parts and path.parts[0] not in
                    {"sinkland", "assets", "templates", "static", "VERSION"})):
            sys.exit(f"Unsafe or unexpected archive entry: {member.name}")
PY
    tar --extract --gzip --file "$archive" --directory "$destination" \
        --no-same-owner --no-same-permissions
    [[ -f "$destination/sinkland" && -s "$destination/assets/haikus.json" &&
       -d "$destination/assets/books" && -d "$destination/templates" &&
       -d "$destination/static" ]] || die "Release is missing runtime files."
    [[ $(cat "$destination/VERSION") == "$version" ]] || die "Release version mismatch."
    find "$destination" -type d -exec chmod 755 {} +
    find "$destination" -type f -exec chmod 644 {} +
    chmod 755 "$destination/sinkland"
}

write_units() {
    cat > "$UNIT_DIR/sinkland.service" <<EOF
$MANAGED_MARKER
[Unit]
Description=Sinkland
After=network.target

[Service]
Type=simple
DynamicUser=yes
WorkingDirectory=$INSTALL_ROOT/current
ExecStart=$INSTALL_ROOT/current/sinkland
EnvironmentFile=$CONFIG_DIR/sinkland.env
Environment=SINKLAND_BIND=127.0.0.1:43796
Restart=on-failure
RestartSec=5
Nice=10
CPUWeight=20
IOWeight=20
IOSchedulingClass=idle
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
EOF
    cat > "$UNIT_DIR/sinkland-cloudflared.service" <<EOF
$MANAGED_MARKER
[Unit]
Description=Cloudflare Tunnel for Sinkland
Wants=network-online.target sinkland.service
After=network-online.target sinkland.service

[Service]
Type=notify
DynamicUser=yes
LoadCredential=tunnel-token:$CONFIG_DIR/tunnel-token
ExecStart=$cloudflared tunnel --no-autoupdate --metrics 127.0.0.1:43797 run --token-file \${CREDENTIALS_DIRECTORY}/tunnel-token
TimeoutStartSec=90
Restart=on-failure
RestartSec=5
Nice=10
CPUWeight=20
IOWeight=20
IOSchedulingClass=idle
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
EOF
    chmod 644 "$UNIT_DIR/sinkland.service" "$UNIT_DIR/sinkland-cloudflared.service"
}

main() {
    local hostname="" token_file="" requested_version=latest token="" command unit
    local metadata checksum expected release cloudflared_help cloudflared_source=""
    version="" previous="" work="" staging="" switched=false
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --hostname|--token-file|--version|--cloudflared)
                [[ $# -ge 2 && -n $2 && $2 != --* ]] || die "Missing value for $1."
                case "$1" in
                    --hostname) hostname=$2 ;;
                    --token-file) token_file=$2 ;;
                    --version) requested_version=$2 ;;
                    --cloudflared) cloudflared_source=$2 ;;
                esac
                shift 2 ;;
            --help|-h) usage; return ;;
            *) die "Unknown option: $1" ;;
        esac
    done
    require_root
    check_architecture
    for command in curl python3 tar sha256sum systemctl flock find install mktemp; do
        command -v "$command" > /dev/null || die "Install required command: $command"
    done
    cloudflared="$INSTALL_ROOT/bin/cloudflared"
    if [[ -z $cloudflared_source ]]; then
        cloudflared_source=$(command -v cloudflared || true)
        if [[ -z $cloudflared_source && -x $cloudflared ]]; then
            cloudflared_source=$cloudflared
        fi
    fi
    [[ -n $cloudflared_source ]] ||
        die "cloudflared is not in sudo PATH. Install it or pass --cloudflared \"\$(mise which cloudflared)\"."
    [[ $cloudflared_source == /* && -f $cloudflared_source && -x $cloudflared_source ]] ||
        die "cloudflared must be an absolute path to an executable file: $cloudflared_source"
    [[ $cloudflared_source != */shims/* ]] ||
        die "Pass the real executable, not a shim: --cloudflared \"\$(mise which cloudflared)\"."
    cloudflared_help=$("$cloudflared_source" tunnel run --help)
    [[ $cloudflared_help == *--token-file* ]] ||
        die "Update cloudflared: --token-file requires version 2025.4.0 or newer."
    metadata=$(systemctl --version)
    [[ $metadata =~ ^systemd[[:blank:]]+([0-9]+) ]] || die "Could not determine systemd version."
    (( BASH_REMATCH[1] >= 247 )) || die "systemd 247 or newer is required for tunnel credentials."
    systemctl show-environment > /dev/null || die "systemd must be running."
    exec 9>"$LOCK_FILE"
    flock -n 9 || die "Another Sinkland installer is running."
    for unit in sinkland.service sinkland-cloudflared.service; do
        metadata=$(systemctl cat "$unit" 2>/dev/null || true)
        if [[ -n $metadata && $metadata != *"$MANAGED_MARKER"* ]]; then
            die "$unit already exists and is not managed by this installer; refusing to overwrite it."
        fi
    done
    [[ ! -e "$INSTALL_ROOT/current" || -L "$INSTALL_ROOT/current" ]] ||
        die "$INSTALL_ROOT/current must be a symlink, not a directory."
    if [[ -z $hostname && -f "$CONFIG_DIR/hostname" ]]; then
        hostname=$(cat "$CONFIG_DIR/hostname")
    fi
    hostname=${hostname:-sinkland.meadow.cafe}
    [[ ${#hostname} -le 253 &&
       $hostname =~ ^([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,63}$ ]] ||
        die "Use a DNS hostname such as sinkland.meadow.cafe, without https:// or a path."
    [[ $requested_version == latest || $requested_version =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
        die "Version must be latest or a stable tag such as v0.1.0."
    if [[ -n $token_file ]]; then
        [[ -f $token_file ]] || die "Token file not found: $token_file"
        token=$(cat "$token_file")
    elif [[ ! -s "$CONFIG_DIR/tunnel-token" ]]; then
        [[ -t 0 ]] || die "First install needs --token-file PATH when not running interactively."
        read -r -s -p "Cloudflare tunnel token (hidden): " token
        printf '\n'
    fi
    if [[ -n $token_file || ! -s "$CONFIG_DIR/tunnel-token" ]]; then
        [[ $token =~ ^[A-Za-z0-9+/=_-]+$ ]] || die "Token must be nonempty and contain no whitespace."
    fi
    umask 077
    work=$(mktemp -d)
    trap cleanup EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    metadata="https://api.github.com/repos/$REPOSITORY/releases/latest"
    if [[ $requested_version != latest ]]; then
        metadata="https://api.github.com/repos/$REPOSITORY/releases/tags/$requested_version"
    fi
    download "$metadata" "$work/release.json"
    version=$(python3 - "$work/release.json" "$ASSET" <<'PY'
import json
import re
import sys

with open(sys.argv[1]) as file:
    release = json.load(file)
tag = release.get("tag_name", "")
assets = {asset["name"] for asset in release.get("assets", [])}
if (not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag)
        or release.get("draft") or release.get("prerelease")
        or not {sys.argv[2], sys.argv[2] + ".sha256"} <= assets):
    sys.exit("Release is not a complete stable ARM64 release.")
print(tag)
PY
    )
    [[ $requested_version == latest || $requested_version == "$version" ]] ||
        die "GitHub returned an unexpected release tag."
    printf 'Downloading Sinkland %s (ARM64)...\n' "$version"
    download "https://github.com/$REPOSITORY/releases/download/$version/$ASSET" "$work/$ASSET"
    download "https://github.com/$REPOSITORY/releases/download/$version/$ASSET.sha256" "$work/$ASSET.sha256"
    checksum=$(cat "$work/$ASSET.sha256")
    [[ $checksum =~ ^([[:xdigit:]]{64})[[:blank:]]+sinkland-linux-arm64.tar.gz$ ]] ||
        die "Invalid checksum file."
    expected=${BASH_REMATCH[1]}
    (cd "$work" && sha256sum --check --strict "$ASSET.sha256")
    install -d -m 755 "$INSTALL_ROOT" "$INSTALL_ROOT/releases" "$CONFIG_DIR" "$UNIT_DIR"
    release="$INSTALL_ROOT/releases/$version"
    if [[ -e $release ]]; then
        [[ -f "$release/.archive-sha256" && $(cat "$release/.archive-sha256") == "$expected" ]] ||
            die "Existing release directory does not match the downloaded release: $release"
    else
        staging=$(mktemp -d "$INSTALL_ROOT/releases/.staging.XXXXXX")
        extract_release "$work/$ASSET" "$staging"
        printf '%s\n' "$expected" > "$staging/.archive-sha256"
        mv "$staging" "$release"
        staging=""
    fi
    # A dynamic service user cannot access mise binaries inside a protected home.
    install -d -m 755 "$INSTALL_ROOT/bin"
    install -m 755 "$cloudflared_source" "$INSTALL_ROOT/bin/cloudflared.next"
    mv -Tf "$INSTALL_ROOT/bin/cloudflared.next" "$cloudflared"
    if [[ ! -f "$CONFIG_DIR/sinkland.env" ]]; then
        printf 'SINKLAND_FRIENDS=[]\n' > "$CONFIG_DIR/sinkland.env"
    fi
    chmod 600 "$CONFIG_DIR/sinkland.env"
    if [[ -n $token ]]; then
        printf '%s' "$token" > "$CONFIG_DIR/tunnel-token"
    fi
    chmod 600 "$CONFIG_DIR/tunnel-token"
    unset token
    printf '%s\n' "$hostname" > "$CONFIG_DIR/hostname"
    write_units
    systemctl daemon-reload
    if [[ -L "$INSTALL_ROOT/current" ]]; then
        previous=$(readlink "$INSTALL_ROOT/current")
    fi
    switched=true
    switch_release "$release"
    systemctl enable sinkland.service sinkland-cloudflared.service
    systemctl restart sinkland.service
    wait_http http://127.0.0.1:43796/
    systemctl is-active --quiet sinkland.service
    switched=false
    systemctl restart sinkland-cloudflared.service
    wait_http http://127.0.0.1:43797/ready
    systemctl is-active --quiet sinkland-cloudflared.service
    printf '\nInstalled Sinkland %s; both services are running at low CPU/I/O priority.\n' "$version"
    printf 'In the Cloudflare tunnel dashboard, configure %s -> http://127.0.0.1:43796\n' "$hostname"
    printf 'Public URL: https://%s/ (DNS/routing must be configured in Cloudflare).\n' "$hostname"
    printf 'Logs: journalctl -u sinkland -u sinkland-cloudflared\n'
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    main "$@"
fi
