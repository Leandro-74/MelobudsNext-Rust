param(
    [string]$Version,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path

if (-not $Version) {
    $Version = (Select-String -Path (Join-Path $Root "Cargo.toml") `
        -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
}

$Name = "melobudsnext-$Version-$Target"
$Stage = Join-Path $Root "dist/$Name"
$Zip = Join-Path $Root "dist/$Name.zip"

Push-Location $Root
try {
    cargo build --release --locked --target $Target

    if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target/$Target/release/melobudsnext.exe" $Stage
    Copy-Item "target/$Target/release/melobudsnext-gui.exe" $Stage
    Copy-Item README.md, PROTOCOLO_MelobudsNext.md, LICENSE $Stage

    if (Test-Path $Zip) { Remove-Item -Force $Zip }
    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip

    $Hash = (Get-FileHash $Zip -Algorithm SHA256).Hash.ToLowerInvariant()
    "$Hash  $Name.zip" | Set-Content -Encoding ascii "$Zip.sha256"

    Write-Host "Gerado: $Zip"
    Write-Host "SHA-256: $Hash"
} finally {
    Pop-Location
}
