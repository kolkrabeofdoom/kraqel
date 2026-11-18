param(
    [Parameter(Mandatory)][guid]$GuestUuid,
    [Parameter(Mandatory)][string]$GuestBuild,
    [Parameter(Mandatory)][guid]$PolicyId,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{64}$')][string]$PolicySha256,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{64}$')][string]$ManifestSha256
)

$ErrorActionPreference = 'Stop'
$root = 'C:\BP3-UMCI-Fixture'
$log = 'Microsoft-Windows-CodeIntegrity/Operational'
$machine = Get-CimInstance Win32_ComputerSystem
$product = Get-CimInstance Win32_ComputerSystemProduct
$os = Get-CimInstance Win32_OperatingSystem
if ($machine.Manufacturer -ne 'Microsoft Corporation' -or $machine.Model -ne 'Virtual Machine' -or
    $GuestUuid -eq [guid]::Empty -or [guid]$product.UUID -ne $GuestUuid -or
    $os.Version -ne $GuestBuild -or $machine.PartOfDomain) {
    throw 'Guest identity, build or workgroup mismatch.'
}
if (@(Get-NetAdapter -IncludeHidden | Where-Object Status -eq 'Up').Count -ne 0) {
    throw 'An active guest adapter was found.'
}
$ci = Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard
if ($ci.UsermodeCodeIntegrityPolicyEnforcementStatus -ne 2) {
    throw 'The guest is not enforcing user-mode code integrity.'
}
$json = & "$env:SystemRoot\System32\CiTool.exe" -lp -json
if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate effective policies.' }
$matching = @((ConvertFrom-Json ($json -join "`n")).Policies | Where-Object {
    $_.PolicyID -and [guid]$_.PolicyID -eq $PolicyId -and $_.IsEnforced -eq $true
})
if ($PolicyId -eq [guid]::Empty -or $matching.Count -ne 1) { throw 'Expected policy is not enforced.' }
$installed = Join-Path "$env:SystemRoot\System32\CodeIntegrity\CiPolicies\Active" ('{' + $PolicyId + '}.cip')
if ((Get-FileHash -LiteralPath $installed).Hash -ne $PolicySha256) {
    throw 'Installed policy bytes differ from the reviewed policy.'
}
$manifestPath = Join-Path $root 'manifest.json'
if ((Get-FileHash -LiteralPath $manifestPath).Hash -ne $ManifestSha256) {
    throw 'Fixture manifest differs from the reviewed manifest.'
}
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$names = [ordered]@{
    'allow/bp3-starter.exe' = 'bp3-starter.exe'
    'allow/bp3-worker.exe' = 'bp3-worker.exe'
    'allow/allowed.dll' = 'allowed.dll'
    'deny/foreign.exe' = 'foreign.exe'
    'deny/foreign.dll' = 'foreign.dll'
    'input.bin' = 'input.bin'
}
if ($manifest.version -ne 1 -or $manifest.purpose -ne 'bp3-umci-synthetic-fixture' -or
    $manifest.files.Count -ne $names.Count -or
    ((Get-Item -LiteralPath $root).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw 'Unexpected fixture shape.'
}
foreach ($name in $names.Keys) {
    $expected = @($manifest.files | Where-Object path -ceq $name)
    $item = Get-Item -LiteralPath (Join-Path $root $names[$name])
    if ($expected.Count -ne 1 -or $item.PSIsContainer -or
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        $item.Length -ne $expected[0].bytes -or
        (Get-FileHash -LiteralPath $item.FullName).Hash -ne $expected[0].sha256) {
        throw "Fixture verification failed for $name."
    }
}
$start = Get-Date
$output = & (Join-Path $root 'bp3-starter.exe') --prepared-offline-guest
$exitCode = $LASTEXITCODE
$output
if ($exitCode -ne 0) { throw "Starter failed with $exitCode." }

$deadline = $start.AddSeconds(30)
do {
    $blocks = @(Get-WinEvent -FilterHashtable @{ LogName = $log; Id = 3077; StartTime = $start } -ErrorAction SilentlyContinue)
    $counts = [ordered]@{}
    foreach ($name in 'foreign.exe','foreign.dll') {
        $counts[$name] = @($blocks | Where-Object { $_.Message -like "*\BP3-UMCI-Fixture\$name*" }).Count
    }
    if ($counts['foreign.exe'] -and $counts['foreign.dll']) { break }
    Start-Sleep -Milliseconds 250
} while ((Get-Date) -lt $deadline)
if (-not $counts['foreign.exe'] -or -not $counts['foreign.dll']) {
    throw 'Required Code Integrity block events were not found in the run window.'
}
[ordered]@{ starterExit = $exitCode; nativeFixtureChecksPassed = $true; blockEvents = $counts } |
    ConvertTo-Json -Depth 3
