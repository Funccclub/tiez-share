# syntax=docker/dockerfile:1
#
# TieZ Share - 局域网文件中转 / 聊天室 (ARM64)
#
# 默认产出 linux/arm64 镜像（iStoreOS 软路由最常用的架构）。
# 关键点：编译阶段用 x86_64 宿主机上的 aarch64-musl 交叉工具链，
# 编译过程不做 QEMU 模拟，所以速度接近本地编译；
# 运行阶段只做文件拷贝（没有 RUN），因此也不需要 QEMU。
#
# 需要本地 x86_64 镜像做测试时（例如在 Windows/Mac 上验证功能）：
#   docker build \
#     --build-arg MUSL_CROSS_IMAGE=messense/rust-musl-cross:x86_64-musl \
#     --build-arg RUST_TARGET=x86_64-unknown-linux-musl \
#     --build-arg RUNTIME_PLATFORM=linux/amd64 \
#     -t tiez-share:amd64 .

ARG MUSL_CROSS_IMAGE=messense/rust-musl-cross:aarch64-musl
ARG RUST_TARGET=aarch64-unknown-linux-musl
ARG RUNTIME_PLATFORM=linux/arm64

# ---------------------------------------------------------------- build stage
FROM --platform=$BUILDPLATFORM ${MUSL_CROSS_IMAGE} AS builder

ARG RUST_TARGET
ARG RUN_TESTS=0
WORKDIR /app

# 先只拷贝清单（含 Cargo.lock，保证依赖版本可复现）并用桩文件编译，
# 把依赖层缓存下来；之后改业务代码时不会重新编译 axum/tokio。
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --locked --release --target ${RUST_TARGET} \
 && rm -rf src

COPY src ./src
# 注意 `touch`：BuildKit 的 COPY 会归一化文件时间戳，若源文件时间比刚才
# 依赖层里桩编译的产物还旧，cargo 会误判"无需重编"，打出一个空的桩二进制
# （现象：容器启动后立刻 exit 0，日志为空）。touch 一下强制重建。
RUN touch src/*.rs \
 && if [ "$RUN_TESTS" = 1 ]; then cargo test --locked --release --target ${RUST_TARGET}; fi \
 && cargo build --locked --release --target ${RUST_TARGET} \
 && cp target/${RUST_TARGET}/release/tiez-share /tiez-share \
 && ls -lh /tiez-share

# -------------------------------------------------------------- runtime stage
FROM --platform=${RUNTIME_PLATFORM} alpine:3.20

LABEL org.opencontainers.image.title="TieZ Share" \
      org.opencontainers.image.description="独立运行的局域网文件中转 / 聊天室服务（提取自 jimuzhe/tiez-clipboard）" \
      org.opencontainers.image.licenses="GPL-3.0" \
      org.opencontainers.image.source="https://github.com/Funccclub/tiez-share"

# 静态 musl 二进制，不依赖镜像内的 libc
COPY --from=builder /tiez-share /usr/local/bin/tiez-share

ENV TIEZ_PORT=12345 \
    TIEZ_BIND=0.0.0.0 \
    TIEZ_DATA_DIR=/data \
    TIEZ_MAX_HISTORY=1000 \
    TIEZ_THEME=mica \
    TIEZ_COLOR_MODE=system \
    TIEZ_TITLE=局域网聊天室 \
    TIEZ_JOIN_NOTICE=0

VOLUME ["/data"]
EXPOSE 12345
WORKDIR /data

# alpine 自带 busybox wget，用它做健康检查（scratch 镜像里没有可执行的探针）
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- "http://127.0.0.1:${TIEZ_PORT}/healthz" >/dev/null 2>&1 || exit 1

ENTRYPOINT ["/usr/local/bin/tiez-share"]
