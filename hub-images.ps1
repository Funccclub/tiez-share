<#
    TieZ Share - Docker Hub 镜像管理（列出标签 / 清理旧版本 / 删除指定标签）

    用法：
        .\hub-images.ps1                     列出所有标签（默认行为）
        .\hub-images.ps1 -List               同上
        .\hub-images.ps1 -Prune -Keep 3      只保留最新 3 个版本，其余连架构标签一起删
        .\hub-images.ps1 -Prune -Keep 3 -DryRun
                                             只打印将删除什么，不动手
        .\hub-images.ps1 -Delete 0.0.9,0.0.9-arm64
                                             删除指定标签

    认证：
        - 只「列出」公开仓库时不需要认证。
        - 「删除」必须要 Access Token（不是登录密码；开了 2FA 就更是必须用 token）。
          在 https://hub.docker.com/settings/security 新建，权限选 Read & Delete。
          三种提供方式，优先级从高到低：
            1) -Token <明文>            （会进命令历史，不推荐）
            2) 环境变量 DOCKERHUB_TOKEN
            3) 不加参数，脚本会安全提示输入（不回显）——推荐
          用户名同理：-Username，或环境变量 DOCKERHUB_USER，或在下面 $DefaultUsername 填死。

    网络：
        国内直连 hub.docker.com 会超时，必须走代理。本脚本默认走 127.0.0.1:7899，
        也就是 tools\proxy-auth.mjs 起的那个认证注入代理：
            node tools\proxy-auth.mjs 7899
        注意：管理镜像只走 HTTP API，**不需要**改 Docker Desktop 的 settings-store.json、
        也不需要重启 Docker Desktop。那个折腾只是为了 docker push 本身。

    ⚠ 本文件必须保存为「UTF-8 带 BOM」。Windows PowerShell 5.1 对无 BOM 的 UTF-8
      文件会按 GBK 解析，中文提示会把字符串引号吞掉，从而报出莫名其妙的语法错误。
#>

param(
    [switch]$List,
    [switch]$Prune,
    [string[]]$Delete,

    [string]$Username,
    [string]$Repository = "tiez-share",

    [string]$Proxy = "http://127.0.0.1:7899",
    [switch]$NoProxy,

    # -Prune 保留多少个最新版本
    [int]$Keep = 3,

    [switch]$DryRun,
    [switch]$Force
)

# Docker Hub 用户名，已按本项目填好；换账号时改这里，或临时传 -Username
$DefaultUsername = "14854"

# docker/hub 的进度输出走 stderr，别让 Stop 把正常输出当错误中断
$ErrorActionPreference = "Continue"

# Windows PowerShell 5.1 默认可能还在 TLS 1.0，hub.docker.com 会直接拒
try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch { }

Set-Location -Path $PSScriptRoot

# ------------------------------------------------------------------ 参数收口
if (-not $Prune -and -not $Delete) { $List = $true }
if ($Prune -and $Delete) {
    Write-Host "[x] -Prune 和 -Delete 不能同时用" -ForegroundColor Red
    exit 1
}
if (-not $Username) { $Username = $env:DOCKERHUB_USER }
if (-not $Username) { $Username = $DefaultUsername }
if (-not $Username) {
    Write-Host "[x] 缺用户名。用 -Username 指定，或设环境变量 DOCKERHUB_USER，或填在脚本里 `$DefaultUsername" -ForegroundColor Red
    exit 1
}

$Namespace = $Username
$UseProxy  = -not $NoProxy
$ProxyUrl  = $Proxy

function Test-ProxyUp {
    param([string]$Url)
    try {
        $u = [Uri]$Url
        $c = New-Object Net.Sockets.TcpClient
        $iar = $c.BeginConnect($u.Host, $u.Port, $null, $null)
        $ok = $iar.AsyncWaitHandle.WaitOne(1500, $false)
        $c.Close()
        return $ok
    } catch { return $false }
}

