#!/bin/sh
# bun build --watch --outdir=./ETOH/dist --minify --sourcemap=linked --splitting ./ETOH/Scripts/index.ts ./ETOH/Scripts/debug.ts
# bun build --watch --outdir=./external ./Scripts/storage.ts
bun build --watch --outdir=./ETOH/dist --minify --sourcemap=linked --splitting ./ETOH/Scripts/ui.ts
