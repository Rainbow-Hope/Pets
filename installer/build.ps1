[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem

$installerRoot = [IO.Path]::GetFullPath($PSScriptRoot)
$repoRoot = [IO.Path]::GetFullPath((Split-Path -Parent $installerRoot))
$venvRoot = Join-Path $installerRoot '.venv-build'
$python = Join-Path $venvRoot 'Scripts\python.exe'
$pyinstaller = Join-Path $venvRoot 'Scripts\pyinstaller.exe'
$buildRoot = Join-Path $installerRoot 'build'
$distRoot = Join-Path $installerRoot 'dist'
$releaseRoot = Join-Path $installerRoot 'release'
$releaseFolder = Join-Path $releaseRoot 'Instalador-Pets-Windows'
$releaseZip = Join-Path $repoRoot 'Instalador-Pets-Windows.zip'

function Remove-InstallerPath {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith(
        $installerRoot + [IO.Path]::DirectorySeparatorChar,
        [StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Recusa ao remover caminho fora de installer: $fullPath"
    }
    if (Test-Path -LiteralPath $fullPath) {
        Remove-Item -LiteralPath $fullPath -Recurse -Force
    }
}

if (-not (Test-Path -LiteralPath $python)) {
    python -m venv $venvRoot
}

& $python -m pip install --upgrade pip
& $python -m pip install --requirement (Join-Path $installerRoot 'requirements-build.txt')

Push-Location $repoRoot
try {
    & $python -m unittest discover -s installer/tests -v
    if ($LASTEXITCODE -ne 0) {
        throw "Os testes falharam."
    }

    Remove-InstallerPath -Path $buildRoot
    Remove-InstallerPath -Path $distRoot
    Remove-InstallerPath -Path $releaseRoot

    & $pyinstaller `
        --clean `
        --noconfirm `
        --distpath $distRoot `
        --workpath $buildRoot `
        (Join-Path $installerRoot 'Instalador-Pets.spec')
    if ($LASTEXITCODE -ne 0) {
        throw "O PyInstaller falhou."
    }

    $exe = Join-Path $distRoot 'Instalador-Pets.exe'
    if (-not (Test-Path -LiteralPath $exe)) {
        throw "Executavel nao encontrado: $exe"
    }

    $selfTest = Start-Process `
        -FilePath $exe `
        -ArgumentList '--self-test' `
        -Wait `
        -PassThru `
        -WindowStyle Hidden
    if ($selfTest.ExitCode -ne 0) {
        throw "O self-test do executavel falhou com codigo $($selfTest.ExitCode)."
    }

    New-Item -ItemType Directory -Force -Path $releaseFolder | Out-Null
    Copy-Item -LiteralPath $exe -Destination (Join-Path $releaseFolder 'Instalador-Pets.exe')
    Copy-Item `
        -LiteralPath (Join-Path $installerRoot 'LEIA-ME.txt') `
        -Destination (Join-Path $releaseFolder 'LEIA-ME.txt')

    if (Test-Path -LiteralPath $releaseZip) {
        Remove-Item -LiteralPath $releaseZip -Force
    }
    [IO.Compression.ZipFile]::CreateFromDirectory(
        $releaseRoot,
        $releaseZip,
        [IO.Compression.CompressionLevel]::Optimal,
        $false
    )

    Write-Output "Release criada: $releaseZip"
} finally {
    Pop-Location
}