if ($UseProxy -and -not (Test-ProxyUp $ProxyUrl)) {
    Write-Host "[x] 代理 $ProxyUrl 没在监听。" -ForegroundColor Red
    Write-Host "    hub.docker.com 国内直连会超时，先开代理：" -ForegroundColor Yellow
    Write-Host "      node tools\proxy-auth.mjs 7899" -ForegroundColor Yellow
    Write-Host "    确实想直连（例如已在墙外）就加 -NoProxy" -ForegroundColor Yellow
    exit 1
}

# ------------------------------------------------------------------ HTTP 小工具
function Invoke-Hub {
    param(
        [string]$Uri,
        [string]$Method = "Get",
        [string]$Token,
        [object]$Body
    )
    $headers = @{ "Accept" = "application/json" }
    if ($Token) { $headers["Authorization"] = "JWT $Token" }

    $req = @{
        Uri         = $Uri
        Method      = $Method
        Headers     = $headers
        ErrorAction = "Stop"
    }
    if ($UseProxy) { $req["Proxy"] = $ProxyUrl }
    if ($Body) {
        $req["Body"]        = ($Body | ConvertTo-Json -Compress)
        $req["ContentType"] = "application/json"
    }
    return Invoke-RestMethod @req
}

function Get-HubToken {
    param([string]$Token)
    if (-not $Token) {
        $Token = $env:DOCKERHUB_TOKEN
    }
    if (-not $Token) {
        # 非交互环境（CI / 重定向 / 管道）下 Read-Host 会永久阻塞，先拦掉
        if ([Console]::IsInputRedirected) {
            throw "需要 Access Token，但当前输入不是交互终端，无法提示输入。请设环境变量 DOCKERHUB_TOKEN，或加 -Token 参数。"
        }
        $sec = Read-Host "请输入 Docker Hub Access Token（输入不回显）" -AsSecureString
        $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($sec)
        try   { $Token = [Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr) }
        finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
    }
    if (-not $Token) { throw "没有拿到 Access Token" }

    try {
        $r = Invoke-Hub -Uri "https://hub.docker.com/v2/users/login/" `
                        -Method "Post" `
                        -Body @{ username = $Username; password = $Token }
        return $r.token
    } catch {
        $status = $null
        try { $status = [int]$_.Exception.Response.StatusCode } catch { }
        if ($status -eq 401 -or $status -eq 403) {
            throw "认证失败（HTTP $status）。确认用的是 Access Token 而不是登录密码；若账号开了 2FA，密码登录一定失败。"
        }
        throw "登录请求失败：$($_.Exception.Message)"
    }
}

function Get-HubTags {
    param([string]$Token)
    $all = New-Object System.Collections.ArrayList
    $url = "https://hub.docker.com/v2/repositories/$Namespace/$Repository/tags/?page_size=100"
    $page = 0
    while ($url) {
        $page++
        $r = Invoke-Hub -Uri $url -Token $Token
        foreach ($t in $r.results) { [void]$all.Add($t) }
        $url = $r.next
        if ($page -gt 50) { break }   # 防御：正常仓库不可能有 5000 个标签
    }
    return $all
}

# 把 tag 名拆成版本三元组；不是 semver 的（latest / dev / ...）返回 $null
function Get-Semver {
    param([string]$Tag)
    # 先剥掉架构后缀，0.2.0-arm64 -> 0.2.0
    $base = $Tag -replace '-(amd64|arm64|arm|386|ppc64le|s390x|riscv64)$', ''
    if ($base -match '^v?(\d+)\.(\d+)\.(\d+)$') {
        return [pscustomobject]@{
            Major = [int]$Matches[1]
            Minor = [int]$Matches[2]
            Patch = [int]$Matches[3]
            Base  = "$($Matches[1]).$($Matches[2]).$($Matches[3])"
        }
    }
    return $null
}

function Get-TagSize {
    param($Tag)
    $sum = 0
    foreach ($img in $Tag.images) {
        # attestation 清单（provenance/SBOM）的 architecture 是 unknown，跳过，
        # 否则体积和架构列都会被它污染
        if ($img.architecture -eq "unknown") { continue }
        if ($img.size) { $sum += [int64]$img.size }
    }
    if ($sum -eq 0 -and $Tag.full_size) { $sum = [int64]$Tag.full_size }
    return $sum
}

