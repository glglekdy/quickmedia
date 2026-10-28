# QuickMedia 빌드 스크립트 -> dist/QuickMedia-Setup-<버전>.exe (ffmpeg 포함 설치 파일)
#
#   ./build.ps1                      PATH 의 ffmpeg 를 넣는다
#   ./build.ps1 -FFmpeg C:\x\ffmpeg.exe
#
# Inno Setup 6 이 필요하다: winget install JRSoftware.InnoSetup

param([string]$FFmpeg)

# PyInstaller 는 로그를 stderr 로 쓰므로 Stop 으로 두면 안 된다. 실패는 종료 코드로 본다.
$ErrorActionPreference = "Continue"
Set-Location $PSScriptRoot

if (-not $FFmpeg) {
    $FFmpeg = (Get-Command ffmpeg).Source
    $target = (Get-Item $FFmpeg).Target
    if ($target) { $FFmpeg = $target }
}

$iscc = (Get-Command ISCC -ErrorAction SilentlyContinue).Source
if (-not $iscc) {
    $iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
        Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $iscc) { throw "Inno Setup 6 이 필요해요: winget install JRSoftware.InnoSetup" }

$version = [regex]::Match((Get-Content ytdl/__init__.py -Raw), '__version__ = "([^"]+)"').Groups[1].Value

# PySide6 에서 안 쓰는 큰 모듈은 뺀다 (DCCON 다운로더와 같은 방식)
$excludes = @("tkinter", "PySide6.QtQml", "PySide6.QtQuick", "PySide6.QtPdf", "PySide6.QtNetwork",
              "PySide6.QtOpenGL", "PySide6.QtSvg", "PySide6.QtSql", "PySide6.QtTest", "unittest") |
    ForEach-Object { "--exclude-module", $_ }

python tools/version_info.py build/version_info.txt
if ($LASTEXITCODE -ne 0) { throw "버전 정보 생성 실패" }

# 파일명은 ASCII 로 둔다. GitHub 가 릴리즈 자산 이름에서 한글을 지워 버린다.
# onedir: 실행할 때마다 임시 폴더에 풀지 않아 빨리 켜진다. 설치 파일이 폴더째 설치한다.
python -m PyInstaller --noconfirm --clean --onedir --windowed --name QuickMedia `
    --version-file build/version_info.txt --icon assets/icon.ico --add-data "ytdl/assets;ytdl/assets" `
    --add-binary "$FFmpeg;." @excludes app.py
if ($LASTEXITCODE -ne 0) { throw "빌드 실패" }

& $iscc "/DAppVersion=$version" installer/QuickMedia.iss
if ($LASTEXITCODE -ne 0) { throw "설치 파일 만들기 실패" }

$setup = "dist/QuickMedia-Setup-$version.exe"
$mb = (Get-Item $setup).Length / 1MB
Write-Host ("완료: $setup  ({0:N0} MB)" -f $mb) -ForegroundColor Green
