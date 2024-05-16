#!/bin/bash
set -e

mkdir -p dist

cp pkg/wasm_executor.js dist/index.js
cp pkg/wasm_executor.d.ts dist/index.d.ts
cp pkg/wasm_executor_bg* dist
