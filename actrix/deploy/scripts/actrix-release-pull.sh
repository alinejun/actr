#!/usr/bin/env bash
#
# actrix-release-pull.sh — manually pull an Actrix binary from a GitHub Release
# and switch the running version.
#
# Actrix is built by the Actr Release Train and published to a GitHub Release.
# This script never runs automatically: an administrator must invoke it with an
# explicit --tag <version> or --latest. Without one of these it exits without
# downloading or switching anything.
#
# Usage:
#   sudo actrix-release-pull.sh --tag v0.4.3
#   sudo actrix-release-pull.sh --tag v0.4.3 --install-dir /opt/actrix
#   sudo actrix-release-pull.sh --latest
#   sudo actrix-release-pull.sh --tag v0.4.3 --no-restart   # first install
#
# Configuration is read from /etc/actrix/release.env when present (see
# actrix-release.env.example). Command-line flags override the file.
#
# Exit codes:
#   0  success
#   1  usage / precondition error
#   2  download or verification failure
#   3  install or service failure (rolled back when possible)

set -euo pipefail

# ───────────────────────── constants & defaults ─────────────────────────
REPO_DEFAULT="Actrium/actr"
SERVICE_NAME_DEFAULT="actrix-managed"
HEALTH_WAIT_DEFAULT="3"
CONFIG_PATH_DEFAULT="/etc/actrix/config.toml"
RELEASE_ENV_FILE="/etc/actrix/release.env"

# Asset name template per architecture. The Release Train publishes raw
# binaries named actrix-linux-x86_64 / actrix-linux-arm64.
asset_name_for_arch() {
    case "$1" in
        x86_64)         echo "actrix-linux-x86_64" ;;
        aarch64|arm64)  echo "actrix-linux-arm64"  ;;
        *)
            print_error "Unsupported architecture: $1"
            print_error "Only Linux x86_64 and arm64 binaries are published."
            exit 1
            ;;
    esac
}

# ───────────────────────── output helpers ─────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
print_step()  { echo -e "${BLUE}[STEP]${NC}  $*"; }
print_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
print_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

die() { print_error "$*"; exit "${2:-1}"; }

# ───────────────────────── argument parsing ─────────────────────────
TAG=""
LATEST=0
INSTALL_DIR_ARG=""
NO_RESTART=0
CONFIG_PATH_ARG=""
WORKING_DIR_ARG=""

usage() {
    cat <<EOF
Usage: $0 --tag <tag> | --latest [options]

Required (exactly one):
  --tag <tag>        Pull a specific Release tag, e.g. v0.4.3
  --latest           Pull the latest stable Release

Optional:
  --install-dir <path>   Install root (default: current directory)
  --config <path>        Config file passed to actrix (default: ${CONFIG_PATH_DEFAULT})
  --working-dir <path>   systemd WorkingDirectory (default: dirname of --config)
  --no-restart           Prepare version + symlink + service file, do not restart
  -h, --help             Show this help

Environment (overridable in ${RELEASE_ENV_FILE}):
  ACTRIX_REPOSITORY          GitHub owner/repo (default: ${REPO_DEFAULT})
  ACTRIX_SERVICE_NAME        systemd unit name (default: ${SERVICE_NAME_DEFAULT})
  ACTRIX_HEALTH_WAIT_SECONDS seconds to wait for service to become active (default: ${HEALTH_WAIT_DEFAULT})
  ACTRIX_CONFIG_PATH         default --config value
  ACTRIX_WORKING_DIR         default --working-dir value
  GITHUB_TOKEN               only needed for private repositories (Contents: Read)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --tag)           TAG="$2"; shift 2 ;;
        --latest)        LATEST=1; shift ;;
        --install-dir)   INSTALL_DIR_ARG="$2"; shift 2 ;;
        --config)        CONFIG_PATH_ARG="$2"; shift 2 ;;
        --working-dir)   WORKING_DIR_ARG="$2"; shift 2 ;;
        --no-restart)    NO_RESTART=1; shift ;;
        -h|--help)       usage; exit 0 ;;
        *)               print_error "Unknown argument: $1"; usage; exit 1 ;;
    esac
