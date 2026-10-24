#!/usr/bin/env bash
set -euo pipefail
current_default_sha=$(gh api "repos/${GITHUB_REPOSITORY}/git/ref/heads/master" --jq .object.sha)
if [[ "$current_default_sha" != "$RELEASE_SHA" ]]; then
  echo "::notice::Skipping outdated release ${RELEASE_SHA}; master is ${current_default_sha}"
  exit 0
fi

notes_file="${RUNNER_TEMP}/latest-release-notes.md"

cat > "${notes_file}" <<EOF
latest channel for the current master tip.

- Branch: \`master\`
- Commit: \`${RELEASE_SHA}\`
- Commit URL: ${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/commit/${RELEASE_SHA}
EOF
echo >> "${notes_file}"
uv run --script "$(dirname "$0")/release-binary-sizes.py" report "${RELEASE_ASSET_DIR}" >> "${notes_file}"

git fetch --tags origin
git tag -f latest "${RELEASE_SHA}"
git push --force origin refs/tags/latest

if gh release view latest >/dev/null 2>&1; then
  gh release edit latest \
    --title "latest" \
    --notes-file "${notes_file}" \
    --prerelease
else
  gh release create latest \
    --title "latest" \
    --notes-file "${notes_file}" \
    --prerelease \
    --latest=false \
    --target "${RELEASE_SHA}"
fi

gh release upload latest "${RELEASE_ASSET_DIR}"/* --clobber

# Keep latest assets aligned with the current build matrix and archive format.
gh release view latest --json assets \
  --jq '.assets[].name | select(startswith("kat-"))' \
  > "${RUNNER_TEMP}/latest-release-package-names.txt"
while IFS= read -r asset; do
  if [[ ! -f "${RELEASE_ASSET_DIR}/${asset}" ]]; then
    gh release delete-asset latest "${asset}" --yes
  fi
done < "${RUNNER_TEMP}/latest-release-package-names.txt"
