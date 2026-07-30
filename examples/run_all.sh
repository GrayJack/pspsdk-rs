#!/bin/bash -e

shopt -s nullglob

for dir in ./examples/**/; do
    echo "Checking $dir"

    cd "$dir"
    cargo pspbuild --release
    cd "../.."

    echo ""
done

/bin/ls -l ./target/mipsel-sony-psp/release | grep "\.prx"
echo ""
/bin/ls -l ./target/mipsel-sony-psp/release | grep "\.PBP"

shopt -u nullglob