#!/usr/bin/env sh

cat scripts/foo
echo "script.sh called with $@" > scripts/log.txt
