param(
    [string]$Install = 'D:\SteamLibrary\steamapps\common\GarrysMod',
    [string]$Sheets = (Join-Path $PSScriptRoot '..\sheets')
)
$ErrorActionPreference = 'Stop'
# Research inventory only. Never executes Lua, changes Steam files or marks acceptance passed.
function Fields([string]$body) {
    $values = @{}
    foreach ($pair in [regex]::Matches($body, '"([^"\r\n]+)"\s+"([^"\r\n]*)"')) {
        $values[$pair.Groups[1].Value] = $pair.Groups[2].Value
    }
    return $values
}
$controls = @(
    foreach ($file in Get-ChildItem -LiteralPath (Join-Path $Install 'garrysmod\resource') -Filter 'Options*.res' | Sort-Object Name) {
        $text = [IO.File]::ReadAllText($file.FullName)
        foreach ($block in [regex]::Matches($text, '"([^"\r\n]+)"\s*\{([^{}]*)\}', 'Singleline')) {
            $v = Fields $block.Groups[2].Value
            if (!$v.ContainsKey('ControlName')) { continue }
            [pscustomobject][ordered]@{
                source = 'resource/' + $file.Name
                control_id = $block.Groups[1].Value
                kind = $v['ControlName']
                xpos_raw = $v['xpos']; ypos_raw = $v['ypos']
                width_raw = $v['wide']; height_raw = $v['tall']
                label_token = $v['labelText']; command = $v['command']
                visible = $v['visible']; enabled = $v['enabled']; tab_order = $v['tabPosition']
                status = 'reference_only'; result = 'not_run'
                remaining = 'Resolve native proportional coordinates inheritance localization runtime options and behavior'
            }
        }
    }
)
$settings = @(
    $source = Join-Path $Install 'garrysmod\gamemodes\sandbox\sandbox.txt'
    foreach ($block in [regex]::Matches([IO.File]::ReadAllText($source), '(?m)^\s*(\d+)\s*\{([^{}]*)\}', 'Singleline')) {
        $v = Fields $block.Groups[2].Value
        [pscustomobject][ordered]@{
            order = $block.Groups[1].Value; name = $v['name']; kind = $v['type']
            label_token = $v['text']; default = $v['default']; singleplayer = $v['singleplayer']
            replicate = $v['replicate']; dontcreate = $v['dontcreate']; help = $v['help']
            source = 'gamemodes/sandbox/sandbox.txt'; status = 'reference_only'; result = 'not_run'
            remaining = 'Implement control binding actual runtime effect persistence and reference acceptance'
        }
    }
)
$controls | Export-Csv -LiteralPath (Join-Path $Sheets 'native_menu_controls.csv') -NoTypeInformation -Encoding UTF8
$settings | Export-Csv -LiteralPath (Join-Path $Sheets 'newgame_reference_options.csv') -NoTypeInformation -Encoding UTF8
Write-Output ('Cataloged ' + $controls.Count + ' literal native option controls and ' + $settings.Count + ' Sandbox new-game settings. Dynamic and inherited controls are not inferred.')

