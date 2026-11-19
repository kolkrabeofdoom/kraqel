$ErrorActionPreference='Stop'
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$vs) { throw 'MSVC x64 required.' }
Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
$env:RUSTFLAGS='-C target-feature=+crt-static'
& cargo build --manifest-path "$PSScriptRoot\Cargo.toml" --locked --offline --release
if ($LASTEXITCODE) { throw 'Rust build failed.' }
$out="$PSScriptRoot\target\release"
& cl.exe /nologo /W4 /WX /O2 /MT /EHsc /utf-8 "$PSScriptRoot\accessibility.cpp" "/Fo$out\observer.obj" "/Fe$out\observer.exe" user32.lib ole32.lib oleaut32.lib uiautomationcore.lib /link /subsystem:windows
if ($LASTEXITCODE) { throw 'Accessibility observer build failed.' }
