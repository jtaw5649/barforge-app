#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
spec_path="${root_dir}/../barforge-registry-api/docs/openapi.yaml"
output_dir="${root_dir}/crates/barforge-registry-client"
template_dir="${root_dir}/openapi-templates/rust"

npx @openapitools/openapi-generator-cli generate \
  -i "${spec_path}" \
  -g rust \
  -o "${output_dir}" \
  -t "${template_dir}" \
  --additional-properties=packageName=barforge_registry_client,packageVersion=0.1.0,library=reqwest,hideGenerationTimestamp=true \
  --global-property=apiTests=false,modelTests=false,apiDocs=false,modelDocs=false

rm -f \
  "${output_dir}/.travis.yml" \
  "${output_dir}/git_push.sh" \
  "${output_dir}/.gitignore"
