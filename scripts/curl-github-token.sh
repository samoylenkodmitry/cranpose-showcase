#!/usr/bin/env bash
# Gives curl this run's GitHub token for api.github.com, and for no other host.
#
# On Windows, wgpu builds DXC through mach-dxcompiler-rs, whose build script
# asks the GitHub API with curl whether its download is an immutable release.
# Without a token the request shares the runner's address with every other
# job on it, often hits the API's rate limit, and the build stops with
# "Refusing to download a mutable GitHub release". A .netrc entry reaches
# api.github.com only, so the token goes nowhere else.
set -euo pipefail

: "${GITHUB_TOKEN:?set GITHUB_TOKEN to the token of the run}"
dir="$RUNNER_TEMP/curl"
mkdir -p "$dir"
netrc="$dir/netrc"
printf 'machine api.github.com\nlogin x-access-token\npassword %s\n' "$GITHUB_TOKEN" > "$netrc"
chmod 600 "$netrc"
if command -v cygpath >/dev/null; then
  netrc="$(cygpath -m "$netrc")"
  home="$(cygpath -w "$dir")"
else
  home="$dir"
fi
for name in .curlrc _curlrc; do
  printf 'netrc-file = "%s"\n' "$netrc" > "$dir/$name"
done
echo "CURL_HOME=$home" >> "$GITHUB_ENV"