function Format-Size {
    param([int64]$Bytes)
    if ($Bytes -le 0) { return "-" }
    if ($Bytes -ge 1GB) { return ("{0:N2} GB" -f ($Bytes / 1GB)) }
    if ($Bytes -ge 1MB) { return ("{0:N1} MB" -f ($Bytes / 1MB)) }
    return ("{0:N0} KB" -f ($Bytes / 1KB))
}

function Get-TagArch {
    param($Tag)
    $archs = @()
    foreach ($img in $Tag.images) {
        if ($img.architecture -and $img.architecture -ne "unknown") { $archs += $img.architecture }
    }
    if ($archs.Count -eq 0) { return "-" }
    return (($archs | Sort-Object -Unique) -join ",")
}

# ------------------------------------------------------------------ 取数据
# 注意：这里**不**认证。列表对公开仓库是匿名可读的，而认证需要交互输入 token；
# 提前认证会让 -DryRun 也卡在输入提示上。凭据推迟到真正要删的时候再要。
$token = $null

Write-Host "[i] 拉取 $Namespace/$Repository 的标签列表..." -ForegroundColor DarkGray
try {
    $tags = Get-HubTags -Token $null    # 公开仓库列表不需要认证
} catch {
    $status = $null
    try { $status = [int]$_.Exception.Response.StatusCode } catch { }
    if ($status -eq 404) {
        Write-Host "[x] 仓库不存在：$Namespace/$Repository" -ForegroundColor Red
        Write-Host "    用 -Repository 指定别的仓库名，或确认用户名拼写。" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "[x] 拉取失败：$($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

if ($tags.Count -eq 0) {
    Write-Host "[i] 这个仓库还没有任何标签。" -ForegroundColor Yellow
    exit 0
}

# ------------------------------------------------------------------ 列表
if ($List) {
    $rows = foreach ($t in $tags) {
        $sv = Get-Semver $t.name
        [pscustomobject]@{
            Tag        = $t.name
            Version    = if ($sv) { $sv.Base } else { "-" }
            Arch       = Get-TagArch $t
            Size       = Format-Size (Get-TagSize $t)
            LastPushed = if ($t.last_updated) { ([datetime]$t.last_updated).ToLocalTime().ToString("yyyy-MM-dd HH:mm") } else { "-" }
            Digest     = if ($t.digest) { $t.digest.Substring(7, 12) } else { "-" }
        }
    }

    Write-Host ""
    Write-Host "$Namespace/$Repository  （共 $($tags.Count) 个标签）" -ForegroundColor Cyan
    $rows | Sort-Object { $_.LastPushed } -Descending | Format-Table -AutoSize

    # 按版本汇总，方便决定保留几个
    $versioned = $tags | ForEach-Object { Get-Semver $_.name } | Where-Object { $_ } |
                 Group-Object Base | Sort-Object { [version]$_.Name } -Descending
    if ($versioned) {
        Write-Host "可清理的版本（按新→旧）：" -ForegroundColor Cyan
        $i = 0
        foreach ($g in $versioned) {
            $i++
            $mark = if ($i -le $Keep) { "保留" } else { "可删" }
            Write-Host ("  {0,2}. {1,-10} {2,2} 个标签   [{3}]" -f $i, $g.Name, $g.Count, $mark)
        }
        Write-Host ""
        Write-Host "想清理：.\hub-images.ps1 -Prune -Keep $Keep -DryRun    （先干跑看看）" -ForegroundColor Yellow
    }
    exit 0
}

# ------------------------------------------------------------------ 决定删哪些
$toDelete = @()

if ($Delete) {
    foreach ($d in $Delete) {
        $hit = $tags | Where-Object { $_.name -eq $d }
        if (-not $hit) {
            Write-Host "[!] 标签不存在，跳过：$d" -ForegroundColor Yellow
            continue
        }
        $toDelete += $hit
    }
} elseif ($Prune) {
    $versioned = @{}
    foreach ($t in $tags) {
        $sv = Get-Semver $t.name
        if (-not $sv) { continue }
        if (-not $versioned.ContainsKey($sv.Base)) { $versioned[$sv.Base] = @() }
        $versioned[$sv.Base] += $t
    }

    $order = $versioned.Keys | Sort-Object { [version]$_ } -Descending
    if ($order.Count -le $Keep) {
        Write-Host "[i] 只有 $($order.Count) 个版本，Keep=$Keep，没什么可清理的。" -ForegroundColor Green
        exit 0
    }

    $doomed = $order | Select-Object -Skip $Keep
    Write-Host ""
    Write-Host "保留最新 $Keep 个版本：" -ForegroundColor Green
    foreach ($v in ($order | Select-Object -First $Keep)) {
        Write-Host "    [留] $v  （$($versioned[$v].Count) 个标签）" -ForegroundColor Green
    }
    Write-Host "将删除：" -ForegroundColor Yellow
    foreach ($v in $doomed) {
        foreach ($t in $versioned[$v]) {
            Write-Host "    [删] $($t.name)" -ForegroundColor Yellow
            $toDelete += $t
        }
    }
    # 非 semver 的标签（latest 等）永不动
    $kept = $tags | Where-Object { -not (Get-Semver $_.name) }
    if ($kept) {
        Write-Host "非版本标签一律保留：" -ForegroundColor DarkGray
        foreach ($t in $kept) { Write-Host "    [留] $($t.name)" -ForegroundColor DarkGray }
    }
}

if ($toDelete.Count -eq 0) {
    Write-Host "[i] 没有要删除的标签。" -ForegroundColor Green
    exit 0
}

# 体积估算：架构标签和 manifest 指向同一批 blob，按 image digest 去重更接近真实回收量
$digests = @{}
$approx = 0
foreach ($t in $toDelete) {
    foreach ($img in $t.images) {
        if ($img.digest -and -not $digests.ContainsKey($img.digest)) {
            $digests[$img.digest] = $true
            if ($img.size) { $approx += [int64]$img.size }
        }
    }
}

Write-Host ""
Write-Host "共计 $($toDelete.Count) 个标签，去重后约 $(Format-Size $approx) 可回收。" -ForegroundColor Yellow

if ($DryRun) {
    Write-Host "[i] -DryRun：什么都没删。" -ForegroundColor Cyan
    exit 0
}

# ------------------------------------------------------------------ 执行删除
# 先认证再让人确认：免得敲完 yes 才发现 token 是错的或没配
if (-not $token) {
    Write-Host "[i] 认证中（$Namespace）..." -ForegroundColor DarkGray
    try { $token = Get-HubToken -Token $null }
    catch { Write-Host "[x] $($_.Exception.Message)" -ForegroundColor Red; exit 1 }
}

if (-not $Force) {
    $ans = Read-Host "删除不可逆，确认请输入 yes"
    if ($ans -ne "yes") {
        Write-Host "[i] 已取消。" -ForegroundColor Cyan
        exit 0
    }
}

$ok = 0
$fail = 0
foreach ($t in $toDelete) {
    $uri = "https://hub.docker.com/v2/repositories/$Namespace/$Repository/tags/$($t.name)/"
    try {
        Invoke-Hub -Uri $uri -Method "Delete" -Token $token | Out-Null
        Write-Host "    [ok] $($t.name)" -ForegroundColor Green
        $ok++
    } catch {
        $status = $null
        try { $status = [int]$_.Exception.Response.StatusCode } catch { }
        Write-Host "    [x ] $($t.name)  HTTP $status $($_.Exception.Message)" -ForegroundColor Red
        $fail++
    }
}

Write-Host ""
Write-Host "完成：成功 $ok 个，失败 $fail 个。" -ForegroundColor Cyan

if ($ok -gt 0) {
    Write-Host ""
    Write-Host "验证剩余标签：" -ForegroundColor DarkGray
    Write-Host "    .\hub-images.ps1" -ForegroundColor DarkGray
}
