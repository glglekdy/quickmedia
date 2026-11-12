# QuickMedia 빌드 스크립트 -> dist/QuickMedia-Setup-<버전>.exe (yt-dlp 포함 설치 파일)
#
#   ./build.ps1                                  yt-dlp 를 받아서 넣는다
#   ./build.ps1 -YtDlp C:\x\yt-dlp.exe
#   ./build.ps1 -FFmpeg C:\x\ffmpeg.exe          릴리즈에 함께 올릴 dist/ffmpeg-win64.zip 도 만든다
#   ./build.ps1 -NoInstaller                     dist/QuickMedia 폴더까지만 만든다
#
# ffmpeg 는 설치 파일에 넣지 않는다. PC 에 없으면 앱이 릴리즈의 ffmpeg-win64.zip 을 받는다.
#
# Rust(https://rustup.rs)와 Inno Setup 6 이 필요하다: winget install JRSoftware.InnoSetup

param([string]$FFmpeg, [string]$YtDlp, [switch]$NoInstaller)

# cargo 는 진행 상황을 stderr 로 쓰므로 Stop 으로 두면 안 된다. 실패는 종료 코드로 본다.
$ErrorActionPreference = "Continue"
Set-Location $PSScriptRoot

$iscc = $null
if (-not $NoInstaller) {
    $iscc = (Get-Command ISCC -ErrorAction SilentlyContinue).Source
    if (-not $iscc) {
        $iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
            Where-Object { Test-Path $_ } | Select-Object -First 1
    }
    if (-not $iscc) { throw "Inno Setup 6 이 필요해요: winget install JRSoftware.InnoSetup" }
}

# 넣을 yt-dlp. 따로 주지 않으면 scripts/fetch-tools.ps1 이 받아 둔 것을 쓴다.
if (-not $YtDlp) {
    & "$PSScriptRoot/scripts/fetch-tools.ps1"
    if (-not $?) { throw "yt-dlp 받기 실패" }
    $YtDlp = "build/tools/yt-dlp.exe"
}
foreach ($tool in @($YtDlp, $FFmpeg) | Where-Object { $_ }) {
    if (-not (Test-Path $tool)) { throw "파일이 없어요: $tool" }
}

# 버전은 Cargo.toml 한 곳에서 정한다
$version = [regex]::Match((Get-Content Cargo.toml -Raw), '(?m)^version = "([^"]+)"').Groups[1].Value
if (-not $version) { throw "Cargo.toml 에서 버전을 읽지 못했어요" }

cargo build --release
if ($LASTEXITCODE -ne 0) { throw "빌드 실패" }

# 설치 파일이 폴더째 설치한다. 옛 빌드의 파일이 섞이지 않게 비우고 시작한다.
$app = "dist/QuickMedia"
if (Test-Path $app) { Remove-Item $app -Recurse -Force }
New-Item -ItemType Directory -Force $app | Out-Null
Copy-Item "target/release/QuickMedia.exe" $app
Copy-Item $YtDlp "$app/yt-dlp.exe"

# 앱이 받아 갈 ffmpeg. ffmpeg.exe 하나만 넣어 받는 양을 줄인다.
if ($FFmpeg) {
    $zip = "dist/ffmpeg-win64.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    $stage = "build/ffmpeg-zip"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Force $stage | Out-Null
    Copy-Item $FFmpeg "$stage/ffmpeg.exe"
    Compress-Archive -Path "$stage/ffmpeg.exe" -DestinationPath $zip -CompressionLevel Optimal
    Remove-Item $stage -Recurse -Force
    Write-Host ("ffmpeg: $zip  ({0:N0} MB)" -f ((Get-Item $zip).Length / 1MB))
}

if ($NoInstaller) {
    $mb = (Get-ChildItem $app | Measure-Object Length -Sum).Sum / 1MB
    Write-Host ("완료: $app  ({0:N0} MB)" -f $mb) -ForegroundColor Green
    return
}

& $iscc "/DAppVersion=$version" installer/QuickMedia.iss
if ($LASTEXITCODE -ne 0) { throw "설치 파일 만들기 실패" }

# 파일명은 ASCII 로 둔다. GitHub 가 릴리즈 자산 이름에서 한글을 지워 버린다.
$setup = "dist/QuickMedia-Setup-$version.exe"
$mb = (Get-Item $setup).Length / 1MB
Write-Host ("완료: $setup  ({0:N0} MB)" -f $mb) -ForegroundColor Green
