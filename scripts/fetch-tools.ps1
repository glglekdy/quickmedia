# yt-dlp.exe 와 ffmpeg.exe 를 받는다. 개발·빌드에서 함께 쓴다. 이미 있으면 건너뛴다.
#   ./scripts/fetch-tools.ps1 [-Dest build/tools]
param([string]$Dest = "build/tools")

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
New-Item -ItemType Directory -Force $Dest | Out-Null

$ytdlp = Join-Path $Dest "yt-dlp.exe"
if (-not (Test-Path $ytdlp)) {
    Invoke-WebRequest "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" -OutFile $ytdlp
}

$ffmpeg = Join-Path $Dest "ffmpeg.exe"
if (-not (Test-Path $ffmpeg)) {
    $zip = Join-Path $Dest "ffmpeg.zip"
    $dir = Join-Path $Dest "ffmpeg-zip"
    Invoke-WebRequest "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -OutFile $zip
    Expand-Archive $zip $dir -Force
    $exe = (Get-ChildItem $dir -Recurse -Filter ffmpeg.exe | Select-Object -First 1).FullName
    Move-Item $exe $ffmpeg -Force
    Remove-Item $zip, $dir -Recurse -Force
}

Write-Host "yt-dlp: $ytdlp"
Write-Host "ffmpeg: $ffmpeg"
