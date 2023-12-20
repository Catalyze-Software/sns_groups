#!/bin/sh

canisters=(
    "child"
    "parent"
)

echo -e "${GREEN}> $ENV: Generating required files..${NC}"
dfx generate --network ic

for t in ${canisters[@]}; do
    
    echo -e "${GREEN} $ENV > Generating candid for $t..${NC}"
    cargo test candid -p $t
    
    echo -e "${GREEN} $ENV > Building $t..${NC}"
    dfx build --network ic $t

    mkdir -p wasm
    cp -r target/wasm32-unknown-unknown/release/$t.wasm wasm/$t.wasm
    gzip -c wasm/$t.wasm > wasm/$t.wasm.gz

done

rm -rf src/declarations
echo -e "${GREEN} $ENV > Stopping local replica..${NC}"
