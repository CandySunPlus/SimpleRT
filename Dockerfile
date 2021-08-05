FROM adoptopenjdk/openjdk8:alpine

ENV PATH=/root/.cargo/bin:$PATH

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g' /etc/apk/repositories && \
      apk add rustup binutils gcc musl-dev bash && \
      rustup-init -y && \
      rustup target add aarch64-linux-android armv7-linux-androideabi && \
      cargo install cargo-ndk

