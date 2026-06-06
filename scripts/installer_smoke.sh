#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="${ROOT_DIR}/install.sh"
SMOKE_ROOT="${PATHOLOG_INSTALLER_SMOKE_ROOT:-$(mktemp -d)}"
KEEP_SMOKE_ROOT="${PATHOLOG_KEEP_INSTALLER_SMOKE_ROOT:-}"
FAKEBIN="${SMOKE_ROOT}/fakebin"
DIST="${SMOKE_ROOT}/dist"
ZERO_SHA256="0000000000000000000000000000000000000000000000000000000000000000"

if [[ -z "$KEEP_SMOKE_ROOT" ]]; then
    trap 'rm -rf "$SMOKE_ROOT"' EXIT
else
    printf 'Keeping installer smoke root: %s\n' "$SMOKE_ROOT"
fi

mkdir -p "$FAKEBIN" "$DIST"

write_fake_tools() {
    cat >"${FAKEBIN}/curl" <<'FAKE_CURL'
#!/usr/bin/env sh
set -eu

out=""
url=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        -o)
            shift
            out="$1"
            ;;
        -*)
            ;;
        *)
            url="$1"
            ;;
    esac
    shift
done

case "$url" in
    *api.github.com*)
        if [ "${PATHOLOG_FAKE_API_FORBIDDEN:-}" = "1" ]; then
            printf 'unexpected GitHub API request\n' >&2
            exit 22
        fi
        printf '%s\n' '{"tag_name":"v9.9.9"}'
        ;;
    *)
        if [ -z "$out" ]; then
            printf 'missing curl -o destination for %s\n' "$url" >&2
            exit 2
        fi
        file="${url##*/}"
        cp "${PATHOLOG_FAKE_DIST:?}/${file}" "$out"
        ;;
esac
FAKE_CURL

    cat >"${FAKEBIN}/uname" <<'FAKE_UNAME'
#!/usr/bin/env sh
set -eu

case "${1:-}" in
    -s)
        printf '%s\n' "${PATHOLOG_FAKE_OS:?}"
        ;;
    -m)
        printf '%s\n' "${PATHOLOG_FAKE_ARCH:?}"
        ;;
    *)
        /usr/bin/uname "$@"
        ;;
esac
FAKE_UNAME

    cat >"${FAKEBIN}/ldd" <<'FAKE_LDD'
#!/usr/bin/env sh
set -eu

case "${PATHOLOG_FAKE_LIBC:-gnu}" in
    musl)
        printf 'musl libc\n'
        ;;
    *)
        printf 'ldd (GNU libc) 2.39\n'
        ;;
esac
FAKE_LDD

    chmod 755 "${FAKEBIN}/curl" "${FAKEBIN}/uname" "${FAKEBIN}/ldd"
}

write_checksum() {
    local archive="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$DIST" && sha256sum "$archive" >"${archive}.sha256")
    else
        (cd "$DIST" && shasum -a 256 "$archive" >"${archive}.sha256")
    fi
}

make_package() {
    local package="$1"
    local archive="$2"
    local include_completions="${3:-1}"
    local package_dir="${DIST}/${package}"

    rm -rf "$package_dir"
    mkdir -p "$package_dir"

    cat >"${package_dir}/patholog" <<'FAKE_PATHOLOG'
#!/usr/bin/env sh
set -eu

case "${1:-}" in
    --version)
        printf 'patholog 9.9.9\n'
        ;;
    completions)
        case "${2:-}" in
            bash | zsh | fish)
                printf 'generated %s completion\n' "$2"
                ;;
            *)
                printf 'unsupported shell: %s\n' "${2:-}" >&2
                exit 2
                ;;
        esac
        ;;
    *)
        printf 'fake patholog\n'
        ;;
esac
FAKE_PATHOLOG
    chmod 755 "${package_dir}/patholog"

    if [[ "$include_completions" == "1" ]]; then
        mkdir -p "${package_dir}/completions"
        printf 'packaged bash completion\n' >"${package_dir}/completions/patholog.bash"
        printf 'packaged zsh completion\n' >"${package_dir}/completions/_patholog"
        printf 'packaged fish completion\n' >"${package_dir}/completions/patholog.fish"
    fi

    printf 'readme\n' >"${package_dir}/README.md"
    printf 'license\n' >"${package_dir}/LICENSE"
    printf 'third-party licenses\n' >"${package_dir}/THIRD-PARTY-LICENSES.md"

    rm -f "${DIST}/${archive}" "${DIST}/${archive}.sha256"
    case "$archive" in
        *.tar.gz)
            (cd "$DIST" && tar -czf "$archive" "$package")
            ;;
        *.zip)
            (cd "$DIST" && zip -qr "$archive" "$package")
            ;;
        *)
            printf 'unsupported fake archive: %s\n' "$archive" >&2
            exit 2
            ;;
    esac

    write_checksum "$archive"
}

assert_completion_files() {
    local home_dir="$1"
    local expected_source="$2"
    local bash_completion="${home_dir}/.local/share/bash-completion/completions/patholog"
    local zsh_completion="${home_dir}/.zfunc/_patholog"
    local fish_completion="${home_dir}/.config/fish/completions/patholog.fish"

    case "$expected_source" in
        packaged)
            grep -Fqx 'packaged bash completion' "$bash_completion"
            grep -Fqx 'packaged zsh completion' "$zsh_completion"
            grep -Fqx 'packaged fish completion' "$fish_completion"
            ;;
        generated)
            grep -Fqx 'generated bash completion' "$bash_completion"
            grep -Fqx 'generated zsh completion' "$zsh_completion"
            grep -Fqx 'generated fish completion' "$fish_completion"
            ;;
        none)
            [[ ! -e "$bash_completion" ]]
            [[ ! -e "$zsh_completion" ]]
            [[ ! -e "$fish_completion" ]]
            ;;
        *)
            printf 'unsupported expected completion source: %s\n' "$expected_source" >&2
            exit 2
            ;;
    esac
}

