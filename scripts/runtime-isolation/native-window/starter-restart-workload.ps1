param([Parameter(Mandatory)][ValidateSet('reference','exercise')][string]$Mode)
$trial = if ($Mode -eq 'reference') { 'reference' } else { 'starter-restart' }
& "$PSScriptRoot\pipe-guest.ps1" -Trial $trial
exit $LASTEXITCODE
