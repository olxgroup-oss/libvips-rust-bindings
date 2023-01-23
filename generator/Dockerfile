# (c) Copyright 2019-2023 OLX
FROM rust:1.66.1-alpine3.17

ENV VIPS_VERSION=8.13.3

RUN apk add --update -q --no-cache --repository=https://dl-cdn.alpinelinux.org/alpine/v3.17/main \
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
            poppler-dev \
            tiff-dev

RUN wget https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.gz && \
    mkdir vips && \
    tar xvzf vips-${VIPS_VERSION}.tar.gz -C vips --strip-components 1 && \
     rm -f vips-${VIPS_VERSION}.tar.gz

WORKDIR /vips

RUN ./configure --quiet --enable-debug=no --without-python && \
    make -j10 && \
    make install && \
    ldconfig /etc/ld.so.conf.d && \
    rm -rf vips

WORKDIR /

RUN rustup component add rustfmt --toolchain 1.66.1-x86_64-unknown-linux-musl
