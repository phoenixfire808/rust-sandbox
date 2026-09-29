param(
    [string]$Install = 'D:\SteamLibrary\steamapps\common\GarrysMod',
    [string]$Sheets = (Join-Path $PSScriptRoot '..\sheets')
)
$ErrorActionPreference = 'Stop'
$root = Join-Path $Install 'garrysmod'
# Read-only metadata extraction. Never evaluate Lua or overwrite authored capability/route sheets.
function Clean([string]$s) {
    [regex]::Replace($s, '(?s)"(?:\\.|[^"\\])*"|''(?:\\.|[^''\\])*''|--\[\[.*?\]\]|--[^\r\n]*', {
        param($m)
        if ($m.Value.StartsWith('--')) { [regex]::Replace($m.Value, '[^\r\n]', ' ') } else { $m.Value }
    })
}
function Parts([string]$s) {
    $start=0; $depth=0; $quote=[char]0; $escape=$false
    for ($i=0; $i -lt $s.Length; $i++) {
        $c=$s[$i]
        if ($quote -ne [char]0) {
            if ($escape) { $escape=$false } elseif ($c -eq '\') { $escape=$true } elseif ($c -eq $quote) { $quote=[char]0 }
            continue
        }
        if ($c -eq '"' -or $c -eq "'") { $quote=$c; continue }
        if ($c -in @('{','(','[')) { $depth++ }
        if ($c -in @('}',')',']')) { $depth-- }
        if ($c -eq ',' -and $depth -eq 0) { $s.Substring($start,$i-$start).Trim(); $start=$i+1 }
    }
    if ($start -lt $s.Length) { $s.Substring($start).Trim() }
}
function Scalar([string]$s) {
    $s=$s.Trim()
    if ($s.Length -ge 2 -and (($s.StartsWith('"') -and $s.EndsWith('"')) -or ($s.StartsWith("'") -and $s.EndsWith("'")))) { return $s.Substring(1,$s.Length-2) }
    return $s
}
function Fields([string]$s) {
    $out=[ordered]@{}
    $s=$s.Trim()
    if (!$s.StartsWith('{') -or !$s.EndsWith('}')) { throw "Expected literal table, got $s" }
    foreach ($p in @(Parts $s.Substring(1,$s.Length-2))) {
        if ($p -match '(?s)^\s*(\w+)\s*=\s*(.*)$') { $out[$matches[1]]=[regex]::Replace($matches[2].Trim(),'\s+',' ') }
        elseif ($p) { throw "Unrecognized declaration field: $p" }
    }
    return $out
}
function Calls([string]$s,[string]$name) {
    foreach ($m in [regex]::Matches($s, '(?m)^\s*' + [regex]::Escape($name) + '\s*\(')) {
        $begin=$m.Index+$m.Length; $depth=1; $quote=[char]0; $escape=$false; $end=$begin
        for (; $end -lt $s.Length; $end++) {
            $c=$s[$end]
            if ($quote -ne [char]0) {
                if ($escape) { $escape=$false } elseif ($c -eq '\') { $escape=$true } elseif ($c -eq $quote) { $quote=[char]0 }
                continue
            }
            if ($c -eq '"' -or $c -eq "'") { $quote=$c; continue }
            if ($c -eq '(') { $depth++ }; if ($c -eq ')') { $depth--; if (!$depth) { break } }
        }
        if ($depth) { throw "Unbalanced $name declaration" }
        $actual=$s.IndexOf($name,$m.Index)
        [pscustomobject]@{ index=$actual; line=1+([regex]::Matches($s.Substring(0,$actual),'\n')).Count; args=@(Parts $s.Substring($begin,$end-$begin)) }
    }
}
$phrases=@{}
Get-ChildItem (Join-Path $root 'resource/localization/en') -Filter *.properties -File | Sort-Object Name | ForEach-Object {
    foreach ($line in [IO.File]::ReadAllLines($_.FullName)) {
        if ($line -match '^\s*([^#=\s][^=]*)=(.*)$') { $phrases[$matches[1].Trim()]=$matches[2].Trim() }
    }
}
function Label([string]$s) { $s=Scalar $s; if ($s.StartsWith('#') -and $phrases.ContainsKey($s.Substring(1))) { return $phrases[$s.Substring(1)] }; return $s }
function Category([string]$s,[int]$index) {
    $m=[regex]::Matches($s.Substring(0,$index),'(?m)^\s*(?:local\s+)?Category\s*=\s*"([^"]*)"')
    if ($m.Count) { return (Scalar $m[$m.Count-1].Groups[1].Value) }; return 'Other'
}
function Condition([string]$category,[string]$class) {
    if ($class -eq 'npc_fisherman') { return 'mounted:lostcoast' }
    if ($category -eq 'Half-Life: Source') { return 'mounted:hl1_or_hl1mp' }
    if ($category -eq 'Portal') { return 'mounted:portal' }
    return 'base'
}
$entries=[Collections.Generic.List[object]]::new()
$scope=[Collections.Generic.List[object]]::new()
function Track([string]$relative,[string]$role,[int]$count) {
    $path=Join-Path $root $relative
    $scope.Add([pscustomobject][ordered]@{ source=$relative; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant(); role=$role; extracted_rows=$count; limitation='Literal metadata only; no Lua execution or native behavior verification' })
}
function Entry([string]$kind,[string]$key,[string]$class,[string]$name,[string]$category,[System.Collections.IDictionary]$fields,[string]$source,[int]$line,[string]$visibility) {
    if (!$class -or !$key) { throw "Missing class/key at ${source}:$line" }
    $entries.Add([pscustomobject][ordered]@{
        id=$kind+'_'+$key.ToLowerInvariant(); kind=$kind; spawn_name=$key; class_name=$class
        label=(Label $name); category=(Label $category); model=(Scalar ([string]$fields['Model']))
        icon='entities/'+$key+'.png'; condition=(Condition $category $class); visibility=$visibility
        admin_only=([string]$fields['AdminOnly'] -eq 'true').ToString().ToLowerInvariant()
        defaults_json=($fields | ConvertTo-Json -Compress -Depth 8); source=$source; line=$line
    })
}
foreach ($pair in @(@('npc','AddNPC','lua/autorun/base_npcs.lua'),@('vehicle','AddVehicle','lua/autorun/base_vehicles.lua'))) {
    $source=$pair[2]; $s=Clean ([IO.File]::ReadAllText((Join-Path $root $source))); $before=$entries.Count
    foreach ($call in @(Calls $s $pair[1])) {
        $f=Fields $call.args[0]; $class=Scalar ([string]$f['Class']); $key=$class
        if ($call.args.Count -gt 1) { $key=Scalar $call.args[1] }
        $name=[string]$f['Name']; if (!$name) { $name='#'+$key }
        $cat=Scalar ([string]$f['Category']); if ($cat -eq 'Category') { $cat=Category $s $call.index }
        $f['Category']=$cat
        Entry $pair[0] $key $class $name $cat $f $source $call.line 'spawnmenu'
    }
    Track $source 'NPC or vehicle registrations including conditional mounts' ($entries.Count-$before)
}
$source='lua/autorun/game_hl2.lua'; $s=Clean ([IO.File]::ReadAllText((Join-Path $root $source))); $before=$entries.Count
foreach ($pair in @(@('entity','ADD_ITEM'),@('weapon','ADD_WEAPON'))) {
    foreach ($call in @(Calls $s $pair[1])) {
        $class=Scalar $call.args[0]; $key=$class; $cat=Category $s $call.index; $f=[ordered]@{}
        if ($pair[0] -eq 'entity') {
            $f['NormalOffset']='32'; $f['DropToFloor']='true'
            if ($call.args.Count -gt 1) { $f['NormalOffset']=$call.args[1] }
            if ($call.args.Count -gt 2) { $extras=Fields $call.args[2]; foreach ($k in $extras.Keys) { $f[$k]=$extras[$k] } }
            if ($call.args.Count -gt 3) { $key=Scalar $call.args[3] }
        }
        $name='#'+$key; if ($f.Contains('PrintName')) { $name=[string]$f['PrintName'] }
        Entry $pair[0] $key $class $name $cat $f $source $call.line 'spawnmenu'
    }
}
Track $source 'Native item and weapon registration helpers; excludes NPC equipment-only declarations' ($entries.Count-$before)
foreach ($pair in @(@('entity','lua/entities'),@('weapon','lua/weapons'),@('entity','gamemodes/sandbox/entities/entities'),@('weapon','gamemodes/sandbox/entities/weapons'))) {
    foreach ($item in Get-ChildItem (Join-Path $root $pair[1]) | Sort-Object Name) {
        $file=$item; $class=$item.BaseName
        if ($item.PSIsContainer) { $file=Get-Item (Join-Path $item.FullName 'shared.lua') -ErrorAction SilentlyContinue; $class=$item.Name }
        if (!$file -or $file.Extension -ne '.lua') { continue }
        $s=Clean ([IO.File]::ReadAllText($file.FullName)); $f=[ordered]@{}; $line=1
        foreach ($m in [regex]::Matches($s,'(?m)^\s*(?:ENT|SWEP)\.(PrintName|Category|Spawnable|AdminOnly|Base|Type)\s*=\s*([^\r\n]+)')) {
            $f[$m.Groups[1].Value]=$m.Groups[2].Value.Trim(); $line=1+([regex]::Matches($s.Substring(0,$m.Index),'\n')).Count
        }
        $visibility='inherited_or_internal'; if ($f['Spawnable'] -eq 'true') { $visibility='spawnmenu' } elseif ($f['Spawnable'] -eq 'false') { $visibility='internal' }
        $name=[string]$f['PrintName']; if (!$name) { $name=$class }
        $cat=[string]$f['Category']; if (!$cat) { $cat='Other' }
        $source=$file.FullName.Substring($root.Length+1).Replace('\','/')
        Entry $pair[0] $class $class $name (Scalar $cat) $f $source $line $visibility
        Track $source 'Scripted entity or weapon literal registration fields; inheritance not executed' 1
    }
}
if (@($entries | Group-Object id | Where-Object Count -gt 1).Count) { throw 'Duplicate spawn reference IDs' }
$entries | Sort-Object kind,category,label,id | Export-Csv (Join-Path $Sheets 'spawn_reference.csv') -NoTypeInformation -Encoding UTF8
# Capture literal geometry/control/registration metadata from stock UI, not copied function bodies.
$ui=[Collections.Generic.List[object]]::new()
$uiFiles=@(Get-ChildItem (Join-Path $root 'gamemodes/sandbox/gamemode/spawnmenu') -Recurse -Filter *.lua)
$uiFiles+=@(Get-ChildItem (Join-Path $root 'lua/skins') -Filter *.lua)
$uiFiles+=@(Get-ChildItem (Join-Path $root 'lua/autorun/properties') -Filter *.lua)
$uiFiles+=@(Get-ChildItem (Join-Path $root 'lua/vgui') -Filter *.lua)
$uiFiles+=@(Get-Item (Join-Path $root 'lua/autorun/menubar.lua'),(Join-Path $root 'lua/autorun/properties.lua'),(Join-Path $root 'lua/autorun/utilities_menu.lua'))
foreach ($file in $uiFiles | Sort-Object FullName -Unique) {
    $s=Clean ([IO.File]::ReadAllText($file.FullName)); $source=$file.FullName.Substring($root.Length+1).Replace('\','/'); $before=$ui.Count
    $lineNo=0
    foreach ($line in ($s -split '\r?\n')) {
        $lineNo++
        foreach ($m in [regex]::Matches($line,'(?<receiver>[\w.]+)[:.](?<method>SetSize|SetWide|SetTall|SetPos|SetLeftWidth|SetRightMin|SetLeftMin|SetDividerWidth|SetOpenSize|SetSpaceX|SetSpaceY|SetBorder|Dock|DockMargin|DockPadding|SetText|SetFont|SetImage|SetMaterial|SetConVar|SetCookieName|SetKeyboardInputEnabled|SetMouseInputEnabled|SetWorldClicker|SetDoubleClickingEnabled|AddOption|AddChoice|AddCreationTab|AddToolMenuTab|AddToolMenuOption|CreateContentIcon|AddContentType|Register|Create|CheckBox|NumSlider|ComboBox|TextEntry)\s*\(')) {
            # Bound each call on this line, honoring nested calls and strings. Multiline calls remain partial metadata.
            $begin=$m.Index+$m.Length; $end=$begin; $depth=1; $quote=[char]0; $escape=$false
            for (; $end -lt $line.Length; $end++) {
                $ch=$line[$end]
                if ($quote -ne [char]0) {
                    if ($escape) { $escape=$false } elseif ($ch -eq '\') { $escape=$true } elseif ($ch -eq $quote) { $quote=[char]0 }
                    continue
                }
                if ($ch -eq '"' -or $ch -eq "'") { $quote=$ch; continue }
                if ($ch -eq '(') { $depth++ }; if ($ch -eq ')') { $depth--; if (!$depth) { break } }
            }
            $args=($line.Substring($begin,$end-$begin) -split '\bfunction\b')[0]
            $literals=@([regex]::Matches($args,'"((?:\\.|[^"\\])*)"') | ForEach-Object { $_.Groups[1].Value })
            $numbers=@([regex]::Matches(($args -split 'function')[0],'(?<![\w.])[-+]?\d+(?:\.\d+)?(?![\w.])') | ForEach-Object { $_.Value })
            $ui.Add([pscustomobject][ordered]@{ source=$source; line=$lineNo; receiver=$m.Groups['receiver'].Value; method=$m.Groups['method'].Value; literal_arguments=($literals -join '|'); numeric_atoms=($numbers -join '|'); status='reference_only'; result='not_run'; limitation='Single-line literal scan; multiline calls expressions and inheritance require review; numbers are atoms not coordinates' })
        }
    }
    Track $source 'Literal Derma geometry controls and registration metadata' ($ui.Count-$before)
}
$ui | Export-Csv (Join-Path $Sheets 'menu_lua_reference.csv') -NoTypeInformation -Encoding UTF8
$scope | Sort-Object source | Export-Csv (Join-Path $Sheets 'spawn_menu_reference_scope.csv') -NoTypeInformation -Encoding UTF8
Write-Output "Extracted $($entries.Count) spawn definitions, $($ui.Count) UI metadata rows, and $($scope.Count) hashed source files. No Lua executed and no authored implementation states changed."
