<#
    fix-dockerhub-manifest.ps1
    修复 14854/tiez-share 的 :latest / :0.2.0 多架构清单（外科手术式）

    背景：
        Docker Hub 上 :0.1.0 的 manifest list 完好（amd64 + arm64），
        但 2026-09-22 被某次只推 amd64 的构建覆盖了 :latest 和 :0.2.0，
        导致 arm64 路由器 docker pull :latest 报 “no matching manifest for linux/arm64”。
        本脚本把 :latest / :0.2.0 的清单重新构建为多架构，指向 :0.1.0 已有的
        amd64 / arm64 两个 digest —— 不重新编译任何东西，内容与当前源码一致。

    用法：
        .\fix-dockerhub-manifest.ps1                       # 修 :latest 和 :0.2.0
        .\fix-dockerhub-manifest.ps1 -Skip02               # 只修 :latest
        .\fix-dockerhub-manifest.ps1 -WhatIf               # 干跑：取数据、拼 JSON，不推送

    凭据：
        - 代理默认 http://192.168.31.10:7893（用户 14854 / 密码 plmnko），可 -Proxy/-ProxyUser/-ProxyPass 覆盖
        - Docker Hub 账号 14854 需要 Access Token（Read & Write）：
            \$env:DOCKERHUB_TOKEN = "dckr_pat_xxx"     # 推荐
            或参数 -HubToken；或运行时交互输入（不回显）
          token 在 https://hub.docker.com/settings/security 生成

    前置校验（脚本内自动做）：
        - :0.1.0 必须是含 amd64+arm64 的 manifest list，否则直接退出
        - :latest 如果已是多架构，跳过（幂等）
    本文件必须保存为 UTF-8 with BOM（PS 5.1 会把无 BOM 文件按 GBK 解析）。
#>

param(
    [string]$Proxy      = "http://192.168.31.10:7893",
    [string]$ProxyUser  = "14854",
    [string]$ProxyPass  = "plmnko",
    [string]$HubUser    = "14854",
    [string]$HubToken   = "",
    [switch]$Skip02,
    [switch]$WhatIf
)

$ErrorActionPreference = "Stop"
$repo = "14854/tiez-share"
$reg  = "https://registry-1.docker.io"
$auth = "https://auth.docker.io/token"

# ---------- 1. 代理参数 ----------
Write-Host "[1/6] 代理：$Proxy" -ForegroundColor Cyan
$proxyArgs = @("-x", $Proxy, "-U", "${ProxyUser}:${ProxyPass}")
function Invoke-Curl {
    param([string[]]$ArgsList, [int]$Timeout = 40)
    & curl.exe -s -m $Timeout @proxyArgs @ArgsList 2>$null
    if ($LASTEXITCODE -ne 0) { throw "curl 失败 (exit $LASTEXITCODE): $($ArgsList -join ' ')" }
}

function Get-AnonToken {
    # 匿名 pull token（验证网络通路 + GET 清单用）
    $j = Invoke-Curl @("-w", "`n%{http_code}", "${auth}?service=registry.docker.io&scope=repository:${repo}:pull")
    $lines = $j -split "`n"
    if ($lines[-1] -ne "200") { throw "匿名 pull token 失败: $($lines[-1])" }
    if ($lines[0] -notmatch '"token":"([^"]+)"') { throw "匿名 token 解析失败" }
    return $Matches[1]
}

function Get-PushToken {
    param([string]$T)
    $j = Invoke-Curl @("-u", "${HubUser}:$T", "-w", "`n%{http_code}", "${auth}?service=registry.docker.io&scope=repository:${repo}:pull,push")
    $lines = $j -split "`n"
    if ($lines[-1] -ne "200") { throw "push token 失败 (HTTP $($lines[-1])): $($lines[0])" }
    if ($lines[0] -notmatch '"token":"([^"]+)"') { throw "push token 解析失败" }
    return $Matches[1]
}

# ---------- 2. 拿匿名 token，读 :0.1.0 清单 ----------
Write-Host "[2/6] 读取 :0.1.0 多架构清单（数据源）" -ForegroundColor Cyan
$tok = Get-AnonToken
$list = Invoke-Curl @("-H", "Authorization: Bearer $tok",
    "-H", "Accept: application/vnd.docker.distribution.manifest.list.v2+json",
    "$reg/v2/$repo/manifests/0.1.0")
$listObj = $list | ConvertFrom-Json
if (-not $listObj.manifests) { throw ":0.1.0 不是 manifest list，中止" }
$children = @($listObj.manifests | Where-Object { $_.platform.architecture -in @("amd64","arm64") })
$archNames = ($children | ForEach-Object { "$($_.platform.os)/$($_.platform.architecture)" }) -join " + "
Write-Host "    :0.1.0 平台: $archNames" -ForegroundColor Green
if ($children.Count -ne 2) { throw ":0.1.0 缺 amd64 或 arm64 子清单，中止" }

