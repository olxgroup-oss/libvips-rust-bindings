# (c) Copyright 2019-2020 OLX
FROM rust:1.45-alpine3.12

ENV VIPS_VERSION=8.10.0

RUN apk add --update --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/community \
            build-base \
            pkgconfig \
            openssl-dev\
            clang \
            glib-dev \
            expat-dev \
            tiff-dev \
            libjpeg-turbo-dev \
            libexif-dev \
            giflib-dev \
            librsvg-dev \
            lcms2-dev \
            libpng-dev \
            orc-dev \
            libwebp-dev \
            libheif-dev \
            libimagequant-dev \
            gobject-introspection-dev \
            fftw-dev \
            pango-dev \
            libgsf-dev 

RUN wget https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.gz

RUN mkdir vips

RUN tar xvzf vips-${VIPS_VERSION}.tar.gz -C vips --strip-components 1

WORKDIR /vips

RUN ./configure --enable-debug=no --without-python

RUN make

RUN make install

RUN ldconfig /etc/ld.so.conf.d

WORKDIR /

RUN rm -rf vips

RUN rm -f vips-${VIPS_VERSION}.tar.gz

RUN rustup component add rustfmt --toolchain 1.45.2-x86_64-unknown-linux-musl
