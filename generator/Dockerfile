# (c) Copyright 2019-2025 OLX
FROM rust:1.89.0-alpine3.20

ENV VIPS_VERSION=8.17.2

RUN apk add --quiet --update --no-cache --repository=https://dl-cdn.alpinelinux.org/alpine/v3.20/main \
  build-base \
  pkgconfig \
  clang \
  clang-libclang \
  meson \
  ninja \
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
  && wget -q https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.xz \
  && mkdir vips \
  && tar xJf vips-${VIPS_VERSION}.tar.xz -C vips --strip-components 1 \
  && cd /vips \
  && meson setup build \
  && cd /vips/build \
  && meson compile >/dev/null \
  && meson install > /dev/null \
  && ldconfig /etc/ld.so.conf.d \
  && rm -rf vips \
  && rm -f vips-${VIPS_VERSION}.tar.xz \
  && rustup component add rustfmt
