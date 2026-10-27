# 유튜브 다운로더 빌드 스크립트 -> dist/YoutubeDownloader.exe (ffmpeg 포함 단일 exe)
#
#   ./build.ps1                      PATH 의 ffmpeg 를 넣는다
#   ./build.ps1 -FFmpeg C:\x\ffmpeg.exe

param([string]$FFmpeg)

# PyInstaller 는 로그를 stderr 로 쓰므로 Stop 으로 두면 안 된다. 실패는 종료 코드로 본다.
$ErrorActionPreference = "Continue"
Set-Location $PSScriptRoot

if (-not $FFmpeg) {
    $FFmpeg = (Get-Command ffmpeg).Source
    $target = (Get-Item $FFmpeg).Target
    if ($target) { $FFmpeg = $target }
}

# PySide6 에서 안 쓰는 큰 모듈은 뺀다 (DCCON 다운로더와 같은 방식)
$excludes = @("tkinter", "PySide6.QtQml", "PySide6.QtQuick", "PySide6.QtPdf", "PySide6.QtNetwork",
              "PySide6.QtOpenGL", "PySide6.QtSvg", "PySide6.QtSql", "PySide6.QtTest", "unittest") |
    ForEach-Object { "--exclude-module", $_ }

python -m PyInstaller --noconfirm --clean --onefile --windowed --name YoutubeDownloader `
    --icon assets/icon.ico --add-data "ytdl/assets;ytdl/assets" `
    --add-binary "$FFmpeg;." @excludes app.py
if ($LASTEXITCODE -ne 0) { throw "빌드 실패" }

$mb = (Get-Item dist/YoutubeDownloader.exe).Length / 1MB
Write-Host ("완료: dist/YoutubeDownloader.exe  ({0:N0} MB)" -f $mb) -ForegroundColor Green
