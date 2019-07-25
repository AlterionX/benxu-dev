#!/usr/bin/sh

cd wasm-script
./compile.sh
cd ../server
./build_for_deploy.sh
mv server benxu-dev-server
