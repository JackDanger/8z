#!/usr/bin/env bash
# checkout_siblings.sh — Clone all sibling crates that are path-deps of 7zippy.
#
# Run from the repo root (the directory containing Cargo.toml). Each crate is
# checked out as a sibling of the working tree, i.e. one level up (../NAME).
#
# To add a new sibling when a codec lands, just append its name to SIBLINGS.

set -euo pipefail

SIBLINGS=(
  lazippy
  lazippier
  bzippy2
  pippyzippy
  jumpzippy
  jumpzippier
  lockzippy
)

REPO_ORG="${SIBLING_ORG:-JackDanger}"

for name in "${SIBLINGS[@]}"; do
  dest="../${name}"
  if [ -d "${dest}" ]; then
    echo "  [skip] ${dest} already exists"
    continue
  fi
  echo "  [clone] ${REPO_ORG}/${name} -> ${dest}"
  # actions/checkout cannot write outside the workspace, so we clone into a
  # temporary subdirectory and then move it to the sibling location.
  tmp="_sibling_${name}"
  git clone --depth=1 "https://github.com/${REPO_ORG}/${name}.git" "${tmp}"
  mkdir -p "${dest}"
  cp -r "${tmp}/." "${dest}/"
  rm -rf "${dest}/.git" "${tmp}"
done

echo "Sibling crates ready."