run_install() {
    local name="$1"
    local os="$2"
    local arch="$3"
    local libc="$4"
    local expected_target="$5"
    local forced_linux_libc="${6:-}"
    local expected_completions="${7:-packaged}"
    local install_completions="${8:-}"
    local home_kind="${9:-dir}"
    local install_dir="${SMOKE_ROOT}/install/${name}"
    local home_dir="${SMOKE_ROOT}/home/${name}"
    local output="${SMOKE_ROOT}/${name}.out"

    rm -rf "$install_dir" "$home_dir"
    mkdir -p "$install_dir"
    case "$home_kind" in
        dir)
            mkdir -p "$home_dir"
            ;;
        file)
            mkdir -p "$(dirname "$home_dir")"
            printf 'not a directory\n' >"$home_dir"
            ;;
        *)
            printf 'unsupported home kind: %s\n' "$home_kind" >&2
            exit 2
            ;;
    esac

    if ! env \
        PATH="${FAKEBIN}:${PATH}" \
        HOME="$home_dir" \
        PATHOLOG_FAKE_DIST="$DIST" \
        PATHOLOG_FAKE_OS="$os" \
        PATHOLOG_FAKE_ARCH="$arch" \
        PATHOLOG_FAKE_LIBC="$libc" \
        PATHOLOG_LINUX_LIBC="$forced_linux_libc" \
        PATHOLOG_INSTALL_COMPLETIONS="$install_completions" \
        PATHOLOG_INSTALL_DIR="$install_dir" \
        sh "$INSTALLER" >"$output" 2>&1; then
        cat "$output" >&2
        exit 1
    fi

    "${install_dir}/patholog" --version | grep -qx 'patholog 9.9.9'
    grep -Fqx "[INFO] Target: ${expected_target}" "$output"
    if [[ "$expected_completions" == "write-failure" ]]; then
        grep -Fq "[WARN] Failed to create completion directory:" "$output"
        assert_completion_files "$home_dir" "none"
    else
        assert_completion_files "$home_dir" "$expected_completions"
    fi
    printf 'ok: %s\n' "$name"
}

expect_install_failure() {
    local name="$1"
    local os="$2"
    local arch="$3"
    local libc="$4"
    local expected_error="$5"
    local home_dir="${SMOKE_ROOT}/home/${name}"
    local output="${SMOKE_ROOT}/${name}.out"

    rm -rf "$home_dir"
    mkdir -p "$home_dir"

    if env \
        PATH="${FAKEBIN}:${PATH}" \
        HOME="$home_dir" \
        PATHOLOG_FAKE_DIST="$DIST" \
        PATHOLOG_FAKE_OS="$os" \
        PATHOLOG_FAKE_ARCH="$arch" \
        PATHOLOG_FAKE_LIBC="$libc" \
        PATHOLOG_INSTALL_DIR="${SMOKE_ROOT}/install/${name}" \
        sh "$INSTALLER" >"$output" 2>&1; then
        cat "$output" >&2
        printf 'expected installer failure for %s\n' "$name" >&2
        exit 1
    fi

    grep -Fq "$expected_error" "$output"
    printf 'ok: %s\n' "$name"
}

main() {
    write_fake_tools

    make_package \
        "patholog-9.9.9-x86_64-unknown-linux-gnu" \
        "patholog-9.9.9-x86_64-unknown-linux-gnu.tar.gz"
    make_package \
        "patholog-9.9.9-x86_64-unknown-linux-musl" \
        "patholog-9.9.9-x86_64-unknown-linux-musl.tar.gz" \
        "0"
    make_package \
        "patholog-9.9.9-aarch64-unknown-linux-musl" \
        "patholog-9.9.9-aarch64-unknown-linux-musl.tar.gz"
    make_package \
        "patholog-9.9.9-universal-apple-darwin" \
        "patholog-9.9.9-universal-apple-darwin.zip"

    run_install "linux-gnu-x86_64" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-gnu"
    run_install "linux-musl-aarch64" \
        "Linux" "aarch64" "musl" "aarch64-unknown-linux-musl"
    run_install "linux-musl-override" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-musl" "musl" "generated"
    run_install "macos-universal" \
        "Darwin" "arm64" "gnu" "universal-apple-darwin"
    run_install "linux-no-completions" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-gnu" "" "none" "0"
    run_install "linux-completion-write-failure" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-gnu" "" "write-failure" "" "file"

    printf '%s  %s\n' \
        "$ZERO_SHA256" \
        "patholog-9.9.9-x86_64-unknown-linux-gnu.tar.gz" \
        >"${DIST}/patholog-9.9.9-x86_64-unknown-linux-gnu.tar.gz.sha256"
    expect_install_failure "bad-checksum" \
        "Linux" "x86_64" "gnu" "Checksum verification failed"
    expect_install_failure "unsupported-arch" \
        "Linux" "s390x" "gnu" "Unsupported architecture"
    expect_install_failure "unsupported-os" \
        "FreeBSD" "x86_64" "gnu" "Unsupported operating system"
}

main "$@"
