#!/bin/sh

canisters=(
    "child"
    "parent"
)

echo -e "${GREEN}> $ENV: Generating required files..${NC}"
cargo test --test generate
dfx generate --network ic

for t in ${canisters[@]}; do
    echo -e "${GREEN} $ENV > Building $t..${NC}"
    dfx build --network ic $t

    mkdir -p wasm
    cp -r target/wasm32-unknown-unknown/release/$t.wasm wasm/$t.wasm
    gzip -c wasm/$t.wasm > wasm/$t.wasm.gz

    mkdir -p frontend/$t
    cp -a src/declarations/$t frontend
done

rm -rf src/declarations
echo -e "${GREEN} $ENV > Stopping local replica..${NC}"