done

# ───────────────────────── load release.env ─────────────────────────
if [ -f "$RELEASE_ENV_FILE" ]; then
    print_info "Loading configuration from ${RELEASE_ENV_FILE}"
    # shellcheck disable=SC1090
    set -a
    . "$RELEASE_ENV_FILE"
    set +a
fi

REPOSITORY="${ACTRIX_REPOSITORY:-$REPO_DEFAULT}"
SERVICE_NAME="${ACTRIX_SERVICE_NAME:-$SERVICE_NAME_DEFAULT}"
HEALTH_WAIT="${ACTRIX_HEALTH_WAIT_SECONDS:-$HEALTH_WAIT_DEFAULT}"
CONFIG_PATH="${CONFIG_PATH_ARG:-${ACTRIX_CONFIG_PATH:-$CONFIG_PATH_DEFAULT}}"
WORKING_DIR="${WORKING_DIR_ARG:-${ACTRIX_WORKING_DIR:-}}"

# Resolve working dir default = dirname of config (so relative paths inside
# the config resolve the same way they did under the previous deployment).
if [ -z "$WORKING_DIR" ]; then
    WORKING_DIR="$(dirname "$CONFIG_PATH")"
fi

# ───────────────────────── preconditions ─────────────────────────
print_step "Checking preconditions"

if [ "$EUID" -ne 0 ]; then
    die "This script must be run as root (need to write systemd units and /etc/actrix)."
fi

if [ -z "$TAG" ] && [ "$LATEST" -ne 1 ]; then
    print_error "No version requested. Pass --tag <tag> or --latest."
    usage
    exit 1
fi
if [ -n "$TAG" ] && [ "$LATEST" -eq 1 ]; then
    die "Pass either --tag or --latest, not both."
fi

ARCH="$(uname -m)"
ASSET_NAME="$(asset_name_for_arch "$ARCH")"
print_info "Architecture: ${ARCH}  →  asset: ${ASSET_NAME}"

for cmd in curl jq systemctl mktemp install ln; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        die "Required command not found: $cmd"
    fi
done

# ───────────────────────── resolve install dir ─────────────────────────
if [ -n "$INSTALL_DIR_ARG" ]; then
    INSTALL_DIR="$INSTALL_DIR_ARG"
else
    INSTALL_DIR="$(pwd -P)"
fi
# Convert to absolute path.
INSTALL_DIR="$(cd "$INSTALL_DIR" 2>/dev/null && pwd -P)" || INSTALL_DIR="$(readlink -f "$INSTALL_DIR_ARG" 2>/dev/null || echo "$INSTALL_DIR_ARG")"

if [ -L "$INSTALL_DIR" ]; then
    die "Install directory is a symlink, refusing: $INSTALL_DIR"
fi
if [ -f "$INSTALL_DIR" ]; then
    die "Install target is a regular file, refusing: $INSTALL_DIR"
fi
if [ ! -d "$INSTALL_DIR" ]; then
    print_info "Creating install directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi
INSTALL_DIR="$(cd "$INSTALL_DIR" && pwd -P)"

RELEASES_DIR="${INSTALL_DIR}/releases"
CURRENT_LINK="${INSTALL_DIR}/current"
mkdir -p "$RELEASES_DIR"

print_info "Install directory: ${INSTALL_DIR}"
print_info "Config file:       ${CONFIG_PATH}"
print_info "Working directory: ${WORKING_DIR}"
print_info "Repository:        ${REPOSITORY}"
print_info "Service:           ${SERVICE_NAME}.service"

if [ ! -f "$CONFIG_PATH" ]; then
    die "Config file not found: $CONFIG_PATH"
fi

# Record the previous current target so we can roll back on failure.
PREV_TARGET=""
if [ -L "$CURRENT_LINK" ]; then
    PREV_TARGET="$(readlink "$CURRENT_LINK")"
    print_info "Previous current → ${PREV_TARGET}"
fi

# ───────────────────────── query GitHub Release ─────────────────────────
print_step "Querying GitHub Release"

