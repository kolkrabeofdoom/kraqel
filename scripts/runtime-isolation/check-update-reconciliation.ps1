param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256,
    [Parameter(Mandatory=$true)][string]$App,
    [Parameter(Mandatory=$true)][string]$AppSha256,
    [switch]$RetryAborted,
    [switch]$PrepareTransitions
)
# Local synthetic research only. Keeps private evidence outside Git; never opens the real store.
$ErrorActionPreference='Stop'
if($RetryAborted -and $PrepareTransitions){throw 'Select one bounded research series'}
foreach($pair in @(@($Probe,$ProbeSha256),@($App,$AppSha256))) {
    if($pair[1] -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $pair[0]).Hash.ToLowerInvariant() -ne $pair[1]) {
        throw 'Expected an explicitly approved complete executable hash'
    }
}
$Probe=(Resolve-Path -LiteralPath $Probe).Path
$App=(Resolve-Path -LiteralPath $App).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$candidate=Join-Path $fixtureRoot 'candidate.exe'
Copy-Item -LiteralPath $Probe -Destination $candidate
$file=[IO.File]::Open($candidate,[IO.FileMode]::Append,[IO.FileAccess]::Write)
try {$file.Write([Text.Encoding]::UTF8.GetBytes('KRAQEL schema4 synthetic candidate'));$file.Flush($true)} finally {$file.Dispose()}
$candidateSha=(Get-FileHash -LiteralPath $candidate).Hash.ToLowerInvariant()
$steps=[Collections.Generic.List[object]]::new()
$held=[Collections.Generic.List[Diagnostics.Process]]::new()
$cases=[Collections.Generic.List[object]]::new()
$zero='0'*64
function Invoke-Control([string]$Action,[string]$Head,[int]$ExpectedExit=0,[string]$Point='none',[string]$Exe=$Probe) {
    $info=[Diagnostics.ProcessStartInfo]::new($Exe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true
    $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    if($Point -ne 'none') {
        $argv=@('product-control-child',$store,$Action,$Point,$Head,$App)
    } else {
        $source=$App;$sha=$AppSha256
        if($Action -eq 'prepare-update'){$source=$candidate;$sha=$candidateSha}
        $argv=@('product-session-child',$store,$Action,$source,$sha,$Head)
    }
    foreach($arg in $argv){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info);$null=$process.Handle;$held.Add($process)
    $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
    if(-not $process.WaitForExit(60000)){$process.Kill();$process.WaitForExit();throw 'Native reconciliation timeout'}
    $out=$stdout.GetAwaiter().GetResult();$err=$stderr.GetAwaiter().GetResult();$index=$steps.Count
    [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$out)
    [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$err)
    $steps.Add(@{pid=$process.Id;case=$caseName;action=$Action;point=$Point;image=[IO.Path]::GetFileName($Exe);exitCode=$process.ExitCode;expectedExitCode=$ExpectedExit})
    $steps | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $fixtureRoot 'processes.json')
    if($process.ExitCode -ne $ExpectedExit){throw 'Unexpected native reconciliation exit; inspect local evidence'}
    if($ExpectedExit -eq 0){return $out | ConvertFrom-Json}
}
function Control-Fingerprint {
    $entries=@(Get-ChildItem -LiteralPath "$store/control/records" -File | Sort-Object Name | ForEach-Object {$_.Name+':'+(Get-FileHash -LiteralPath $_.FullName).Hash})
    foreach($name in @('head.mac','authority.key')){$entries+=($name+':'+(Get-FileHash -LiteralPath "$store/control/$name").Hash)}
    return $entries -join '|'
}
try {
    $faults=if($PrepareTransitions){@('artifact-removed','package-created','prepare-recorded')}else{@('partial-copy','tampered-package','abort-published')}
    foreach($fault in $faults) {
        $caseName='case-'+$cases.Count.ToString('D2')
        $caseRoot=Join-Path $fixtureRoot $caseName;New-Item -ItemType Directory -Path $caseRoot | Out-Null
        $store=Join-Path $caseRoot 'store'
        $state=Invoke-Control init $zero
        $state=Invoke-Control migrate4 $state.control.headSha256
        $state=Invoke-Control bind $state.control.headSha256
        $before=$state.control.headSha256
        $key=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
        if($PrepareTransitions) {
            $state=Invoke-Control prepare-update $before
            $state=Invoke-Control abort-update $state.control.headSha256
            $abortedHead=$state.control.headSha256
            $package=Join-Path $store "bin/$candidateSha.exe"
            Invoke-Control prepare $abortedHead 86 $fault
            $state=Invoke-Control status $zero
            $crashHead=$state.control.headSha256
            $hasPending=($fault -eq 'prepare-recorded')
            $packagePresent=Test-Path -LiteralPath $package
            $packageBytes=if($packagePresent){(Get-Item -LiteralPath $package).Length}else{$null}
            if($state.control.pendingUpdate -ne $hasPending -or $packagePresent -ne ($fault -ne 'artifact-removed') -or ($packagePresent -and $packageBytes -ne 0)){throw 'Unexpected crash boundary state'}
            if(-not $hasPending -and $crashHead -ne $abortedHead){throw 'Early crash changed control head'}
            $fingerprint=Control-Fingerprint
            Invoke-Control activate-update $crashHead 1
            if($hasPending){Invoke-Control recover $crashHead 1}
            if((Control-Fingerprint) -ne $fingerprint){throw 'Rejected post-crash operation mutated control'}
            if($hasPending){$state=Invoke-Control abort-update $crashHead}
            $retryHead=$state.control.headSha256
            $fingerprint=Control-Fingerprint
            Invoke-Control prepare-update $before 1
            if((Control-Fingerprint) -ne $fingerprint -or (Test-Path -LiteralPath $package) -ne $packagePresent -or ($packagePresent -and (Get-Item -LiteralPath $package).Length -ne 0)){throw 'Stale retry mutated artifact/control'}
            $state=Invoke-Control prepare-update $retryHead
            if(-not $state.control.pendingUpdate -or (Get-FileHash -LiteralPath $package).Hash.ToLowerInvariant() -ne $candidateSha){throw 'Boundary retry did not prepare exact package'}
            $state=Invoke-Control abort-update $state.control.headSha256
            $state=Invoke-Control recover $state.control.headSha256
            if($state.outcome.summary.projectCount -ne 1 -or $state.outcome.summary.documentCount -ne 2 -or -not $state.outcome.summary.replayBlocked){throw 'Boundary recovery changed import authority'}
            $state=Invoke-Control retire $state.control.headSha256
            $cleanup=Invoke-Control cleanup $state.control.headSha256
            if(-not $cleanup.outcome.profileAbsent -or -not $cleanup.outcome.alreadyRecorded -or $cleanup.control.headSha256 -ne $state.control.headSha256){throw 'Boundary cleanup did not settle'}
            if((Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key){throw 'Boundary crash changed key'}
            $cases.Add(@{fault=$fault;localStoreRoot=$store;beforeHead=$before;abortedHead=$abortedHead;crashHead=$crashHead;pendingAtCrash=$hasPending;packagePresentAtCrash=$packagePresent;packageBytesAtCrash=$packageBytes;final=$cleanup;rejectedControlUnchanged=$true;keyUnchanged=$true})
            continue
        }
        if($fault -eq 'partial-copy') {
            Invoke-Control prepare $before 86 'package-partial'
            $state=Invoke-Control status $zero
        } else {
            $state=Invoke-Control prepare-update $before
        }
        $pending=$state.control.headSha256
        if(-not $state.control.pendingUpdate){throw 'Missing authenticated pending update'}
        $package=Join-Path $store "bin/$candidateSha.exe"
        if($fault -eq 'tampered-package') {
            $file=[IO.File]::Open($package,[IO.FileMode]::Append,[IO.FileAccess]::Write)
            try {$file.Write([Text.Encoding]::UTF8.GetBytes('synthetic corruption'));$file.Flush($true)} finally {$file.Dispose()}
        }
        $unchanged=Control-Fingerprint
        if($fault -ne 'abort-published'){Invoke-Control activate-update $pending 1}
        Invoke-Control abort-update $before 1
        Invoke-Control abort-update $pending 1 'none' $candidate
        Invoke-Control recover $pending 1
        if($fault -eq 'partial-copy') {
            $lock=[IO.File]::Open("$store/control/owner.lock",[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
            try {Invoke-Control abort-update $pending 1;Invoke-Control activate-update $pending 1} finally {$lock.Dispose()}
        }
        if((Control-Fingerprint) -ne $unchanged){throw 'Rejected operation mutated control'}
        if($fault -eq 'abort-published') {
            Invoke-Control abort $pending 86 'head-published'
            $state=Invoke-Control status $zero
        } else {
            $state=Invoke-Control abort-update $pending
        }
        if($state.control.pendingUpdate -or $state.control.activeBinarySha256 -ne $ProbeSha256){throw 'Abort changed actor or retained pending'}
        $after=$state.control.headSha256;$terminal=Control-Fingerprint
        Invoke-Control abort-update $after 1
        Invoke-Control activate-update $after 1
        if((Control-Fingerprint) -ne $terminal){throw 'Terminal retry mutated control'}
        $retryEvidence=$null
        if($RetryAborted) {
            $beforeBytes=(Get-Item -LiteralPath $package).Length
            $beforeSha=(Get-FileHash -LiteralPath $package).Hash.ToLowerInvariant()
            Invoke-Control prepare-update $pending 1
            $packageLock=[IO.File]::Open($package,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
            try {Invoke-Control prepare-update $after 1} finally {$packageLock.Dispose()}
            if((Control-Fingerprint) -ne $terminal -or (Get-FileHash -LiteralPath $package).Hash.ToLowerInvariant() -ne $beforeSha){throw 'Rejected retry mutated control or package'}
            $state=Invoke-Control prepare-update $after
            if(-not $state.control.pendingUpdate -or (Get-FileHash -LiteralPath $package).Hash.ToLowerInvariant() -ne $candidateSha){throw 'Retry did not prepare exact candidate'}
            $retryEvidence=@{beforeBytes=$beforeBytes;beforeSha256=$beforeSha;preparedHead=$state.control.headSha256;rejectedControlAndPackageUnchanged=$true}
            $state=Invoke-Control abort-update $state.control.headSha256
            $after=$state.control.headSha256
        }
        $state=Invoke-Control recover $after
        if($state.outcome.summary.projectCount -ne 1 -or $state.outcome.summary.documentCount -ne 2 -or -not $state.outcome.summary.replayBlocked){throw 'Recovery or replay invariant failed'}
        $state=Invoke-Control retire $state.control.headSha256
        $cleanup=Invoke-Control cleanup $state.control.headSha256
        if(-not $cleanup.outcome.profileAbsent -or -not $cleanup.outcome.alreadyRecorded -or $cleanup.control.headSha256 -ne $state.control.headSha256){throw 'Cleanup did not settle idempotently'}
        if((Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key){throw 'Key changed'}
        $cases.Add(@{fault=$fault;localStoreRoot=$store;beforeHead=$before;pendingHead=$pending;abortedHead=$after;retry=$retryEvidence;final=$cleanup;rejectedControlUnchanged=$true;keyUnchanged=$true})
    }
    @{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;candidateSha256=$candidateSha;appSha256=$AppSha256;childProcesses=$steps.Count;retryAborted=[bool]$RetryAborted;prepareTransitions=[bool]$PrepareTransitions;cases=$cases;networkMeasurement=$false} | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $fixtureRoot 'result.json')
    Write-Output $fixtureRoot
} finally {
    foreach($process in $held){$process.Dispose()}
}
