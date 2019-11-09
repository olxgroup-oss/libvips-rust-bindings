#!/bin/sh

cc -g -Wall introspect.c -o ${OUT_DIR}/introspect `pkg-config vips --cflags --libs`