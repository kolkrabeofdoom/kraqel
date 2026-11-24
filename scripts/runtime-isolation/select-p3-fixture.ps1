# Local synthetic choices only. Exit codes are the selection protocol.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
[Windows.Forms.Application]::EnableVisualStyles()
$form = [Windows.Forms.Form]::new()
try {
    $form.Text = 'KRAQEL - P3-Quellauswahl'
    $form.AutoSize = $true
    $form.AutoSizeMode = 'GrowAndShrink'
    $form.FormBorderStyle = 'FixedDialog'
    $form.StartPosition = 'CenterScreen'
    $form.MaximizeBox = $false
    $form.MinimizeBox = $false
    $layout = [Windows.Forms.FlowLayoutPanel]::new()
    $layout.FlowDirection = 'TopDown'
    $layout.WrapContents = $false
    $layout.Padding = 12
    $layout.AutoSize = $true
    $form.Controls.Add($layout)
    $label = [Windows.Forms.Label]::new()
    $label.Text = 'Welchen synthetischen Projektordner als Testkopie erfassen?'
    $label.AutoSize = $true
    $layout.Controls.Add($label)
    foreach ($entry in @('Projekt Nord', 'Projekt Sued', 'Abbrechen')) {
        $button = [Windows.Forms.Button]::new()
        $button.Text = $entry
        $button.AutoSize = $true
        $button.DialogResult = switch ($entry) {
            'Projekt Nord' { 'Yes' }
            'Projekt Sued' { 'No' }
            'Abbrechen'    { 'Cancel' }
        }
        $layout.Controls.Add($button)
    }
    $form.CancelButton = $button
    $form.ActiveControl = $button
    $result = $form.ShowDialog()
} finally {
    $form.Dispose()
}
switch ($result) {
    'Yes' { exit 11 }
    'No' { exit 12 }
    'Cancel' { exit 10 }
    default { exit 1 }
}
