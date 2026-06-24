$identifier = (Get-Content "$PSScriptRoot\..\src-tauri\tauri.conf.json" | ConvertFrom-Json).identifier
$db = "$env:APPDATA\$identifier\time-tracker.db"

if (-not (Test-Path $db)) {
    Write-Error "Database not found at $db - run the app at least once first."
    exit 1
}

& sqlite3 @('-cmd', '.headers on', '-cmd', '.mode table', $db)
