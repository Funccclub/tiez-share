#!/usr/bin/env bash
# TieZ Share - 构建 ARM64 镜像（Linux / macOS）
#
#   ./build-image.sh            构建 arm64 镜像
#   ./build-image.sh --save     构建后导出 tiez-share-arm64.tar
#   ./build-image.sh --test     额外构建 x86_64 测试镜像，方便本机验证
set -euo pipefail

cd "$(dirname "$0")"

TAG="tiez-share:arm64"
SAVE=0
TEST=0

for arg in "$@"; do
    case "$arg" in
        --save) SAVE=1 ;;
        --test) TEST=1 ;;
        *) TAG="$arg" ;;
    esac
done

echo "==> 构建 $TAG (linux/arm64)"
# --provenance=false --sbom=false：不生成 attestation 清单，
# 避免 docker save 出的包里出现 unknown/unknown 平台条目。
docker build --provenance=false --sbom=false -t "$TAG" .

echo "==> 完成："
docker images "$TAG"

if [ "$SAVE" = "1" ]; then
    OUT="tiez-share-arm64.tar"
    echo "==> 导出镜像到 $OUT"
    docker save -o "$OUT" "$TAG"
    ls -lh "$OUT"
    echo "    传到路由器：scp $OUT root@<路由IP>:/root/"
    echo "    路由器上：  docker load -i /root/$OUT"
fi

if [ "$TEST" = "1" ]; then
    echo "==> 构建本机 x86_64 测试镜像 tiez-share:amd64"
    docker build \
        --build-arg MUSL_CROSS_IMAGE=messense/rust-musl-cross:x86_64-musl \
        --build-arg RUST_TARGET=x86_64-unknown-linux-musl \
        --build-arg RUNTIME_PLATFORM=linux/amd64 \
        -t tiez-share:amd64 .
    echo "==> 本地试跑：docker run --rm -p 12345:12345 -v tiez-data:/data tiez-share:amd64"
fi
