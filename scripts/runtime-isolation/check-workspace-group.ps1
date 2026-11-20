param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256
)
# C3 synthetic guardian/root/leaf only. No generic network diagnostic or production data.
$ErrorActionPreference='Stop'
if($ProbeSha256 -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $Probe).Hash.ToLowerInvariant() -ne $ProbeSha256){throw 'Unexpected explicitly selected C3 hash'}
$Probe=(Resolve-Path -LiteralPath $Probe).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
New-Item -ItemType Directory -Path (Join-Path $fixtureRoot 'case-01') | Out-Null
$store=Join-Path $fixtureRoot 'case-01/store';$zero='0'*64
$steps=[Collections.Generic.List[object]]::new();$runs=[Collections.Generic.List[object]]::new()
function Invoke-Step([string]$Action,[string]$Head,[int]$Expected=0,[switch]$Data) {
    $info=[Diagnostics.ProcessStartInfo]::new($Probe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    $entry=if($Data){'product-p2c-data-child'}else{'product-c3-group-child'}
    foreach($arg in @($entry,$store,$Action,$Head)){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info)
    try {
        $out=$process.StandardOutput.ReadToEndAsync();$err=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(45000)){$process.Kill();$process.WaitForExit();throw 'C3 deadline exceeded; pending launch must remain blocked'}
        $stdout=$out.GetAwaiter().GetResult();$stderr=$err.GetAwaiter().GetResult();$index=$steps.Count
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$stdout)
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$stderr)
        $steps.Add(@{pid=$process.Id;entry=$entry;action=$Action;exitCode=$process.ExitCode;expectedExitCode=$Expected})
        $steps | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath "$fixtureRoot/processes.json"
        if($process.ExitCode -ne $Expected){throw "Unexpected C3 $Action exit; inspect private evidence in $fixtureRoot"}
        if($Expected -eq 0){return $stdout | ConvertFrom-Json}
    }finally{$process.Dispose()}
}
$state=Invoke-Step init $zero -Data
$state=Invoke-Step prepare $state.control.headSha256 -Data
$key=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
$state=Invoke-Step bind $state.control.headSha256
$oldHead=$state.control.headSha256
foreach($mode in @('start','start-bad-mac','start-unassigned','start-bad-leaf','start')) {
    $state=Invoke-Step $mode $state.control.headSha256
    $expected=($mode -eq 'start')
    if([bool]$state.outcome.passed -ne $expected -or $state.pendingGroup -or -not $state.outcome.groupDrained -or -not $state.outcome.profilePreserved){
        $state | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath "$fixtureRoot/unexpected.json"
        throw "C3 $mode did not match; inspect private evidence in $fixtureRoot"
    }
    if($expected -and (-not $state.outcome.details.rootExitedWhileChildAlive -or -not $state.outcome.details.secondWriterAndRetirementBlocked -or $state.outcome.details.privateDenials -ne 12)){throw 'C3 native group proof missing'}
    $runs.Add(@{mode=$mode;generation=$state.generation;outcome=$state.outcome})
    if($runs.Count -eq 1){Invoke-Step start $oldHead 1;$state=Invoke-Step status $state.control.headSha256}
}
$state=Invoke-Step retire $state.control.headSha256 -Data
$state=Invoke-Step retired-status $state.control.headSha256 -Data
if(-not $state.profileRetired -or (Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key){throw 'C3 profile retirement/key preservation failed'}
@{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;controllerProcesses=$steps.Count;
    generations=5;successfulGroups=2;rejectedBootstraps=3;runs=$runs;profileRemoved=$true;uiStarts=0;networkMeasurement=$false} |
    ConvertTo-Json -Depth 18 | Set-Content -LiteralPath "$fixtureRoot/result.json"
Write-Output $fixtureRoot
