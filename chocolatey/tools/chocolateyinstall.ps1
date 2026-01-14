$ErrorActionPreference = 'Stop'

$packageName = 'pxl'
$version = '0.1.0'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

# Determine architecture
$is64bit = [Environment]::Is64BitOperatingSystem
$isArm64 = $env:PROCESSOR_ARCHITECTURE -eq 'ARM64'

if ($isArm64) {
    $url = "https://github.com/pixelsrc/pixelsrc/releases/download/v$version/pxl-aarch64-pc-windows-msvc.zip"
    $checksum = 'TODO_CHECKSUM_ARM64'
} elseif ($is64bit) {
    $url = "https://github.com/pixelsrc/pixelsrc/releases/download/v$version/pxl-x86_64-pc-windows-msvc.zip"
    $checksum = 'TODO_CHECKSUM_64BIT'
} else {
    throw "32-bit Windows is not supported"
}

$packageArgs = @{
    packageName    = $packageName
    unzipLocation  = $toolsDir
    url64bit       = $url
    checksum64     = $checksum
    checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
