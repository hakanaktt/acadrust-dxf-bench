[CmdletBinding()]
param(
    [ValidateSet("win-x64")]
    [string]$Runtime = "win-x64"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$packageDir = Join-Path $root "dist\acadrust-dxf-bench"
$pyWorkDir = Join-Path $root "target\pyinstaller"

New-Item -ItemType Directory -Force -Path $packageDir | Out-Null

Push-Location $root
try {
    cargo build --release
    Copy-Item "target\release\acadrust-dxf-bench.exe" (Join-Path $packageDir "acadrust-dxf-bench.exe") -Force

    dotnet publish "acadsharp-bench\acadsharp-bench.csproj" `
        --configuration Release `
        --runtime $Runtime `
        --self-contained true `
        -p:PublishSingleFile=true `
        -p:IncludeNativeLibrariesForSelfExtract=true `
        --output $packageDir

    py -3 -m pip install --disable-pip-version-check --upgrade pyinstaller
    py -3 -m PyInstaller `
        --noconfirm `
        --clean `
        --onefile `
        --name ezdxf-bench `
        --collect-all ezdxf `
        --distpath $packageDir `
        --workpath $pyWorkDir `
        --specpath $pyWorkDir `
        "ezdxf-bench\bench.py"

    @'
# acadrust DXF/DWG benchmark

Run `acadrust-dxf-bench.exe --gui` to open the graphical benchmark application.

This package includes self-contained ACadSharp and ezdxf benchmark runners. Keep
`acadsharp-bench.exe` and `ezdxf-bench.exe` next to the main executable.
'@ | Set-Content -NoNewline -Encoding utf8 (Join-Path $packageDir "README.txt")

    Write-Host "Packaged application: $packageDir"
}
finally {
    Pop-Location
}
