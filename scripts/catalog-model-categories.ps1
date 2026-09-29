param([string]$Install = 'D:\SteamLibrary\steamapps\common\GarrysMod', [string]$Output = 'sheets/source_model_categories.csv')
$ErrorActionPreference = 'Stop'
$root = Join-Path $Install 'garrysmod/settings/spawnlist_default'
$lists = @{}
foreach ($file in Get-ChildItem $root -Filter '*.txt' | Sort-Object Name) {
    $text = Get-Content $file.FullName -Raw
    $id = [regex]::Match($text, '"id"\s+"([^"]+)"').Groups[1].Value
    $parent = [regex]::Match($text, '"parentid"\s+"([^"]+)"').Groups[1].Value
    $name = [regex]::Match($text, '"name"\s+"([^"]+)"').Groups[1].Value
    if (!$id -or !$name -or $lists.ContainsKey($id)) { throw "Invalid list metadata: $($file.Name)" }
    $lists[$id] = @{ Parent=$parent; Name=$name; Text=$text; File=$file.Name }
}
function CategoryPath([string]$id, [string[]]$seen = @()) {
    if ($seen -contains $id) { throw "Cyclic spawnlist parent: $id" }
    $list = $lists[$id]
    if (!$list) { throw "Unknown spawnlist parent: $id" }
    if ($list.Parent -eq '0') { return $list.Name }
    return "$(CategoryPath $list.Parent ($seen + $id)) / $($list.Name)"
}
$rows = foreach ($id in $lists.Keys | Sort-Object { [int]$_ }) {
    $list = $lists[$id]
    $category = CategoryPath $id
    $section = ''
    foreach ($block in [regex]::Matches($list.Text, '"\d+"\s*\{([^{}]*)\}')) {
        $body = $block.Groups[1].Value
        $type = [regex]::Match($body, '"type"\s+"([^"]+)"').Groups[1].Value
        if ($type -eq 'header') {
            $section = [regex]::Match($body, '"text"\s+"([^"]+)"').Groups[1].Value
        } elseif ($type -eq 'model') {
            $model = [regex]::Match($body, '(?m)^\s*"model"\s+"([^"]+)"').Groups[1].Value.ToLowerInvariant().Replace('\','/')
            if (!$model.StartsWith('models/') -or !$model.EndsWith('.mdl') -or $model.Contains('..')) { throw "Invalid model: $model" }
            $label = if ($section) { "$category / $section" } else { $category }
            [pscustomobject]@{ model=$model; category=$label; source="settings/spawnlist_default/$($list.File)"; line=1+([regex]::Matches($list.Text.Substring(0,$block.Index), "`n").Count) }
        }
    }
}
$rows = @($rows | Sort-Object model,category -Unique)
$rows | Export-Csv -NoTypeInformation -Encoding UTF8 $Output
Write-Output "Exported $($rows.Count) stock model-category memberships from $($lists.Count) lists. Reference metadata only; mounted availability is resolved at runtime."
