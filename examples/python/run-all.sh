#!/usr/bin/env bash
# Start the example imposters (plain 1294, TLS 1295, mutual TLS 1296), run the
# Python example clients against them, and stop the imposters again. CI runs
# this so the examples keep working; you can run it too, from the repo root:
#
#   examples/python/run-all.sh
#
# Needs the Python package installed (maturin develop) and `python` on PATH
# resolving to that environment, openssl for the certificates, and ports
# 1294-1296 free.
set -euo pipefail

cd "$(dirname "$0")/../.."

cargo build -q -p ippdme-imposter --bin ippdme-imposter
sh examples/tls/make-certs.sh

pids=()
trap 'kill "${pids[@]}" 2>/dev/null || true' EXIT

for config in plain/imposter tls/imposter-tls tls/imposter-mtls; do
  target/debug/ippdme-imposter "examples/$config.yaml" >/dev/null &
  pids+=($!)
done

# Wait until each port accepts connections (up to ~15s).
for port in 1294 1295 1296; do
  for _ in $(seq 1 30); do
    if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then break; fi
    sleep 0.5
  done
done

python examples/python/plain_client.py
python examples/python/tls_client.py
python examples/python/tls_client.py --mtls
echo "Python examples OK"
