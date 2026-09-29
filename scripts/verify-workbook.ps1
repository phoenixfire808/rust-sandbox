param(
    [string]$Workbook = "$PSScriptRoot\..\local\gmod-research.xlsx",
    [string]$Sheets = "$PSScriptRoot\..\sheets",
    [string]$Inventory = "$PSScriptRoot\..\local\inventory-20260929"
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$expected = @{}
foreach ($dir in @($Sheets, $Inventory)) {
    foreach ($file in Get-ChildItem -LiteralPath $dir -Filter *.csv) {
        if ($expected.ContainsKey($file.BaseName)) { throw "Duplicate worksheet name $($file.BaseName)" }
        $expected[$file.BaseName] = (Import-Csv -LiteralPath $file.FullName | Measure-Object).Count
    }
}
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $Workbook).Path)
try {
    $stream = $zip.GetEntry('xl/workbook.xml').Open()
    try { $book = New-Object System.Xml.XmlDocument; $book.Load($stream) } finally { $stream.Dispose() }
    $ns = New-Object System.Xml.XmlNamespaceManager($book.NameTable)
    $ns.AddNamespace('s','http://schemas.openxmlformats.org/spreadsheetml/2006/main')
    $sheetsInBook = $book.SelectNodes('//s:sheet',$ns)
    if ($sheetsInBook.Count -ne $expected.Count) { throw 'Workbook worksheet count differs from CSV inputs' }
    $total = 0
    $index = 0
    foreach ($sheet in $sheetsInBook) {
        $index++
        $name = $sheet.GetAttribute('name')
        if (-not $expected.ContainsKey($name)) { throw "Unexpected worksheet $name" }
        $entry = $zip.GetEntry("xl/worksheets/sheet$index.xml")
        if ($null -eq $entry) { throw "Missing worksheet XML $index" }
        $stream = $entry.Open()
        $settings = New-Object System.Xml.XmlReaderSettings
        $settings.DtdProcessing = [System.Xml.DtdProcessing]::Prohibit
        $settings.XmlResolver = $null
        $reader = [System.Xml.XmlReader]::Create($stream,$settings)
        try {
            $rows = 0
            $formulas = 0
            while ($reader.Read()) {
                if ($reader.NodeType -eq [System.Xml.XmlNodeType]::Element) {
                    if ($reader.LocalName -eq 'row') { $rows++ }
                    if ($reader.LocalName -eq 'f') { $formulas++ }
                }
            }
        } finally { $reader.Dispose(); $stream.Dispose() }
        if ($rows -ne ($expected[$name]+1)) { throw "$name row mismatch: $rows vs $($expected[$name]+1) including header" }
        if ($formulas -ne 0) { throw "Unexpected formulas in $name" }
        Write-Output "PASS $name : $($rows-1) data rows, no formulas"
        $total += $rows-1
    }
    Write-Output "WORKBOOK_VALIDATED: $index sheets, $total data rows"
} finally { $zip.Dispose() }
