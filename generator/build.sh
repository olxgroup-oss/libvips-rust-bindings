#!/bin/sh

rm -rf target || true
docker build -t libvips-builder .
