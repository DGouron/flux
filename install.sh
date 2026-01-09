#!/bin/bash
set -e

REPO="DGouron/flux"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

get_latest_version() {
    local response=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")

    if command -v jq &> /dev/null; then
        echo "$response" | jq -r '.tag_name'
    else
        echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4
    fi
}

get_architecture() {
    local arch=$(uname -m)
    case $arch in
        x86_64)  echo "x86_64" ;;
        aarch64) echo "aarch64" ;;
        arm64)   echo "aarch64" ;;
        *)       echo "unsupported" ;;
    esac
}

get_os() {
    local os=$(uname -s)
    case $os in
        Linux)  echo "unknown-linux-gnu" ;;
        Darwin) echo "apple-darwin" ;;
        *)      echo "unsupported" ;;
    esac
}

main() {
    echo "🚀 Installation de Flux..."

    local version=$(get_latest_version)
    if [ -z "$version" ]; then
        echo "❌ Impossible de récupérer la dernière version"
        exit 1
    fi
    echo "   Version: $version"

    local arch=$(get_architecture)
    local os=$(get_os)

    if [ "$arch" = "unsupported" ] || [ "$os" = "unsupported" ]; then
        echo "❌ Architecture ou OS non supporté: $(uname -m) / $(uname -s)"
        exit 1
    fi

    local target="${arch}-${os}"
    local filename="flux-${version}-${target}.tar.gz"
    local url="https://github.com/$REPO/releases/download/${version}/${filename}"

    echo "   Cible: $target"
    echo "   Téléchargement: $url"

    local tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    curl -sL "$url" -o "$tmpdir/$filename"

    if [ ! -s "$tmpdir/$filename" ]; then
        echo "❌ Échec du téléchargement"
        exit 1
    fi

    tar -xzf "$tmpdir/$filename" -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/flux" "$INSTALL_DIR/"
    mv "$tmpdir/flux-daemon" "$INSTALL_DIR/"
    mv "$tmpdir/flux-gui" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/flux" "$INSTALL_DIR/flux-daemon" "$INSTALL_DIR/flux-gui"

    echo ""
    echo "✅ Flux installé dans $INSTALL_DIR"
    echo ""

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "⚠️  Ajoute $INSTALL_DIR à ton PATH:"
        echo ""
        echo "   echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
        echo "   source ~/.bashrc"
        echo ""
    fi

    echo "📖 Usage:"
    echo "   flux status"
    echo "   flux start -d 25"
    echo "   flux stop"
}

main
