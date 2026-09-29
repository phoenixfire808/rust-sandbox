param([string]$Sheets = (Join-Path $PSScriptRoot '..\sheets'))
$ErrorActionPreference = 'Stop'
# Derived review view. Edit source_weapons/source_vehicles, not this generated coverage table.
$references = @{}
Import-Csv (Join-Path $Sheets 'spawn_reference.csv') | ForEach-Object { $references[$_.id] = $_ }
$rows = @()
foreach ($w in Import-Csv (Join-Path $Sheets 'source_weapons.csv')) {
    $rows += [pscustomobject][ordered]@{
        id=$w.id; kind='weapon'; route=$w.kind
        implementation=$(if ($w.kind -eq 'disabled') {'missing'} else {'partial'})
        scope=$w.scope; remaining=$w.remaining
        acceptance=$(if ($w.kind -eq 'disabled') {'Pending must explain missing native behavior and never equip a fake substitute'} else {'Click equips correct model; release mouse then fire; verify impacts and resources; reload; switch and return; inspect animation; undo/load and focus changes remain safe'})
        result='not_run'; owner='Drew'; reference=$references[$w.id].source; authored_sheet='source_weapons'
    }
}
foreach ($v in Import-Csv (Join-Path $Sheets 'source_vehicles.csv')) {
    $rows += [pscustomobject][ordered]@{
        id=$v.id; kind='vehicle'; route=$v.kind; implementation='partial'; scope=$v.scope; remaining=$v.remaining
        acceptance=$(if ($v.kind -eq 'seat') {'Click spawns correct seat; E enters and exits; seated pose blends; no invented engine; blocked exits reject; removal undo duplication and save/load preserve identity'} else {'Click spawns correct vehicle; E enters; verify seated transition; WASD accelerates reverses and steers; Space brakes; E exits safely; F4 camera; removal undo duplication save/load preserve identity; airboat supports mapped water'})
        result='not_run'; owner='Drew'; reference=$references[$v.id].source; authored_sheet='source_vehicles'
    }
}
$rows | Export-Csv (Join-Path $Sheets 'playable_coverage.csv') -NoTypeInformation -Encoding UTF8
Write-Output "Exported $($rows.Count) per-registration coverage rows. No gameplay acceptance was run."
