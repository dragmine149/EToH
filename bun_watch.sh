#!/bin/sh
bun build --watch --outdir=./dist --sourcemap=linked --splitting ./Scripts/index.ts ./Scripts/debug.ts
# bun build --watch --outdir=./dist --sourcemap=linked --splitting ./Scripts/intdex.ts ./Scripts/debug.ts ./Scripts/Core/ui_test.ts
