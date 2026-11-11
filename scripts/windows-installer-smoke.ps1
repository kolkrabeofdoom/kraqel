[CmdletBinding()]
param(
    [string]$InstallerPath,
    [string]$TauriDriver = $env:KRAQEL_TAURI_DRIVER,
    [string]$EdgeDriver = $env:KRAQEL_EDGE_DRIVER
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$installRoot = Join-Path ([System.IO.Path]::GetTempPath()) 'kraqel-installer-smoke'
$dataRoot = Join-Path ([System.IO.Path]::GetTempPath()) 'kraqel-tauri-smoke-installer'
$uninstallRegistryPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'
$expectedDisplayName = 'KRAQEL Installer-Test'

function Remove-TestDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\')
    $resolvedTarget = [System.IO.Path]::GetFullPath($Path).TrimEnd('\')
    $allowedNames = @('kraqel-installer-smoke', 'kraqel-tauri-smoke-installer')
    if (-not $resolvedTarget.StartsWith("$resolvedTemp\", [System.StringComparison]::OrdinalIgnoreCase) -or
        $allowedNames -notcontains (Split-Path -Leaf $resolvedTarget)) {
        throw "Unsicheres temporäres Löschziel abgelehnt: $resolvedTarget"
    }
    Remove-Item -LiteralPath $resolvedTarget -Recurse -Force -ErrorAction SilentlyContinue
}

function Get-TestUninstallEntries {
    @(Get-ItemProperty $uninstallRegistryPath -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -eq $expectedDisplayName })
}

if ($env:OS -ne 'Windows_NT') {
    throw 'Der Installer-Smoke-Test ist auf Windows begrenzt.'
}

if (-not $InstallerPath) {
    $candidateDirectory = Join-Path $repositoryRoot 'target\debug\bundle\nsis'
    $candidates = @(Get-ChildItem -LiteralPath $candidateDirectory -Filter '*-setup.exe' -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like 'KRAQEL Installer-Test*' })
    if ($candidates.Count -ne 1) {
        throw "Genau ein KRAQEL-Installer-Testartefakt erwartet, gefunden: $($candidates.Count)."
    }
    $InstallerPath = $candidates[0].FullName
}

$installer = (Resolve-Path -LiteralPath $InstallerPath).Path
try {
    [System.Security.Cryptography.X509Certificates.X509Certificate]::CreateFromSignedFile($installer) | Out-Null
    throw 'Das Testartefakt muss explizit unsigniert sein, enthält aber eine Authenticode-Signatur.'
}
catch [System.Security.Cryptography.CryptographicException] {
    # Erwartet: Das getrennte Testartefakt ist bewusst nicht signiert.
}
if ((Split-Path -Leaf $installer) -notmatch '^KRAQEL Installer-Test_0\.1\.0_.*-setup\.exe$') {
    throw "Unerwarteter Artefaktname: $(Split-Path -Leaf $installer)"
}

$entriesBefore = @(Get-TestUninstallEntries)
if ($entriesBefore.Count -ne 0) {
    throw 'Ein älterer KRAQEL Installer-Test ist noch registriert. Bitte zuerst deinstallieren.'
}

Remove-TestDirectory $installRoot
Remove-TestDirectory $dataRoot
$installed = $false

try {
    $installerProcess = Start-Process -FilePath $installer -ArgumentList @('/S', "/D=$installRoot") -Wait -PassThru
    if ($installerProcess.ExitCode -ne 0) {
        throw "Der NSIS-Installer endete mit Code $($installerProcess.ExitCode)."
    }
    $installed = $true

    $entriesAfterInstall = @(Get-TestUninstallEntries)
    if ($entriesAfterInstall.Count -ne 1) {
        throw "Genau ein Deinstallationseintrag erwartet, gefunden: $($entriesAfterInstall.Count)."
    }
    $registeredLocationValue = [string]$entriesAfterInstall[0].InstallLocation
    $registeredLocation = [System.IO.Path]::GetFullPath($registeredLocationValue.Trim('"')).TrimEnd('\')
    if (-not $registeredLocation.Equals($installRoot.TrimEnd('\'), [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Unerwartetes registriertes Installationsziel: $registeredLocation"
    }

    $applicationCandidates = @(Get-ChildItem -LiteralPath $installRoot -Filter '*.exe' -File |
        Where-Object { $_.Name -notmatch '^uninstall\.exe$' })
    if ($applicationCandidates.Count -ne 1) {
        throw "Genau eine installierte Anwendung erwartet, gefunden: $($applicationCandidates.Count)."
    }

    $smokeArguments = @(
        (Join-Path $repositoryRoot 'scripts\windows-tauri-smoke.mjs'),
        '--app', $applicationCandidates[0].FullName,
        '--data-root', $dataRoot
    )
    if ($TauriDriver) { $smokeArguments += @('--tauri-driver', $TauriDriver) }
    if ($EdgeDriver) { $smokeArguments += @('--edge-driver', $EdgeDriver) }
    & node @smokeArguments
    if ($LASTEXITCODE -ne 0) {
        throw "Der Tauri-Persistenztest endete mit Code $LASTEXITCODE."
    }

    $uninstaller = Join-Path $installRoot 'uninstall.exe'
    if (-not (Test-Path -LiteralPath $uninstaller -PathType Leaf)) {
        throw 'Der erwartete NSIS-Deinstaller fehlt.'
    }
    $uninstallerProcess = Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait -PassThru
    if ($uninstallerProcess.ExitCode -ne 0) {
        throw "Der NSIS-Deinstaller endete mit Code $($uninstallerProcess.ExitCode)."
    }
    $installed = $false

    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while ((Get-TestUninstallEntries).Count -ne 0 -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    if ((Get-TestUninstallEntries).Count -ne 0) {
        throw 'Der Deinstallationseintrag wurde nicht entfernt.'
    }
    if (Test-Path -LiteralPath $applicationCandidates[0].FullName) {
        throw 'Die installierte Anwendung blieb nach der Deinstallation zurück.'
    }

    Write-Output 'Windows-Installer-Smoke-Test erfolgreich.'
    Write-Output "Artefakt: $installer"
    Write-Output 'Nachweis: unsigniertes Testartefakt, Installation, Registrierung, Start, App-Daten, SQLite-Persistenz und Deinstallation.'
}
finally {
    if ($installed) {
        $uninstaller = Join-Path $installRoot 'uninstall.exe'
        if (Test-Path -LiteralPath $uninstaller -PathType Leaf) {
            Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait | Out-Null
        }
    }
    Remove-TestDirectory $installRoot
    Remove-TestDirectory $dataRoot
}
