param(
    [Parameter(Mandatory = $true)]
    [string[]] $Path
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_BASE64) -or
    [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_PASSWORD)) {
    throw "WINDOWS_CERTIFICATE_BASE64 and WINDOWS_CERTIFICATE_PASSWORD are required"
}

$signTool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if (-not $signTool) {
    throw "signtool.exe was not found in the Windows SDK"
}

$certificatePath = Join-Path $env:RUNNER_TEMP "espanso-gui-signing.pfx"
try {
    [IO.File]::WriteAllBytes(
        $certificatePath,
        [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_BASE64)
    )

    foreach ($candidate in $Path) {
        $resolved = Resolve-Path $candidate -ErrorAction Stop
        & $signTool.FullName sign `
            /f $certificatePath `
            /p $env:WINDOWS_CERTIFICATE_PASSWORD `
            /fd SHA256 `
            /tr http://timestamp.digicert.com `
            /td SHA256 `
            /d "Espanso GUI" `
            $resolved.Path
        if ($LASTEXITCODE -ne 0) {
            throw "signtool sign failed for $($resolved.Path)"
        }
        & $signTool.FullName verify /pa /v $resolved.Path
        if ($LASTEXITCODE -ne 0) {
            throw "signtool verify failed for $($resolved.Path)"
        }
    }
}
finally {
    Remove-Item $certificatePath -Force -ErrorAction SilentlyContinue
}
