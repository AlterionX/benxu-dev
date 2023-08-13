#!/usr/bin/env bash

set -euxo pipefail;


root=`dirname $0`;
static_server="bin/static-server";

cd "$root";
"$static_server";
