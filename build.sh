#!/bin/bash
echo "Building core ..."
cargo build -p trade-core --release

echo "Building protocol library ..."
cargo build -p trade-protocol --release

echo "Building node ..."
cargo build -p trade-node --release

echo "Building host ..."
cargo build -p trade-host --release

echo "Building web-frontend ..."
yarn --cwd ./trade-web
yarn --cwd ./trade-web build

echo "Building containers ..."
docker build . -f ./docker/host.Dockerfile -t trade-host
docker build . -f ./docker/node.Dockerfile -t trade-node