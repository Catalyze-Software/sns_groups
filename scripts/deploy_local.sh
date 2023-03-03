#!/bin/sh

# metrics
# http://ryjl3-tyaaa-aaaaa-aaaba-cai.localhost:8080/metrics

dfx deploy canister --no-wallet --argument '("groups", principal "ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe", principal "aaaaa-aa")'
dfx canister call canister initialize_first_child_canister