# Record factual control metadata, not copies of proprietary scripts or artwork.
$htmlRoot = Join-Path $Install 'garrysmod\html'
$htmlFiles = @((Get-Item (Join-Path $htmlRoot 'menu.html'))) + @(Get-ChildItem (Join-Path $htmlRoot 'template') -Filter *.html -Recurse | Sort-Object FullName)
$elements = @(
    foreach ($file in $htmlFiles) {
        $body = [IO.File]::ReadAllText($file.FullName)
        # Preserve line positions while excluding commented-out controls.
        $body = [regex]::Replace($body, '(?s)<!--.*?-->', { param($m) [regex]::Replace($m.Value, '[^\r\n]', ' ') })
        foreach ($tag in [regex]::Matches($body, '<(?<tag>[a-z][\w-]*)\b(?<attrs>(?:"[^"]*"|''[^'']*''|[^''">])*)>', 'IgnoreCase')) {
            $a = @{}
            foreach ($attr in [regex]::Matches($tag.Groups['attrs'].Value, '(?<key>[\w-]+)\s*=\s*(?:"(?<value>[^"]*)"|''(?<value>[^'']*)''|(?<value>[^\s>]+))')) {
                $a[$attr.Groups['key'].Value.ToLowerInvariant()] = $attr.Groups['value'].Value
            }
            $events = @('ng-click','ng-dblclick','ng-change','onclick','onchange') | Where-Object { $a.ContainsKey($_) }
            if (!$events.Count -and !$a.ContainsKey('ng-model') -and !$a.ContainsKey('ng-localize') -and !$a.ContainsKey('href') -and !$a.ContainsKey('ng-href') -and $tag.Groups['tag'].Value -notin @('input','button','select','textarea')) { continue }
            $handlers = @(
                foreach ($event in $events) {
                    foreach ($call in [regex]::Matches($a[$event], '([A-Za-z_$][\w$]*)\s*\(')) { $call.Groups[1].Value }
                }
            ) | Select-Object -Unique
            [pscustomobject][ordered]@{
                source = 'html/' + $file.FullName.Substring($htmlRoot.Length + 1).Replace('\','/')
                line = 1 + ([regex]::Matches($body.Substring(0,$tag.Index), '\n')).Count
                tag = $tag.Groups['tag'].Value; control_id = $a['id']; kind = $a['type']
                label_token = $a['ng-localize']; events = $events -join '|'; handlers = $handlers -join '|'
                binding = $a['ng-model']; route = @($a['href'], $a['ng-href']) -join ''
                conditional = [bool]($a.ContainsKey('ng-show') -or $a.ContainsKey('ng-hide') -or $a.ContainsKey('ng-if'))
                repeated = $a.ContainsKey('ng-repeat'); status = 'reference_only'; result = 'not_run'
                remaining = 'Match layout state visibility input behavior and backend; repeated ancestors and generated controls require separate runtime inventory'
            }
        }
    }
)
$elements | Export-Csv -LiteralPath (Join-Path $Sheets 'menu_html_controls.csv') -NoTypeInformation -Encoding UTF8

$geometry = @(
    $cssFiles = @(Get-ChildItem (Join-Path $htmlRoot 'css') -Filter *.css -Recurse | Sort-Object FullName)
    foreach ($file in $cssFiles) {
        $body = [IO.File]::ReadAllText($file.FullName)
        $body = [regex]::Replace($body, '(?s)/\*.*?\*/', { param($m) [regex]::Replace($m.Value, '[^\r\n]', ' ') })
        foreach ($rule in [regex]::Matches($body, '(?<selector>[^{}]+)\{(?<body>[^{}]*)\}')) {
            foreach ($declaration in [regex]::Matches($rule.Groups['body'].Value, '(?<property>[\w-]+)\s*:\s*(?<value>[^;{}]+)')) {
                $property = $declaration.Groups['property'].Value
                if ($property -notmatch '^(position|top|left|right|bottom|width|height|min-width|min-height|max-width|max-height|padding.*|margin.*|font.*|line-height|letter-spacing|text-align|display|overflow.*|z-index|border-radius|color|background-color|box-sizing|vertical-align|white-space)$') { continue }
                [pscustomobject][ordered]@{
                    source = 'html/' + $file.FullName.Substring($htmlRoot.Length + 1).Replace('\','/')
                    line = 1 + ([regex]::Matches($body.Substring(0,$rule.Index), '\n')).Count
                    selector = [regex]::Replace($rule.Groups['selector'].Value.Trim(), '\s+', ' ')
                    property = $property; value = $declaration.Groups['value'].Value.Trim()
                    status = 'reference_only'; result = 'not_run'
                    remaining = 'Resolve cascade and enclosing media conditions at source line; compare equivalent Bevy layout at target viewport'
                }
            }
        }
    }
)
$geometry | Export-Csv -LiteralPath (Join-Path $Sheets 'menu_css_geometry.csv') -NoTypeInformation -Encoding UTF8
Write-Output ('Cataloged ' + $elements.Count + ' HTML control/label definitions and ' + $geometry.Count + ' CSS geometry/style declarations. This is literal-source coverage, not runtime or visual acceptance.')
