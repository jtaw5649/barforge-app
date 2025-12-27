#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"${root_dir}/scripts/generate-registry-client.sh" >/dev/null

default_api="${root_dir}/crates/barforge-registry-client/src/apis/default_api.rs"
if rg -n "=>\\s*return\\s+" "${default_api}" >/dev/null; then
  echo "Found needless return statements in ${default_api}"
  exit 1
fi

models_dir="${root_dir}/crates/barforge-registry-client/src/models"
if rg -n "^\\s*///\\s*$" "${models_dir}" >/dev/null; then
  echo "Found empty doc comments in ${models_dir}"
  exit 1
fi

default_files=(
  create_collection_request.rs
  update_collection_request.rs
  module_category.rs
  notification.rs
  user_role.rs
)

for file in "${default_files[@]}"; do
  path="${models_dir}/${file}"
  if ! rg -n "^\\s*#\\[default\\]" "${path}" >/dev/null; then
    echo "Missing #[default] in ${path}"
    exit 1
  fi
done

if rg -n "impl Default for (Visibility|ModuleCategory|NotificationType|UserRole)" "${models_dir}" >/dev/null; then
  echo "Found manual Default impls in ${models_dir}"
  exit 1
fi
