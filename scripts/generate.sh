#!/bin/sh

canisters=(
    "child"
    "parent"
)

echo -e "${GREEN}> $ENV: Generating required files..${NC}"
cargo test --test generate -q

for t in ${canisters[@]}; do
    echo -e "${GREEN} $ENV > Building $t..${NC}"
    dfx build --network ic $t

    mkdir -p wasm
    cp -r target/wasm32-unknown-unknown/release/$t.wasm wasm/$t.wasm
    gzip -c wasm/$t.wasm > wasm/$t.wasm.gz

    mkdir -p frontend/$t
    cp -r .dfx/ic/canisters/$t/$t.did.d.ts frontend/$t
    cp -r .dfx/ic/canisters/$t/$t.did.js frontend/$t
    cp -r .dfx/ic/canisters/$t/index.js frontend/$t
done

echo -e "${GREEN} $ENV > Stopping local replica..${NC}"