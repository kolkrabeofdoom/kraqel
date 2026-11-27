param(
    [Parameter(Mandatory)][ValidateSet('Missing','Changed')][string]$Case,
    [Parameter(Mandatory)][string]$AnchorPath,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{64}$')][string]$AnchorSha256
)

$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath $AnchorPath).Hash -ne $AnchorSha256) { throw 'Anchor differs.' }
$a = Get-Content -LiteralPath $AnchorPath -Raw | ConvertFrom-Json
$machine = Get-CimInstance Win32_ComputerSystem
if ($machine.Manufacturer -ne 'Microsoft Corporation' -or $machine.Model -ne 'Virtual Machine' -or
    $machine.PartOfDomain -or [guid](Get-CimInstance Win32_ComputerSystemProduct).UUID -ne [guid]$a.uuid -or
    (Get-CimInstance Win32_OperatingSystem).Version -ne $a.build -or
    @(Get-NetAdapter -IncludeHidden | Where-Object Status -eq Up).Count) { throw 'Guest boundary differs.' }
$root = 'C:\BP3-UMCI-Fixture'
$manifestPath = Join-Path $root 'manifest.json'
if ((Get-FileHash -LiteralPath $manifestPath).Hash -ne $a.fixtureHash) { throw 'Unexpected fixture manifest.' }
foreach ($entry in (Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json).files) {
    $name = Split-Path -Leaf $entry.path
    if ((Get-FileHash -LiteralPath (Join-Path $root $name)).Hash -ne $entry.sha256) { throw 'Fixture differs.' }
}

$mode = (Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard).UsermodeCodeIntegrityPolicyEnforcementStatus
$list = & "$env:SystemRoot\System32\CiTool.exe" -lp -json
if ($LASTEXITCODE) { throw 'Policy inventory failed.' }
$policies = (ConvertFrom-Json ($list -join "`n")).Policies
if (-not $policies) { throw 'Policy inventory empty.' }
$match = @($policies | Where-Object { $_.PolicyID -and [guid]$_.PolicyID -eq [guid]$a.policy })
$file = Join-Path "$env:SystemRoot\System32\CodeIntegrity\CiPolicies\Active" ('{' + $a.policy + '}.cip')
if ($Case -eq 'Missing') {
    if ($match.Count -or (Test-Path -LiteralPath $file)) { throw 'Expected the test policy to be absent.' }
    $reason = if ($mode -eq 2) { 'Expected policy is not enforced.' } else { 'The guest is not enforcing user-mode code integrity.' }
} else {
    if ($mode -ne 2 -or $match.Count -ne 1 -or $match[0].IsEnforced -ne $true -or
        (Get-FileHash -LiteralPath $file).Hash -eq $a.policyHash) { throw 'Expected changed, enforced policy bytes.' }
    $reason = 'Installed policy bytes differ from the reviewed policy.'
}

$before = (Get-WinEvent -LogName Security -MaxEvents 1).RecordId
& "$env:SystemRoot\System32\hostname.exe" | Out-Null
if ($LASTEXITCODE) { throw 'First process control failed.' }
$denial = $null
try {
    & (Join-Path $PSScriptRoot 'run-guest.ps1') -GuestUuid $a.uuid -GuestBuild $a.build -PolicyId $a.policy -PolicySha256 $a.policyHash -ManifestSha256 $a.fixtureHash | Out-Null
} catch { $denial = $_.Exception.Message }
& "$env:SystemRoot\System32\whoami.exe" | Out-Null
if ($LASTEXITCODE -or $denial -cne $reason) { throw 'Expected gate denial was not observed.' }

$deadline = (Get-Date).AddSeconds(10)
do {
    $after = (Get-WinEvent -LogName Security -MaxEvents 1).RecordId
    $query = "*[System[EventID=4688 and EventRecordID > $before and EventRecordID <= $after]]"
    $xml = & "$env:SystemRoot\System32\wevtutil.exe" qe Security "/q:$query" /f:xml /e:Events
    if ($LASTEXITCODE) { throw 'Cannot read process creation events.' }
    $starts = 0
    $first = $last = 0
    foreach ($event in (Select-Xml -Content ($xml -join "`n") -XPath "//*[local-name()='Event']")) {
        $fields = @{}
        foreach ($data in $event.Node.EventData.Data) { $fields[$data.Name] = $data.'#text' }
        if ($fields.NewProcessName -eq "$root\bp3-starter.exe") { $starts++ }
        if ([long]$fields.ProcessId -ne $PID) { continue }
        if ($fields.NewProcessName -eq "$env:SystemRoot\System32\hostname.exe") { $first = [long]$event.Node.System.EventRecordID }
        if ($fields.NewProcessName -eq "$env:SystemRoot\System32\whoami.exe") { $last = [long]$event.Node.System.EventRecordID }
    }
    if ($first -gt $before -and $last -gt $first) { break }
    Start-Sleep -Milliseconds 200
} while ((Get-Date) -lt $deadline)
if ($first -le $before -or $last -le $first -or $starts -ne 0) { throw 'Process evidence incomplete or starter executed.' }
[ordered]@{ case=$Case; enforcement=$mode; expectedIdPresent=($match.Count -eq 1); denial=$denial
    before=$before; after=$after; firstControl=$first; lastControl=$last; starterStarts=$starts }
