FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        build-essential \
        clang \
        lld \
        cmake \
        ninja-build \
        bubblewrap \
        pkg-config \
        libssl-dev \
        curl \
        git \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
