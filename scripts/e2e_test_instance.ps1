$ErrorActionPreference = "Continue"

Set-Location -Path "$PSScriptRoot\..\src-tauri"

Write-Host "Creating test instance..."
cargo run --bin agm -- instances create "test_e2e_1"

Write-Host "Starting a running prompt on this project..."
cargo run --bin agm -- prompt . "test e2e prompt execution"

Write-Host "Waiting 2 seconds..."
Start-Sleep -Seconds 2

Write-Host "Switching profile to test_e2e_1..."
cargo run --bin agm -- ff "test_e2e_1"

Write-Host "Waiting 5 seconds for background dispatch..."
Start-Sleep -Seconds 5

Write-Host "Verifying running prompts in test_e2e_1..."
cargo run --bin agm -- prompts ls

Write-Host "Removing test instance..."
cargo run --bin agm -- instances rm "test_e2e_1" --force

Write-Host "Done!"
