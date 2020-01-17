FROM ubuntu

RUN apt update

RUN apt install -y build-essential libz-dev llvm-8-dev gcc curl git

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=/root/.cargo/bin:${PATH}

