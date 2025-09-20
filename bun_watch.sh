#!/bin/sh
bun build --watch --outdir=./dist --minify --sourcemap=linked --splitting ./Scripts/initial.ts ./Scripts/debug.ts
# bun build --watch --outdir=./external ./Scripts/storage.ts
# bun build --watch --outdir=./dist --minify --sourcemap=linked --splitting ./Scripts/ui.ts
