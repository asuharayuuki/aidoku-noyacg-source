$ErrorActionPreference = "Stop"

$sourceDir = Join-Path $PSScriptRoot "src/zh.noymanga-safe"
$output = Join-Path $PSScriptRoot "dist/zh.noymanga-safe.aix"
$cargo = Join-Path $env:USERPROFILE ".cargo/bin/cargo.exe"
$aidoku = Join-Path $env:USERPROFILE ".cargo/bin/aidoku.exe"

Push-Location $sourceDir
try {
    if (Test-Path $aidoku) {
        & $aidoku package
        if ($LASTEXITCODE -ne 0) { throw "aidoku package failed" }
        New-Item -ItemType Directory -Force (Split-Path $output) | Out-Null
        Copy-Item -Force "package.aix" $output
    } else {
        & $cargo +nightly build --release
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
        $payloadDir = Join-Path $sourceDir "target/wasm32-unknown-unknown/release/Payload"
        $zipOutput = Join-Path $PSScriptRoot "dist/zh.noymanga-safe.zip"
        if ((Test-Path $payloadDir) -and $payloadDir.StartsWith($sourceDir, [System.StringComparison]::OrdinalIgnoreCase)) {
            Remove-Item -LiteralPath $payloadDir -Recurse -Force
        }
        New-Item -ItemType Directory -Force $payloadDir | Out-Null
        Copy-Item -Force "res/*" $payloadDir
        Copy-Item -Force "target/wasm32-unknown-unknown/release/aidoku_noymanga_safe.wasm" (Join-Path $payloadDir "main.wasm")
        New-Item -ItemType Directory -Force (Split-Path $output) | Out-Null
        Compress-Archive -Path $payloadDir -DestinationPath $zipOutput -Force
        Move-Item -Force $zipOutput $output
    }
} finally {
    Pop-Location
}

Write-Host "Built $output"
