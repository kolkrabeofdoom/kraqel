# Called by the existing isolated guest task in the measurement overlay.
param([guid]$GuestUuid, [string]$GuestBuild, [guid]$PolicyId, [string]$PolicySha256, [string]$ManifestSha256,
    [ValidateSet('channel-ends','starter-restart','proof-run','proof-incomplete')][string]$ExerciseTrial)
$ErrorActionPreference = 'Stop'
$out = 'C:\BP3-ETW'
$system = "$env:SystemRoot\System32"
$machine = Get-CimInstance Win32_ComputerSystem
if ($machine.Name -ne 'KRAQEL-BP3' -or $machine.PartOfDomain -or
    (Get-CimInstance Win32_ComputerSystemProduct).UUID -ne $GuestUuid -or
    @(Get-NetAdapter -IncludeHidden | Where-Object Status -eq Up).Count -or
    (Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard).UsermodeCodeIntegrityPolicyEnforcementStatus -ne 2) {
    throw 'Guest boundary mismatch.'
}
$logger = "$system\logman.exe"
'{7DD42A49-5329-4832-8DFD-43D979153A88} 0x30 4','{22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716} 0x10 4' | Set-Content "$out\providers.txt" -Encoding ascii
foreach ($mode in 'reference','exercise') {
    $session = "KRAQEL-BP3-$mode"
    & $logger create trace $session -pf "$out\providers.txt" -o "$out\$mode.etl" -f bin -max 128 -bs 64 -nb 16 64 -ets
    if ($LASTEXITCODE -ne 0) { throw 'Cannot start guest trace.' }
    try {
        $controls = @()
        foreach ($addr in '127.0.0.1','::1') {
            foreach ($proto in 'TCP','UDP') {
                if ($proto -eq 'TCP') {
                    $hostPart = if ($addr -eq '::1') { '[::1]' } else { $addr }
                    $exe = 'curl.exe'
                    $port = 135
                    $arguments = "-q --noproxy * --head --silent --max-time 2 --output NUL http://$($hostPart):135/"
                } else {
                    $exe = 'nslookup.exe'
                    $port = 53
                    $arguments = "-timeout=1 -retry=1 -type=A localhost. $addr"
                }
                $helper = Start-Process "$system\$exe" -ArgumentList $arguments -WindowStyle Hidden -PassThru -Wait
                $controls += @{pid=$helper.Id; image=$exe; protocol=$proto; address=$addr; port=$port}
            }
        }
        $workload = if ($mode -eq 'exercise' -and $ExerciseTrial) { $ExerciseTrial } else { $mode }
        $p = Start-Process "$system\WindowsPowerShell\v1.0\powershell.exe" -ArgumentList "-NoProfile -File C:\BP3-Setup\etw-workload.ps1 $workload" -RedirectStandardOutput "$out\$mode.log" -RedirectStandardError "$out\$mode.err" -PassThru -Wait -WindowStyle Hidden
        @{root=$p.Id; exit=$p.ExitCode; controls=$controls} | ConvertTo-Json -Depth 4 | Set-Content "$out\$mode.json"
        if ($p.ExitCode -ne 0) { throw "Workload $mode failed." }
    } finally {
        & $logger stop $session -ets
        if ($LASTEXITCODE -ne 0) { throw 'Cannot stop guest trace.' }
    }
}
