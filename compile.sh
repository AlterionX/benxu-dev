#!/usr/bin/env bash

# Packages servers files for use.

set -euxo pipefail;

root_dir=`dirname $0`;
target_dir="$root_dir/target";
wasm_working_dir="$target_dir/wasm-working";
base_files="$root_dir/resources";
output_artifact="$root_dir/active";

shimmed-wasm-pack() {
    local filename="$1";
    local pkg_name="$2";
    # Compile slideshow.
    #
    # We're using the debug profile to avoid conflicting with the wasm profile.
    #   More specifically, using the debug profile means that no profile is passed to cargo.
    #   Using no specific profile with wasm-pack means cargo receives specifically the release profile.
    #   To avoid conflict, the we pass debug profile to wasm-pack so that wasm-pack leaves out the release.
    #
    # wasm pack assumes out-dir is from the working directory, so we use an absolute path there.
    #
    # wasm pack also assumes that the actual directory for wasm bindgen is related to the profile -- which it
    # is, but we've lied to it earlier. So we'll soft link it instead and pray that cargo doesn't recreate
    # the build directory.
    #
    # Also, out-dir is interpreted as relative to $filename (which is dumb, but moving on). So we get the
    # absolute path instead.
    rm -rf $target_dir/wasm32-unknown-unknown/debug;
    mkdir -p $target_dir/wasm32-unknown-unknown/wasm/;
    # ln always catches me off guard since the link is relative to the provided directory.
    ln -s wasm $target_dir/wasm32-unknown-unknown/debug;
    wasm-pack build \
        "$filename" \
        --debug \
        --target no-modules \
        --out-dir `readlink -f $wasm_working_dir` \
        --out-name $pkg_name \
        -- \
        --profile wasm;
    # Now delete the unholy softlink.
    rm -rf $target_dir/wasm32-unknown-unknown/debug;
}

# Clean up last working session.
rm -rf $wasm_working_dir;
rm -rf $output_artifact;

# Create active directory & copy static files.
cp -R $base_files $output_artifact;
mkdir -p $output_artifact/public/js;
mkdir -p $output_artifact/public/wasm;

# Ensure directories are in place.
mkdir -p $wasm_working_dir;
# Compile slideshow.
shimmed-wasm-pack clients/slideshow slideshow;

# Shuffle files.
mv $wasm_working_dir/*.js $output_artifact/public/js/;
mv $wasm_working_dir/*.wasm $output_artifact/public/wasm/;

# Compile static server.
cargo build --bin static-server --release;
mkdir -p $output_artifact/bin;
cp $target_dir/release/static-server $output_artifact/bin;

# Compile api server.
# TODO

set +x;

echo -e "    \e[32mComplete!\e[0m Built files can be found in the \`active\` directory.";
