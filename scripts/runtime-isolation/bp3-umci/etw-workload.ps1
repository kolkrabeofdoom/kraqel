param([Parameter(Mandatory)][ValidateSet('reference','exercise')][string]$Mode)
$ErrorActionPreference = 'Stop'
try {
    $anchor = Get-Content C:\BP3-Proof\anchor.json -Raw | ConvertFrom-Json
    if ((Get-FileHash C:\BP3-UMCI-Fixture\manifest.json).Hash -ne $anchor.fixtureHash) {
        throw 'Unexpected fixture.'
    }
    if ($Mode -eq 'exercise') {
        & "$PSScriptRoot\run-guest.ps1" -GuestUuid $anchor.uuid -GuestBuild $anchor.build -PolicyId $anchor.policy -PolicySha256 $anchor.policyHash -ManifestSha256 $anchor.fixtureHash
    } else {
        Start-Sleep -Seconds 1
    }
    exit 0
} catch {
    Write-Output $_.Exception.Message
    exit 1
}
