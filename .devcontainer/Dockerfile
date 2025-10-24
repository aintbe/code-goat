FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt-get update && apt-get install -y \
    libseccomp-dev \
    cmake \
    python3 \
    openjdk-17-jdk

ENV SANDBOX_WORKSPACE /workspaces/code-goat/code-goat
ENV RUST_LOG INFO

USER vscode
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN PATH="$HOME/.cargo/bin:$PATH"

RUN LD_LIBRARY_PATH="/workspaces/code-goat/code-goat/target/release:$LD_LIBRARY_PATH"
