FROM amazonlinux:latest

WORKDIR /main
RUN yum -y groupinstall "Development Tools"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH ~/.cargo/bin:$PATH

CMD /bin/bash
