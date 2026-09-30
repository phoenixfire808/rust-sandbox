param([string]$Sheets = (Join-Path $PSScriptRoot '..\sheets'))
$ErrorActionPreference = 'Stop'
# Derived review view. Edit source_weapons/source_vehicles/source_npcs/source_npc_equipment, not this generated coverage table.
$equipment = @{}
Import-Csv (Join-Path $Sheets 'source_npc_equipment.csv') | ForEach-Object { $equipment[$_.id] = $_ }
$references = @{}
Import-Csv (Join-Path $Sheets 'spawn_reference.csv') | ForEach-Object { $references[$_.id] = $_ }
$rows = @()
foreach ($w in Import-Csv (Join-Path $Sheets 'source_weapons.csv')) {
    $rows += [pscustomobject][ordered]@{
        id=$w.id; kind='weapon'; route=$w.kind
        implementation=$(if ($w.kind -eq 'disabled') {'missing'} else {'partial'})
        scope=$w.scope; remaining=$w.remaining
        acceptance=$(if ($w.kind -eq 'disabled') {'Pending must explain missing native behavior and never equip a fake substitute'} elseif ($w.kind -eq 'gravity') {'Punt eligible dynamic props; pull and hold then launch or drop; reject unsupported or heavy targets; verify LOS and mass readiness; changing weapon UI focus death or scene clears gravity state without clearing physgun holds'} else {'Click equips correct model; release mouse then fire; verify impacts and resources; reload; switch and return; inspect animation; undo/load and focus changes remain safe'})
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
foreach ($n in Import-Csv (Join-Path $Sheets 'source_npcs.csv')) {
    $rows += [pscustomobject][ordered]@{
        id=$n.id; kind='npc'; route=$n.kind
        implementation=$(if ($n.kind -eq 'disabled') {'missing'} else {'partial'})
        scope=$(if ($equipment.ContainsKey($n.id)) { $n.scope + '; ' + $equipment[$n.id].scope } else { $n.scope }); remaining=$(if ($equipment.ContainsKey($n.id)) { $n.remaining + '; ' + $equipment[$n.id].remaining } else { $n.remaining })
        acceptance=$(if ($n.kind -eq 'disabled') {'Pending explains unsupported class and does not spawn a cosmetic prop substitute'} else {'Click spawns original animated actor on clear floor; verify facing and collision; hostile actors pursue visible opponents; walls block damage; toggles gate thinking and player targeting; weapon damage removes actors; scene save/load and undo preserve identity and health' + $(if ($equipment.ContainsKey($n.id)) {'; verify one round per burst shot and renewed LOS each round; magazine depletion reloads; spawn and restore reset magazine full'} else {''})})
        result='not_run'; owner='Drew'; reference=$references[$n.id].source; authored_sheet='source_npcs'
    }
}
$rows | Export-Csv (Join-Path $Sheets 'playable_coverage.csv') -NoTypeInformation -Encoding UTF8
Write-Output "Exported $($rows.Count) per-registration coverage rows. No gameplay acceptance was run."
