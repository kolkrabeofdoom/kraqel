param([Parameter(Mandatory)][ValidateSet('reference','exercise')][string]$Mode)
$trial = if ($Mode -eq 'reference') { 'reference' } else { 'channel-ends' }
& "$PSScriptRoot\pipe-guest.ps1" -Trial $trial
exit $LASTEXITCODE