API_HOST="api.github.com"
AUTH_HEADER=()
if [ -n "${GITHUB_TOKEN:-}" ]; then
    AUTH_HEADER=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
    print_info "Using GITHUB_TOKEN for authentication"
else
    print_info "No GITHUB_TOKEN set (public repository assumed)"
fi

if [ "$LATEST" -eq 1 ]; then
    RELEASE_URL="https://${API_HOST}/repos/${REPOSITORY}/releases/latest"
else
    RELEASE_URL="https://${API_HOST}/repos/${REPOSITORY}/releases/tags/${TAG}"
fi
print_info "GET ${RELEASE_URL}"

RELEASE_JSON="$(curl -fsSL "${AUTH_HEADER[@]}" \
    -H "Accept: application/vnd.github+json" \
    "$RELEASE_URL")" || die "Failed to query release. Check the tag and repository." 2

RELEASE_TAG="$(printf '%s' "$RELEASE_JSON" | jq -r '.tag_name')"
if [ -z "$RELEASE_TAG" ] || [ "$RELEASE_TAG" = "null" ]; then
    die "Release response did not contain a tag_name." 2
fi
print_info "Resolved release tag: ${RELEASE_TAG}"

# Pick the matching asset.
ASSET_URL="$(printf '%s' "$RELEASE_JSON" | jq -r --arg name "$ASSET_NAME" \
    '.assets[] | select(.name == $name) | .url' | head -n1)"
if [ -z "$ASSET_URL" ] || [ "$ASSET_URL" = "null" ]; then
    die "Asset '${ASSET_NAME}' not found in release ${RELEASE_TAG}." 2
fi
# Also look for an optional .sha256 sidecar (currently not published by CI).
SHA_ASSET_URL="$(printf '%s' "$RELEASE_JSON" | jq -r --arg name "${ASSET_NAME}.sha256" \
    '.assets[] | select(.name == $name) | .url' | head -n1)"
[ "$SHA_ASSET_URL" = "null" ] && SHA_ASSET_URL=""

# ───────────────────────── download ─────────────────────────
print_step "Downloading binary"

WORK_DIR="$(mktemp -d -t actrix-release-pull.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT

BINARY_DL="${WORK_DIR}/${ASSET_NAME}"
print_info "Downloading ${ASSET_NAME} → ${BINARY_DL}"
curl -fsSL "${AUTH_HEADER[@]}" \
    -H "Accept: application/octet-stream" \
    -L -o "$BINARY_DL" "$ASSET_URL" || die "Download failed." 2

# ───────────────────────── verify ─────────────────────────
print_step "Verifying binary"

# SHA-256 verification: the Release Train does not yet publish .sha256 files
# for Actrix. When a sidecar exists we verify; when it does not we warn and
# continue (per current manual-deployment workflow). Once CI publishes
# .sha256 this same path enforces it automatically.
if [ -n "$SHA_ASSET_URL" ]; then
    SHA_DL="${WORK_DIR}/${ASSET_NAME}.sha256"
    curl -fsSL "${AUTH_HEADER[@]}" \
        -H "Accept: application/octet-stream" \
        -L -o "$SHA_DL" "$SHA_ASSET_URL" || die "Failed to download .sha256 sidecar." 2
    EXPECTED="$(awk '{print $1}' "$SHA_DL")"
    ACTUAL="$(sha256sum "$BINARY_DL" | awk '{print $1}')"
    if [ -z "$EXPECTED" ]; then
        die "Downloaded .sha256 sidecar is empty." 2
    fi
    if [ "$EXPECTED" != "$ACTUAL" ]; then
        die "SHA-256 mismatch.
  expected: ${EXPECTED}
  actual:   ${ACTUAL}" 2
    fi
    print_info "SHA-256 verified: ${ACTUAL}"
else
    print_warn "No ${ASSET_NAME}.sha256 sidecar in this release — SHA-256 verification SKIPPED."
    print_warn "Enable CI .sha256 publishing to enforce integrity automatically."
fi

chmod +x "$BINARY_DL"

