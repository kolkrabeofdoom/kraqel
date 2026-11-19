param([ValidateSet('read-scene','deny-save','edit-conflict','durable-reopen','commit-crashes','handshake')][string]$Trial='commit-crashes')
$ErrorActionPreference='Stop'
$dir='C:\Program Files\KRAQEL-Window-Probe'
try {
    $expected=Get-Content -LiteralPath "$dir\anchor.json" -Raw | ConvertFrom-Json
    if ($env:COMPUTERNAME -ne 'KRAQEL-BP3' -or (Get-CimInstance Win32_ComputerSystemProduct).UUID -ne $expected.uuid -or
        (Get-CimInstance Win32_OperatingSystem).Version -ne $expected.build -or (Get-Process -Id $PID).SessionId -eq 0 -or
        @(Get-NetAdapter -IncludeHidden | Where-Object Status -eq Up).Count -or
        (Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard).UsermodeCodeIntegrityPolicyEnforcementStatus -ne 2) { throw 'Guest boundary mismatch.' }
    $json=& "$env:SystemRoot\System32\CiTool.exe" -lp -json
    if ($LASTEXITCODE -ne 0 -or @((ConvertFrom-Json ($json -join ' ')).Policies | Where-Object { ([string]$_.PolicyID).Trim('{}') -eq $expected.policy -and $_.IsEnforced }).Count -ne 1 -or
        (Get-FileHash "$env:SystemRoot\System32\CodeIntegrity\CiPolicies\Active\{$($expected.policy)}.cip").Hash -ne $expected.policyHash) { throw 'Policy mismatch.' }
    foreach ($file in $expected.files) {
        if ((Get-FileHash -LiteralPath "$dir\$($file.name)").Hash -ne $file.hash) { throw 'Fixture mismatch.' }
    }
    & "$dir\pipe-check.exe" $Trial
    exit $LASTEXITCODE
} catch { Write-Output $_.Exception.Message; exit 1 }
