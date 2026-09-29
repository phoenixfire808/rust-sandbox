param(
    [string]$Install = 'D:\SteamLibrary\steamapps\common\GarrysMod',
    [Parameter(Mandatory=$true)][string]$Output
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Output) { throw 'Choose a new output filename. Existing research is never overwritten.' }
$base = Join-Path $Install 'garrysmod/gamemodes/sandbox/entities/weapons/gmod_tool/stools'
$rows = foreach ($file in Get-ChildItem -LiteralPath $base -Filter '*.lua' | Sort-Object Name) {
    $line = 0
    foreach ($text in Get-Content -LiteralPath $file.FullName) {
        $line++
        if ($text -match '^\s*TOOL\.ClientConVar\[\s*"(?<key>[^"]+)"\s*\]\s*=\s*(?<value>"[^"]*"|[-0-9.]+)') {
            [pscustomobject]@{
                tool = $file.BaseName
                setting = $Matches.key
                default = $Matches.value.Trim('"')
                source_line = $line
                source = 'garrysmod/gamemodes/sandbox/entities/weapons/gmod_tool/stools/' + $file.Name
                evidence = 'literal_installed_default_not_runtime_conformance'
            }
        }
    }
}
$rows | Export-Csv -LiteralPath $Output -NoTypeInformation -Encoding UTF8
Write-Output ('Exported ' + @($rows).Count + ' literal tool defaults. Computed and dynamically registered defaults are not claimed covered.')
