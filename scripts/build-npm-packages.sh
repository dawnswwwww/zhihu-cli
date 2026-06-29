#!/usr/bin/env bash
# Build and publish zhihu-cli npm packages from GitHub Release artifacts.
#
# Usage:
#   ./scripts/build-npm-packages.sh <version>
#
# Expects release artifacts to be available at:
#   ./artifacts/zhihu-<target>.tar.gz
#
# Set DRY_RUN=1 to build packages locally without publishing to npm.

set -eo pipefail

VERSION="${1:?Usage: $0 <version>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NPM_DIR="$ROOT/npm"
ARTIFACTS_DIR="$ROOT/artifacts"
SCOPE="@dawnswwwww"
DRY_RUN="${DRY_RUN:-}"

if [[ -n "$DRY_RUN" ]]; then
    echo "DRY_RUN enabled: packages will be built but not published to npm."
fi

if [[ ! -d "$ARTIFACTS_DIR" ]]; then
    echo "error: artifacts directory not found: $ARTIFACTS_DIR" >&2
    exit 1
fi

# Platform definitions. Keep these three arrays in sync.
TARGETS=(
    x86_64-apple-darwin
    aarch64-apple-darwin
    x86_64-unknown-linux-gnu
    aarch64-unknown-linux-gnu
    x86_64-pc-windows-msvc
)
PKGS=(
    ${SCOPE}/zhihu-cli-darwin-x64
    ${SCOPE}/zhihu-cli-darwin-arm64
    ${SCOPE}/zhihu-cli-linux-x64
    ${SCOPE}/zhihu-cli-linux-arm64
    ${SCOPE}/zhihu-cli-win32-x64
)

# Resolve npm os/cpu fields from scoped package name.
os_for_pkg() {
    local name="${1#${SCOPE}/zhihu-cli-}"
    case "$name" in
        darwin-*) echo darwin ;;
        linux-*)  echo linux ;;
        win32-*)  echo win32 ;;
    esac
}

cpu_for_pkg() {
    local name="${1#${SCOPE}/zhihu-cli-}"
    case "$name" in
        *-x64)   echo x64 ;;
        *-arm64) echo arm64 ;;
    esac
}

# Build and publish platform-specific subpackages.
for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    pkg="${PKGS[$i]}"
    artifact="$ARTIFACTS_DIR/zhihu-${target}.tar.gz"

    if [[ ! -f "$artifact" ]]; then
        echo "error: missing artifact: $artifact" >&2
        exit 1
    fi

    pkg_dir="$NPM_DIR/$pkg"
    rm -rf "$pkg_dir"
    mkdir -p "$pkg_dir/bin"

    echo "Packaging $pkg from $artifact ..."

    if [[ "${target}" == *windows* ]]; then
        tar -xzf "$artifact" -C "$pkg_dir/bin/"
        # artifact contains zhihu.exe
    else
        tar -xzf "$artifact" -C "$pkg_dir/bin/"
        chmod +x "$pkg_dir/bin/zhihu"
    fi

    os="$(os_for_pkg "$pkg")"
    cpu="$(cpu_for_pkg "$pkg")"

    cat > "$pkg_dir/package.json" <<EOF
{
  "name": "$pkg",
  "version": "$VERSION",
  "description": "zhihu CLI binary for ${target}",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/dawnswwwww/zhihu-cli.git"
  },
  "os": ["$os"],
  "cpu": ["$cpu"],
  "files": ["bin"]
}
EOF

    if [[ -n "$DRY_RUN" ]]; then
        echo "Would publish $pkg@$VERSION (dry run)"
    else
        (cd "$pkg_dir" && npm publish --access public)
        echo "Published $pkg@$VERSION"
    fi
done

# Build and publish the main package (thin JS shim).
main_pkg_dir="$NPM_DIR/zhihu-cli"
rm -rf "$main_pkg_dir"
mkdir -p "$main_pkg_dir/bin"

cat > "$main_pkg_dir/bin/zhihu.js" <<'EOF'
#!/usr/bin/env node
const { spawn } = require('child_process');

const platform = process.platform;
const arch = process.arch;
const packageName = `@dawnswwwww/zhihu-cli-${platform}-${arch}`;

let binaryPath;
try {
  binaryPath = require.resolve(`${packageName}/bin/zhihu${platform === 'win32' ? '.exe' : ''}`);
} catch (e) {
  console.error(`zhihu-cli: unsupported platform ${platform}-${arch}`);
  console.error('Supported platforms: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 0);
  }
});
EOF
chmod +x "$main_pkg_dir/bin/zhihu.js"

cp "$ROOT/README.md" "$main_pkg_dir/README.md"

cat > "$main_pkg_dir/package.json" <<EOF
{
  "name": "${SCOPE}/zhihu-cli",
  "version": "$VERSION",
  "description": "CLI for the Zhihu Open Platform API",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/dawnswwwww/zhihu-cli.git"
  },
  "bin": {
    "zhihu": "bin/zhihu.js"
  },
  "files": [
    "bin",
    "README.md"
  ],
  "optionalDependencies": {
    "${SCOPE}/zhihu-cli-darwin-x64": "$VERSION",
    "${SCOPE}/zhihu-cli-darwin-arm64": "$VERSION",
    "${SCOPE}/zhihu-cli-linux-x64": "$VERSION",
    "${SCOPE}/zhihu-cli-linux-arm64": "$VERSION",
    "${SCOPE}/zhihu-cli-win32-x64": "$VERSION"
  },
  "os": ["darwin", "linux", "win32"],
  "cpu": ["x64", "arm64"]
}
EOF

if [[ -n "$DRY_RUN" ]]; then
    echo "Would publish ${SCOPE}/zhihu-cli@$VERSION (dry run)"
else
    (cd "$main_pkg_dir" && npm publish --access public)
    echo "Published ${SCOPE}/zhihu-cli@$VERSION"
fi
