# (c) Copyright 2019-2023 OLX
FROM rust:1.71.1-alpine3.18

ENV VIPS_VERSION=8.14.4

RUN apk add --update --no-cache --repository http://dl-cdn.alpinelinux.org/alpine/edge/community \
    build-base \
    pkgconfig \
    clang \
    clang-libclang \
    cgif-dev \
    expat-dev \
    fftw-dev \
    giflib-dev \
    glib-dev \
    gobject-introspection-dev \
    lcms2-dev \
    libexif-dev \
    libgsf-dev \
    libheif-dev \
    libimagequant-dev \
    libjpeg-turbo-dev \
    libpng-dev \
    librsvg-dev \
    libwebp-dev \
    openssl-dev \
    orc-dev \
    pango-dev \
    tiff-dev \
    vips-heif \
    vips-jxl \
    vips-dev \
    vips-cpp

RUN rustup component add rustfmt
