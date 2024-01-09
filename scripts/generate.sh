#!/bin/sh

canisters=(
    "child"
    "parent"
)

echo -e "${GREEN}> $ENV: Generating required files..${NC}"

for t in ${canisters[@]}; do
    echo -e "${GREEN} $ENV > Building $t..${NC}"
    
    cargo build --manifest-path="Cargo.toml" \
    --target wasm32-unknown-unknown \
    --release --package "$t"

    mkdir -p wasm
    cp -r target/wasm32-unknown-unknown/release/$t.wasm wasm/$t.wasm
    gzip -c wasm/$t.wasm > wasm/$t.wasm.gz

    candid-extractor "target/wasm32-unknown-unknown/release/$t.wasm" > "candid/$t.did"
done

echo -e "${GREEN} $ENV > Stopping local replica..${NC}"
