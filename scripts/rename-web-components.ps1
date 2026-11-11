[CmdletBinding()]
param(
    [switch]$Apply
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$sourceRoot = Join-Path $repositoryRoot 'apps\web\src\lib'
$webSourceRoot = Join-Path $repositoryRoot 'apps\web\src'
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

if (-not (Test-Path -LiteralPath $sourceRoot -PathType Container)) {
    throw "Web source directory was not found: $sourceRoot"
}

$componentFiles = @(
    Get-ChildItem -LiteralPath $sourceRoot -Recurse -File -Filter '*.svelte' |
        Where-Object {
            $_.DirectoryName -notlike '*\paraglide\*' -and
            $_.BaseName -cmatch '[A-Z]'
        }
)

$moves = @(
    foreach ($file in $componentFiles) {
        $newBaseName = [regex]::Replace($file.BaseName, '([a-z0-9])([A-Z])', '$1-$2').ToLowerInvariant()
        $destination = Join-Path $file.DirectoryName ($newBaseName + $file.Extension)
        if ($destination -ne $file.FullName) {
            [pscustomobject]@{
                Source = $file.FullName
                Destination = $destination
                OldName = $file.Name
                NewName = [System.IO.Path]::GetFileName($destination)
            }
        }
    }
)

foreach ($move in $moves) {
    if ((Test-Path -LiteralPath $move.Destination) -and $move.Source -ne $move.Destination) {
        throw "Cannot rename '$($move.Source)': destination already exists at '$($move.Destination)'."
    }
}

if ($moves.Count -eq 0) {
    Write-Output 'All Svelte component filenames already use kebab-case.'
    exit 0
}

foreach ($move in $moves) {
    Write-Output ("{0} -> {1}" -f $move.Source, $move.Destination)
}

if (-not $Apply) {
    Write-Output 'Dry run only. Re-run with -Apply to rename files and update imports.'
    exit 0
}

$temporaryMoves = @()
$referenceBackups = @()
try {
    foreach ($move in $moves) {
        $temporaryPath = Join-Path (Split-Path -Parent $move.Source) (
            '.rename-' + [guid]::NewGuid().ToString('N') + $move.Source.Substring($move.Source.LastIndexOf('.'))
        )
        Move-Item -LiteralPath $move.Source -Destination $temporaryPath
        $temporaryMoves += [pscustomobject]@{
            Source = $move.Source
            Temporary = $temporaryPath
            Destination = $move.Destination
            Finalized = $false
        }
    }

    foreach ($move in $temporaryMoves) {
        Move-Item -LiteralPath $move.Temporary -Destination $move.Destination
        $move.Finalized = $true
    }

    $sources = Get-ChildItem -LiteralPath $webSourceRoot -Recurse -File |
        Where-Object {
            $_.Extension -in @('.svelte', '.ts', '.js') -and
            $_.FullName -notlike '*\lib\paraglide\*'
        }

    foreach ($source in $sources) {
        $content = [System.IO.File]::ReadAllText($source.FullName)
        $updated = $content
        foreach ($move in $moves) {
            $updated = $updated.Replace($move.OldName, $move.NewName)
        }
        if ($updated -cne $content) {
            $referenceBackups += [pscustomobject]@{
                Path = $source.FullName
                Content = $content
            }
            [System.IO.File]::WriteAllText($source.FullName, $updated, $utf8NoBom)
        }
    }
} catch {
    foreach ($backup in $referenceBackups) {
        [System.IO.File]::WriteAllText($backup.Path, $backup.Content, $utf8NoBom)
    }
    foreach ($move in $temporaryMoves) {
        if ($move.Finalized -and (Test-Path -LiteralPath $move.Destination)) {
            Move-Item -LiteralPath $move.Destination -Destination $move.Source -Force
        } elseif (Test-Path -LiteralPath $move.Temporary) {
            Move-Item -LiteralPath $move.Temporary -Destination $move.Source -Force
        }
    }
    throw
}

Write-Output ("Renamed {0} Svelte component file(s) and updated source references." -f $moves.Count)
