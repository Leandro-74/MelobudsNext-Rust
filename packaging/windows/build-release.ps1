param(
    [string]$Version,
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$WithCli
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path

if (-not $Version) {
    $Version = (Select-String -Path (Join-Path $Root "Cargo.toml") `
        -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
}

$Dist = Join-Path $Root "dist"
$Exe = Join-Path $Dist "melobudsnext-gui.exe"

Push-Location $Root
try {
    # CRT estático: o .exe não depende do Visual C++ Redistributable.
    $env:RUSTFLAGS = ("$env:RUSTFLAGS -C target-feature=+crt-static").Trim()

    cargo build --release --locked --target $Target --bin melobudsnext-gui
    if ($WithCli) {
        cargo build --release --locked --target $Target --bin melobudsnext
    }

    New-Item -ItemType Directory -Force -Path $Dist | Out-Null
    Copy-Item "target/$Target/release/melobudsnext-gui.exe" $Exe -Force

    $Hash = (Get-FileHash $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
    "$Hash  melobudsnext-gui.exe" | Set-Content -Encoding ascii "$Exe.sha256"

    Write-Host "Gerado: $Exe"
    Write-Host "SHA-256: $Hash"

    if ($WithCli) {
        $Name = "melobudsnext-$Version-$Target"
        $Stage = Join-Path $Dist $Name
        $Zip = Join-Path $Dist "$Name.zip"

        if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
        New-Item -ItemType Directory -Force -Path $Stage | Out-Null
        Copy-Item $Exe $Stage
        Copy-Item "target/$Target/release/melobudsnext.exe" $Stage
        Copy-Item README.md, PROTOCOLO_MelobudsNext.md, LICENSE $Stage

        if (Test-Path $Zip) { Remove-Item -Force $Zip }
        Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip

        $ZipHash = (Get-FileHash $Zip -Algorithm SHA256).Hash.ToLowerInvariant()
        "$ZipHash  $Name.zip" | Set-Content -Encoding ascii "$Zip.sha256"

        Write-Host "Gerado: $Zip"
        Write-Host "SHA-256: $ZipHash"
    }
} finally {
    Pop-Location
}
