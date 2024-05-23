#!/usr/bin/env sh
spec=$1
openapi-generator generate -i "$spec" -g rust -o openapi
