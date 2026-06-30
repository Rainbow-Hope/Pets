[CmdletBinding()]
param(
    [int]$WarmupSeconds = 6,
    [int]$SampleSeconds = 8,
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'

$ProjectRoot = [IO.Path]::GetFullPath($PSScriptRoot)
$RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $ProjectRoot '..'))
$PackagesRoot = Join-Path $RepositoryRoot 'Executar fora do Códex'
if (-not $OutputPath) {
    $OutputPath = Join-Path $ProjectRoot 'package-assets\edition-measurements.json'
}

$Editions = @(
    @{ Edition = 'Normal'; Relative = 'Normal';       Executable = 'DesktopPets.exe' },
    @{ Edition = 'Micro';  Relative = 'Leves\Micro'; Executable = 'DesktopPetsMicro.exe' },
    @{ Edition = 'Nano';   Relative = 'Leves\Nano';  Executable = 'DesktopPetsNano.exe' },
    @{ Edition = 'Pico';   Relative = 'Leves\Pico';  Executable = 'DesktopPetsPico.exe' }
)

$Results = foreach ($Item in $Editions) {
    $Package = Join-Path $PackagesRoot $Item.Relative
    $Executable = Join-Path $Package $Item.Executable
    if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
        throw "Executável ausente: $Executable"
    }

    $Process = Start-Process -FilePath $Executable -WorkingDirectory $Package -PassThru
    try {
        Start-Sleep -Seconds $WarmupSeconds
        $Process.Refresh()
        if ($Process.HasExited) {
            throw "$($Item.Edition) encerrou durante o aquecimento com código $($Process.ExitCode)."
        }
        if ($Process.MainWindowHandle -eq 0) {
            throw "$($Item.Edition) não apresentou uma janela de pet."
        }

        $CpuStart = $Process.TotalProcessorTime
        Start-Sleep -Seconds $SampleSeconds
        $Process.Refresh()
        if ($Process.HasExited) {
            throw "$($Item.Edition) encerrou durante a medição."
        }
        $CpuDelta = $Process.TotalProcessorTime - $CpuStart
        $NormalizedCpu = (
            $CpuDelta.TotalSeconds /
            $SampleSeconds /
            [Environment]::ProcessorCount *
            100
        )
        $PackageBytes = (
            Get-ChildItem -LiteralPath $Package -Recurse -File |
                Measure-Object -Property Length -Sum
        ).Sum
        $ExecutableBytes = (Get-Item -LiteralPath $Executable).Length

        [pscustomobject]@{
            edition = $Item.Edition
            executable_bytes = [int64]$ExecutableBytes
            package_bytes = [int64]$PackageBytes
            working_set_bytes = [int64]$Process.WorkingSet64
            normalized_idle_cpu_percent = [math]::Round($NormalizedCpu, 4)
            main_window_handle = [int64]$Process.MainWindowHandle
        }
    }
    finally {
        $Process.Refresh()
        if (-not $Process.HasExited) {
            Stop-Process -Id $Process.Id -Force
            $Process.WaitForExit(5000) | Out-Null
        }
    }
}

$Report = [ordered]@{
    measured_at = (Get-Date).ToString('o')
    machine = $env:COMPUTERNAME
    windows = [Environment]::OSVersion.VersionString
    logical_processors = [Environment]::ProcessorCount
    warmup_seconds = $WarmupSeconds
    sample_seconds = $SampleSeconds
    editions = @($Results)
}

$Report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
$Results | Format-Table -AutoSize
Write-Host "Relatório salvo em: $OutputPath"
