#!/bin/sh

cc -g -Wall introspect.c -o ./introspect `pkg-config vips --cflags --libs`