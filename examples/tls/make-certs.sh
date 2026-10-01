#!/bin/sh
# Create throwaway certificates for the TLS examples, in examples/tls/certs/:
#   ca.pem / ca.key          a test CA
#   server.pem / server.key  server certificate for "localhost", signed by the CA
#   client.pem / client.key  client certificate (for mutual TLS), signed by the CA
# For local experiments only. Safe to delete the folder and re-run.
set -eu

cd "$(dirname "$0")"
mkdir -p certs
cd certs

openssl req -x509 -newkey rsa:2048 -nodes -days 30 \
  -subj "/CN=ippdme-example-ca" -keyout ca.key -out ca.pem 2>/dev/null

issue() { # issue <name> <subjectAltName>
  printf 'subjectAltName=%s\n' "$2" > "$1.ext"
  openssl req -newkey rsa:2048 -nodes -subj "/CN=$1" \
    -keyout "$1.key" -out "$1.csr" 2>/dev/null
  openssl x509 -req -in "$1.csr" -CA ca.pem -CAkey ca.key -CAcreateserial \
    -days 30 -extfile "$1.ext" -out "$1.pem" 2>/dev/null
  rm -f "$1.csr" "$1.ext"
}

issue server "DNS:localhost"
issue client "DNS:client"
rm -f ca.srl

echo "Wrote $(pwd): ca.pem server.pem server.key client.pem client.key"
