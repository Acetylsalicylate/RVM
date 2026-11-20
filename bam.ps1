param(
    [Parameter(Mandatory = $false)]
    [string]$ProjectPath = ".",

    [Parameter(Mandatory = $false)]
    [ValidateSet("release", "debug")]
    [string]$Profile = "release",

    [Parameter(Mandatory = $false)]
    [string]$TargetDir = "C:\Users\Lev\AppData\Local\RVM"
)

$ErrorActionPreference = "Stop"

# Переходим в каталог проекта
$resolvedProject = Resolve-Path -Path $ProjectPath
Push-Location $resolvedProject

try {
    # Получаем имя пакета из Cargo.toml
    $cargoToml = Join-Path $resolvedProject "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        throw "Не найден Cargo.toml по пути: $cargoToml"
    }

    $packageNameLine = Get-Content $cargoToml | Where-Object { $_ -match '^\s*name\s*=\s*"([^"]+)"' } | Select-Object -First 1
    if (-not $packageNameLine) {
        throw "Не удалось определить имя пакета в Cargo.toml"
    }
    $packageName = ($packageNameLine -replace '^\s*name\s*=\s*"([^"]+)".*$', '$1').Trim()
    # В именах бинарников Cargo заменяет '-' на '_'
    $binName = $packageName -replace '-', '_'
    $exeName = "$binName.exe"

    Write-Host "Сборка проекта '$packageName' ($Profile)..." -ForegroundColor Cyan
    if ($Profile -eq "release") {
        cargo build --release
    } else {
        cargo build
    }

    if ($LASTEXITCODE -ne 0) {
        throw "Ошибка сборки cargo (код выхода: $LASTEXITCODE)"
    }

    $builtExe = Join-Path $resolvedProject "target\$Profile\$exeName"
    if (-not (Test-Path $builtExe)) {
        throw "Собранный файл не найден: $builtExe"
    }

    if (-not (Test-Path $TargetDir)) {
        New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    }

    $destinationExe = Join-Path $TargetDir $exeName
    Move-Item -Path $builtExe -Destination $destinationExe -Force

    Write-Host "Готово! Файл перемещён в: $destinationExe" -ForegroundColor Green
}
finally {
    Pop-Location
}