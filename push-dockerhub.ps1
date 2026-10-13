<#
    把 tiez-share 推到 Docker Hub（同时产出 amd64 + arm64 多架构清单）

    用法：
        .\push-dockerhub.ps1 -Username 你的用户名
        .\push-dockerhub.ps1 -Username 你的用户名 -Version 0.2.0 -Proxy http://127.0.0.1:7899

    前置条件：
        - 本地已有 tiez-share:amd64 和 tiez-share:arm64 两个镜像
        - 能连上 Docker Hub。国内直连通常不通（DNS 被污染、直连超时），
          而镜像加速器只能加速「拉」、不能用于「推」，所以要借一个代理。
          完整做法（已实测通过）：
            1) node tools\proxy-auth.mjs 7899
               在本地开一个不要求认证的代理，转发时自动补上上游凭据
            2) 先退出 Docker Desktop，再编辑 %APPDATA%\Docker\settings-store.json，
               加入以下四项（端口要和第 1 步一致）：
                 "ProxyHttpMode": "manual",
                 "OverrideProxyHTTP": "http://127.0.0.1:7899",
                 "OverrideProxyHTTPS": "http://127.0.0.1:7899",
                 "OverrideProxyExclude": "localhost,127.0.0.1"
               只能这样绕：直接在 GUI 里填带用户名密码的代理地址会让后端启动崩溃
               （invalid http proxy in user settings: must not include credentials）。
               另外别删原有键（尤其 UseContainerdSnapshotter），
               否则镜像存储会切换、已加载的镜像会集体「消失」。
            3) 启动 Docker Desktop，再跑本脚本
          推送完请把这四项删掉并重启 Docker Desktop：
          本地注入代理一停，所有出网（含走镜像加速的拉取）都会失败。

    本文件必须保存为 UTF-8 with BOM：
        Windows PowerShell 5.1 会把无 BOM 的 UTF-8 当 GBK 读，
        中文注释里的引号会吃掉后面的字符导致解析报错。
#>
param(
    [Parameter(Mandatory = $true)][string]$Username,
    [string]$Version = "0.1.0",
    [string]$Proxy = "",
    [switch]$SkipLogin
)

# docker 的进度输出走 stderr，$ErrorActionPreference="Stop" 会把正常输出当错误中断
$ErrorActionPreference = "Continue"

if ($Proxy) {
    $env:HTTP_PROXY = $Proxy
    $env:HTTPS_PROXY = $Proxy
    Write-Host "[i] CLI 操作走代理：$Proxy"
}

$repo = "$Username/tiez-share"

Write-Host "[1/5] 检查本地镜像"
foreach ($arch in @("amd64", "arm64")) {
    & docker image inspect "tiez-share:$arch" *> $null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[x] 缺少 tiez-share:$arch，先按 README 第 3 节构建" -ForegroundColor Red
        exit 1
    }
    Write-Host "    ok  tiez-share:$arch"
}

if (-not $SkipLogin) {
    Write-Host "[2/5] 登录 Docker Hub（用户名 $Username）"
    & docker login -u $Username
    if ($LASTEXITCODE -ne 0) { Write-Host "[x] 登录失败" -ForegroundColor Red; exit 1 }
} else {
    Write-Host "[2/5] 跳过登录（-SkipLogin）"
}

Write-Host "[3/5] 推送各架构镜像"
foreach ($arch in @("amd64", "arm64")) {
    & docker tag "tiez-share:$arch" "${repo}:$Version-$arch"
    Write-Host "    push ${repo}:$Version-$arch"
    & docker push "${repo}:$Version-$arch"
    if ($LASTEXITCODE -ne 0) { Write-Host "[x] $arch 推送失败" -ForegroundColor Red; exit 1 }
}

Write-Host "[4/5] 生成多架构清单"
& docker buildx imagetools create `
    -t "${repo}:$Version" `
    -t "${repo}:latest" `
    "${repo}:$Version-amd64" `
    "${repo}:$Version-arm64"
if ($LASTEXITCODE -ne 0) { Write-Host "[x] 清单生成失败" -ForegroundColor Red; exit 1 }

Write-Host "[5/5] 验证"
& docker buildx imagetools inspect "${repo}:$Version"

Write-Host ""
Write-Host "完成。路由器上可直接：docker pull ${repo}:$Version" -ForegroundColor Green
Write-Host "提醒：GPL-3.0 要求分发镜像时同时提供对应源码，记得也把源码放出去。" -ForegroundColor Yellow
