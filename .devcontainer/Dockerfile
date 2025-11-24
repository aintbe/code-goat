FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt-get update && apt-get install -y \
    # code-goat
    libseccomp-dev \
    # evaluator
    golang-go \
    # tests & misc scripts
    cmake \
    python3 python3-pip python3-venv \
    openjdk-17-jdk

USER vscode
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN PATH="$HOME/.cargo/bin:$PATH"
