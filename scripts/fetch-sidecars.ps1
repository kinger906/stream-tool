$ErrorActionPreference = "Stop"
$BinDir = Join-Path $PSScriptRoot "..\src-tauri\binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$ffmpegUrl = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
$mediamtxUrl = "https://github.com/bluenviron/mediamtx/releases/download/v1.11.3/mediamtx_v1.11.3_windows_amd64.zip"

$tempDir = Join-Path $env:TEMP "stream-tool-sidecars"
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

Write-Host "Downloading FFmpeg..."
$ffmpegZip = Join-Path $tempDir "ffmpeg.zip"
Invoke-WebRequest -Uri $ffmpegUrl -OutFile $ffmpegZip
Expand-Archive -Path $ffmpegZip -DestinationPath (Join-Path $tempDir "ffmpeg") -Force
$ffmpegExe = Get-ChildItem -Path (Join-Path $tempDir "ffmpeg") -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
Copy-Item $ffmpegExe.FullName (Join-Path $BinDir "ffmpeg-x86_64-pc-windows-msvc.exe") -Force
Write-Host "FFmpeg -> src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe"

Write-Host "Downloading MediaMTX..."
$mtxZip = Join-Path $tempDir "mediamtx.zip"
Invoke-WebRequest -Uri $mediamtxUrl -OutFile $mtxZip
Expand-Archive -Path $mtxZip -DestinationPath (Join-Path $tempDir "mediamtx") -Force
$mtxExe = Get-ChildItem -Path (Join-Path $tempDir "mediamtx") -Recurse -Filter "mediamtx.exe" | Select-Object -First 1
Copy-Item $mtxExe.FullName (Join-Path $BinDir "mediamtx-x86_64-pc-windows-msvc.exe") -Force
Write-Host "MediaMTX -> src-tauri/binaries/mediamtx-x86_64-pc-windows-msvc.exe"

Write-Host "Done. Run: npm run tauri dev"
