param(
    [string]$RunId = "35948361280",
    [string]$Tag = "v4.69.0",
    [string]$NotesPath = "d:\work\Antigravity-Manager\.ai-memory\release\release-notes-v4.69.0.md"
)

Write-Host "Monitoring GitHub Actions release run $RunId for tag $Tag..."
for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Seconds 30
    $status = gh run view $RunId --json status,conclusion --jq '.status + " " + .conclusion' 2>$null
    Write-Host "[Watcher] Check $i - Status: $status"
    if ($status -like "*completed*") {
        Write-Host "[Watcher] Run $RunId completed. Waiting 10s then applying split release notes..."
        Start-Sleep -Seconds 10
        gh release edit $Tag --notes-file $NotesPath
        Write-Host "[Watcher] Clean split release notes re-applied successfully to $Tag!"
        break
    }
}
