#!/bin/bash
set -euo pipefail

# Build an SRPM for rpm-builder using the crate tarball from the checkout
# and a vendored dependency tarball.
#
# Usage:
#   ./build-srpm.sh              # uses version from Cargo.toml
#   ./build-srpm.sh 0.4.0        # explicit version
#
# Prerequisites: cargo, rpmbuild
#
# The SRPM is written to the current directory.

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
NAME="rpm-builder"
CRATE_FILE="${NAME}-${VERSION}.crate"
VENDOR_FILE="${NAME}-${VERSION}-vendor.tar.gz"

echo "Building SRPM for ${NAME}-${VERSION}"

BUILDDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR"' EXIT

mkdir -p "$BUILDDIR"/{SOURCES,SPECS}

# Package the checked-out sources instead of downloading from crates.io. The
# release workflow publishes first, and crates.io may not serve the new crate
# immediately afterward.
echo "Packaging ${CRATE_FILE}..."
cargo package --allow-dirty --no-verify
cp "target/package/${CRATE_FILE}" "$BUILDDIR/SOURCES/${CRATE_FILE}"

# Generate vendor tarball from the downloaded crate
echo "Vendoring dependencies..."
VENDORDIR=$(mktemp -d)
trap 'rm -rf "$BUILDDIR" "$VENDORDIR"' EXIT

tar -tzf "$BUILDDIR/SOURCES/${CRATE_FILE}" >/dev/null
tar -xzf "$BUILDDIR/SOURCES/${CRATE_FILE}" -C "$VENDORDIR"
(cd "$VENDORDIR/${NAME}-${VERSION}" && cargo vendor --versioned-dirs > /dev/null)
tar -czf "$BUILDDIR/SOURCES/${VENDOR_FILE}" -C "$VENDORDIR/${NAME}-${VERSION}" vendor/

# Copy the spec file
cp rpm-builder.spec "$BUILDDIR/SPECS/"

# Build the SRPM
echo "Building SRPM..."
rpmbuild -bs \
    --define "_topdir $BUILDDIR" \
    --define "_srcrpmdir $(pwd)" \
    "$BUILDDIR/SPECS/rpm-builder.spec"

echo "Done. SRPM written to $(ls -1t "${NAME}"-*.src.rpm 2>/dev/null | head -1)"
