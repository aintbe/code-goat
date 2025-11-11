FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt-get update && apt-get install -y \
    # code-goat
    libseccomp-dev \
    # qingdao-judger
    cmake \
    # evaluator
    golang-go
    # tests
    openjdk-17-jdk \
    python3 \

USER vscode
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN PATH="$HOME/.cargo/bin:$PATH"
