# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.234.0/containers/alpine/.devcontainer/base.Dockerfile

# [Choice] Alpine version: 3.14, 3.13, 3.12, 3.11
ARG VARIANT="3.15"
FROM mcr.microsoft.com/vscode/devcontainers/base:0-alpine-${VARIANT}

RUN apk update \
    && apk add --no-cache clang lld rustup rust cargo python3 python3-dev libffi-dev openssl-dev py3-pip musl-dev g++

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y --component miri --component rls --component rust-analysis --component rust-docs

RUN python3 -m ensurepip
RUN pip3 install --no-cache --upgrade pip setuptools