# ---------- 3. 看 :latest 现状 ----------
Write-Host "[3/6] :latest 现状" -ForegroundColor Cyan
$cur = Invoke-Curl @("-H", "Authorization: Bearer $tok",
    "-H", "Accept: application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.v2+json",
    "$reg/v2/$repo/manifests/latest")
$curArch = ""
if ($cur -match '"platform":\s*\{[^}]*"architecture":\s*"([^"]+)"') {
    $curArch = (([regex]::Matches($cur, '"architecture":\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }) -join " | ")
}
Write-Host "    :latest 现有架构: $curArch"
if ($curArch -match "arm64") {
    Write-Host "    :latest 已含 arm64，无需修复（幂等退出）" -ForegroundColor Green
    exit 0
}

# ---------- 4. 拼新的 manifest list ----------
Write-Host "[4/6] 构建新 manifest list（复用 :0.1.0 的两个子清单）" -ForegroundColor Cyan
$newManifests = @($children | ForEach-Object {
    [ordered]@{
        mediaType = $_.mediaType
        size      = $_.size
        digest    = $_.digest
        platform  = [ordered]@{ architecture = $_.platform.architecture; os = $_.platform.os }
    }
})
$newList = [ordered]@{
    schemaVersion = 2
    mediaType     = "application/vnd.docker.distribution.manifest.list.v2+json"
    manifests     = $newManifests
}
$payload = $newList | ConvertTo-Json -Depth 5 -Compress
$payload | ForEach-Object { Write-Host "    $($_.Substring(0, [Math]::Min(180, $_.Length)))..." }

$tmp = Join-Path $env:TEMP "tiez-manifest-$([guid]::NewGuid().ToString('N')).json"
[System.IO.File]::WriteAllText($tmp, $payload, (New-Object System.Text.UTF8Encoding($false)))

if ($WhatIf) {
    Write-Host "`n[WhatIf] 以下操作将被跳过：" -ForegroundColor Yellow
    Write-Host "    PUT $reg/v2/$repo/manifests/latest" -ForegroundColor Yellow
    if (-not $Skip02) { Write-Host "    PUT $reg/v2/$repo/manifests/0.2.0 (同上内容)" -ForegroundColor Yellow }
    Write-Host "    token: 未提供 HubToken 时会在真实执行时交互输入。" -ForegroundColor Yellow
    Remove-Item $tmp -ErrorAction SilentlyContinue
    exit 0
}

# ---------- 5. Hub 凭据 & push token ----------
if (-not $HubToken) { $HubToken = $env:DOCKERHUB_TOKEN }
if (-not $HubToken) {
    Write-Host "[5/6] 请输入 Docker Hub Access Token（14854 账号，Read & Write，不会回显）" -ForegroundColor Cyan
    $sec = Read-Host -AsSecureString
    $ptr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($sec)
    $HubToken = [System.Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr)
    [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr)
}
Write-Host "[5/6] 获取 push token" -ForegroundColor Cyan
if ($HubToken -match "[^\x20-\x7e]") { throw "HubToken 含非打印字符，请用纯 ASCII 的 Access Token" }
$pushTok = Get-PushToken $HubToken

# ---------- 6. PUT ----------
function Put-Manifest {
    param([string]$Tag)
    Write-Host "    PUT :$Tag ..." -ForegroundColor Cyan
    $hdr = Invoke-Curl @("-X", "PUT",
        "-H", "Authorization: Bearer $pushTok",
        "-H", "Content-Type: application/vnd.docker.distribution.manifest.list.v2+json",
        "-H", "Accept: application/vnd.docker.distribution.manifest.list.v2+json",
        "--data-binary", "@$tmp",
        "-D", "-", "-o", "NUL", "-w", "%{http_code}",
        "$reg/v2/$repo/manifests/$Tag")
    $code = ($hdr -split "`n")[-1]
    $dig  = ($hdr | Select-String -Pattern '^Docker-Content-Digest: (.*)$').Matches.Groups[1].Value.Trim()
    if ($code -ne "201") { throw ":$Tag PUT 失败 (HTTP $code)" }
    Write-Host "    :$Tag -> HTTP $code, digest $dig" -ForegroundColor Green
}

Write-Host "[6/6] 写入清单" -ForegroundColor Cyan
Put-Manifest "latest"
if (-not $Skip02) { Put-Manifest "0.2.0" }
Remove-Item $tmp -ErrorAction SilentlyContinue

# ---------- 验证 ----------
Write-Host "" 
Write-Host "=== 验证 ===" -ForegroundColor Green
$v = Invoke-Curl @("-H", "Authorization: Bearer $(Get-AnonToken)",
    "-H", "Accept: application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.index.v1+json",
    "$reg/v2/$repo/manifests/latest")
$va = (([regex]::Matches($v, '"architecture":\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value } | Where-Object { $_ -ne "unknown" }) -join " + ")
Write-Host ":latest 现在 = $va"
if ($va -notmatch "arm64") { Write-Host "[x] 验证未过，请人工复核" -ForegroundColor Red; exit 1 }
Write-Host "完成。路由器上 docker pull 14854/tiez-share:latest 应可正常拉取 arm64。" -ForegroundColor Green