# Sanity: the binary must report a version. The clap version string is static
# ("0.1.0") and does NOT track the release tag, so we only require that
# --version exits successfully rather than that it matches the tag.
BIN_VERSION="$("$BINARY_DL" --version 2>/dev/null | head -n1 || true)"
if [ -z "$BIN_VERSION" ]; then
    die "Binary failed to report a version via --version. Refusing to install." 2
fi
print_info "Binary version: ${BIN_VERSION} (release tag: ${RELEASE_TAG})"

# ───────────────────────── install into releases/<tag> ─────────────────────────
print_step "Installing release ${RELEASE_TAG}"

TARGET_DIR="${RELEASES_DIR}/${RELEASE_TAG}"
mkdir -p "$TARGET_DIR"

# Idempotent re-publish: replace the binary atomically, never leave a
# half-written file behind.
TMP_BIN="${TARGET_DIR}/.actrix.new"
cp "$BINARY_DL" "$TMP_BIN"
chmod +x "$TMP_BIN"
mv -f "$TMP_BIN" "${TARGET_DIR}/actrix"

print_info "Installed: ${TARGET_DIR}/actrix"

# ───────────────────────── atomic current switch ─────────────────────────
print_step "Switching current symlink"

NEW_TARGET="${TARGET_DIR}"
TMP_LINK="${INSTALL_DIR}/.current.tmp.$$"
ln -sfn "$NEW_TARGET" "$TMP_LINK"
mv -Tf "$TMP_LINK" "$CURRENT_LINK"
print_info "current → ${NEW_TARGET}"

# ───────────────────────── install systemd unit ─────────────────────────
print_step "Installing systemd unit ${SERVICE_NAME}.service"

UNIT_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
UNIT_TMP="$(mktemp)"
cat > "$UNIT_TMP" <<EOF
[Unit]
Description=Actrix managed service (release ${RELEASE_TAG})
After=network.target

[Service]
Type=simple
WorkingDirectory=${WORKING_DIR}
ExecStart=${CURRENT_LINK}/actrix --config ${CONFIG_PATH}
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=${SERVICE_NAME}
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
install -m 0644 "$UNIT_TMP" "$UNIT_FILE"
rm -f "$UNIT_TMP"
systemctl daemon-reload
print_info "Unit written: ${UNIT_FILE}"

# ───────────────────────── restart / health check ─────────────────────────
if [ "$NO_RESTART" -eq 1 ]; then
    print_warn "--no-restart: version and service file prepared, not restarting."
    print_info "Start manually with:  systemctl start ${SERVICE_NAME}"
    exit 0
fi

print_step "Restarting ${SERVICE_NAME}.service"

restart_service() {
    systemctl restart "$SERVICE_NAME"
}

wait_active() {
    local waited=0
    while [ "$waited" -lt "$HEALTH_WAIT" ]; do
        if systemctl is-active --quiet "$SERVICE_NAME"; then
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done
    systemctl is-active --quiet "$SERVICE_NAME"
}

rollback() {
    print_error "Service did not become active. Rolling back current symlink."
    if [ -n "$PREV_TARGET" ]; then
        local rb_link="${INSTALL_DIR}/.current.rollback.$$"
        ln -sfn "$PREV_TARGET" "$rb_link"
        mv -Tf "$rb_link" "$CURRENT_LINK"
        print_warn "current → ${PREV_TARGET} (previous version)"
        systemctl restart "$SERVICE_NAME" || true
        systemctl status "$SERVICE_NAME" --no-pager || true
    else
        print_error "No previous version to roll back to. current left at ${NEW_TARGET}."
        systemctl status "$SERVICE_NAME" --no-pager || true
    fi
}

if ! restart_service; then
    rollback
    die "Failed to restart ${SERVICE_NAME}.service." 3
fi

if ! wait_active; then
    rollback
    die "${SERVICE_NAME}.service did not become active within ${HEALTH_WAIT}s." 3
fi

print_info "${SERVICE_NAME}.service is active."
systemctl --no-pager --lines=5 status "$SERVICE_NAME" || true

print_step "Done. current → ${NEW_TARGET}  (release ${RELEASE_TAG})"
