[CmdletBinding()]
param(
    [switch]$SkipChecks
)

$ErrorActionPreference = 'Stop'

$ProjectRoot = [IO.Path]::GetFullPath($PSScriptRoot)
$RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $ProjectRoot '..'))
$Manifest = Join-Path $ProjectRoot 'Cargo.toml'
$OutputRoot = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot 'Executar fora do Códex'))
$ExpectedPrefix = $RepositoryRoot.TrimEnd('\') + '\'

if (-not $OutputRoot.StartsWith($ExpectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "O destino de empacotamento saiu do repositório: $OutputRoot"
}

$Cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $Cargo)) {
    throw "Cargo não encontrado em $Cargo"
}

$env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'DesktopPetsBuild'

function Initialize-MsvcEnvironment {
    if (Get-Command 'link.exe' -ErrorAction SilentlyContinue) {
        return
    }

    $VsBase = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022'
    $MsvcRoot = Get-ChildItem -LiteralPath $VsBase -Directory -ErrorAction Stop |
        ForEach-Object { Join-Path $_.FullName 'VC\Tools\MSVC' } |
        Where-Object { Test-Path -LiteralPath $_ } |
        ForEach-Object {
            Get-ChildItem -LiteralPath $_ -Directory |
                Sort-Object Name -Descending |
                Select-Object -First 1
        } |
        Select-Object -First 1

    $SdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10'
    $SdkVersion = Get-ChildItem -LiteralPath (Join-Path $SdkRoot 'Lib') -Directory |
        Where-Object {
            (Test-Path -LiteralPath (Join-Path $_.FullName 'ucrt\x64')) -and
            (Test-Path -LiteralPath (Join-Path $_.FullName 'um\x64'))
        } |
        Sort-Object Name -Descending |
        Select-Object -First 1

    if (-not $MsvcRoot -or -not $SdkVersion) {
        throw 'Ferramentas C++ MSVC ou Windows SDK não foram encontradas para gerar os executáveis.'
    }

    $env:PATH = "$(Join-Path $MsvcRoot.FullName 'bin\Hostx64\x64');$env:PATH"
    $env:LIB = @(
        (Join-Path $MsvcRoot.FullName 'lib\x64')
        (Join-Path $SdkVersion.FullName 'ucrt\x64')
        (Join-Path $SdkVersion.FullName 'um\x64')
    ) -join ';'
}

function Invoke-Cargo {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)

    & $Cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo falhou com código $LASTEXITCODE`: cargo $($Arguments -join ' ')"
    }
}

Initialize-MsvcEnvironment

Invoke-Cargo build --manifest-path $Manifest --release --bins

if (Test-Path -LiteralPath $OutputRoot) {
    $ResolvedOutput = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $OutputRoot).Path)
    if (-not $ResolvedOutput.StartsWith($ExpectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Recusa em remover destino inesperado: $ResolvedOutput"
    }
    Remove-Item -LiteralPath $ResolvedOutput -Recurse -Force
}
New-Item -ItemType Directory -Path $OutputRoot | Out-Null

$Readme = Join-Path $ProjectRoot 'package-assets\LEIA-ME.txt'
$Differences = Join-Path $ProjectRoot 'package-assets\DIFERENCAS-ENTRE-EDICOES.txt'
$RainbowSource = Join-Path $RepositoryRoot 'pets\rainbow-hope'
$NormalAtlas = Join-Path $RainbowSource 'spritesheet.webp'
$MicroAtlas = Join-Path $ProjectRoot 'package-assets\light-atlases\micro.webp'
$NanoAtlas = Join-Path $ProjectRoot 'package-assets\light-atlases\nano.webp'
$PicoAtlas = Join-Path $ProjectRoot 'package-assets\light-atlases\pico.webp'

foreach ($Required in @(
    $Readme,
    $Differences,
    (Join-Path $RainbowSource 'pet.json'),
    $NormalAtlas,
    $MicroAtlas,
    $NanoAtlas,
    $PicoAtlas
)) {
    if (-not (Test-Path -LiteralPath $Required -PathType Leaf)) {
        throw "Arquivo obrigatório ausente: $Required"
    }
}

Copy-Item -LiteralPath $Readme -Destination (Join-Path $OutputRoot 'LEIA-ME.txt')
Copy-Item -LiteralPath $Differences -Destination (Join-Path $OutputRoot 'DIFERENCAS-ENTRE-EDICOES.txt')

$Packages = @(
    @{ Relative = 'Normal';       Executable = 'DesktopPets.exe';      Atlas = $NormalAtlas },
    @{ Relative = 'Leves\Micro'; Executable = 'DesktopPetsMicro.exe'; Atlas = $MicroAtlas },
    @{ Relative = 'Leves\Nano';  Executable = 'DesktopPetsNano.exe';  Atlas = $NanoAtlas },
    @{ Relative = 'Leves\Pico';  Executable = 'DesktopPetsPico.exe';  Atlas = $PicoAtlas }
)

$DefaultConfig = @'
{
  "schemaVersion": 1,
  "startup": "none",
  "startupShortcutName": "DesktopPets.lnk",
  "instances": [
    {
      "id": "00000000-0000-0000-0000-000000000001",
      "petId": "rainbow-hope",
      "sizePercent": 100,
      "position": { "x": 80, "y": 80 },
      "movement": "fixed",
      "semiFixed": { "a": null, "b": null }
    }
  ],
  "lastActive": "00000000-0000-0000-0000-000000000001"
}
'@

foreach ($Package in $Packages) {
    $PackageRoot = Join-Path $OutputRoot $Package.Relative
    $PetRoot = Join-Path $PackageRoot 'pets\rainbow-hope'
    New-Item -ItemType Directory -Path $PetRoot -Force | Out-Null

    $BuiltExecutable = Join-Path $env:CARGO_TARGET_DIR "release\$($Package.Executable)"
    if (-not (Test-Path -LiteralPath $BuiltExecutable -PathType Leaf)) {
        throw "Executável de release ausente: $BuiltExecutable"
    }

    Copy-Item -LiteralPath $BuiltExecutable -Destination (Join-Path $PackageRoot $Package.Executable)
    Copy-Item -LiteralPath $Readme -Destination (Join-Path $PackageRoot 'LEIA-ME.txt')
    Copy-Item -LiteralPath $Differences -Destination (Join-Path $PackageRoot 'DIFERENCAS-ENTRE-EDICOES.txt')
    Copy-Item -LiteralPath (Join-Path $RainbowSource 'pet.json') -Destination (Join-Path $PetRoot 'pet.json')
    Copy-Item -LiteralPath $Package.Atlas -Destination (Join-Path $PetRoot 'spritesheet.webp')
    Set-Content -LiteralPath (Join-Path $PackageRoot 'config.json') -Value $DefaultConfig -Encoding utf8
}

if (-not $SkipChecks) {
    Invoke-Cargo fmt --manifest-path $Manifest --check
    Invoke-Cargo clippy --manifest-path $Manifest --all-targets -- -D warnings
    Invoke-Cargo test --manifest-path $Manifest
}

Write-Host "Pacotes portáteis gerados em: $OutputRoot"
