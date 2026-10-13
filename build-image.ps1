# TieZ Share - 构建 ARM64 镜像（Windows）
#
#   .\build-image.ps1              构建 arm64 镜像
#   .\build-image.ps1 -Save        构建后导出 tiez-share-arm64.tar（方便 scp 到路由器）
#   .\build-image.ps1 -Test        额外构建 x86_64 测试镜像，方便在电脑上先验证
#
# ⚠ 本文件必须保存为「UTF-8 带 BOM」。Windows PowerShell 5.1 对无 BOM 的 UTF-8
#   文件会按 GBK 解析，中文提示会把字符串引号吞掉，从而报出莫名其妙的语法错误
#   （例如 "表达式或语句中包含意外的标记 }"）。用 VS Code 保存时选 UTF-8 with BOM。

param(
    [switch]$Save,
    [switch]$Test,
    [string]$Tag = "tiez-share:arm64"
)

# 注意：这里故意不设 $ErrorActionPreference = "Stop"。
# docker 会把构建进度写进 stderr，Windows PowerShell 5.1 会把它当成错误记录从而中断脚本，
# 所以统一用 $LASTEXITCODE 显式判断失败（下面每个 docker 调用后都有检查）。
Set-Location -Path $PSScriptRoot

Write-Host "==> 构建 $Tag (linux/arm64)" -ForegroundColor Cyan
# --provenance=false --sbom=false：不生成 attestation 清单。
# 否则 docker save 的包里会多出一个 unknown/unknown 平台条目，
# 路由器上较老的 docker load 可能报错或产生困惑。
docker build --provenance=false --sbom=false -t $Tag .
if ($LASTEXITCODE -ne 0) { throw "arm64 构建失败" }

Write-Host "==> 完成：" -ForegroundColor Green
docker images $Tag

if ($Save) {
    $out = Join-Path $PSScriptRoot "tiez-share-arm64.tar"
    Write-Host "==> 导出镜像到 $out" -ForegroundColor Cyan
    # 注意：不要用 docker save | gzip，PowerShell 管道会破坏二进制流
    docker save -o $out $Tag
    if ($LASTEXITCODE -ne 0) { throw "导出失败" }
    $size = [math]::Round((Get-Item $out).Length / 1MB, 1)
    Write-Host "==> 已导出（$size MB）" -ForegroundColor Green
    Write-Host "    传到路由器：scp `"$out`" root@<路由IP>:/root/"
    Write-Host "    路由器上：  docker load -i /root/tiez-share-arm64.tar"
}

if ($Test) {
    Write-Host "==> 构建本机 x86_64 测试镜像 tiez-share:amd64" -ForegroundColor Cyan
    docker build `
        --build-arg MUSL_CROSS_IMAGE=messense/rust-musl-cross:x86_64-musl `
        --build-arg RUST_TARGET=x86_64-unknown-linux-musl `
        --build-arg RUNTIME_PLATFORM=linux/amd64 `
        -t tiez-share:amd64 .
    if ($LASTEXITCODE -ne 0) { throw "amd64 构建失败" }
    Write-Host "==> 本地试跑：docker run --rm -p 12345:12345 -v tiez-data:/data tiez-share:amd64" -ForegroundColor Green
}
