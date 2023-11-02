#!/bin/sh

# shellcheck disable=SC2046
cc -g -Wall introspect.c -o ./introspect $(pkg-config vips --cflags --libs)
