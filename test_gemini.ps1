# Quick test script for Gemini API
# Run: .\test_gemini.ps1

$API_KEY = "AIzaSyAEk3cpzrVVgGGn6GglZ4B_ze1XMXQb8cw"
$MODEL = "gemini-2.0-flash"

$url = "https://generativelanguage.googleapis.com/v1beta/models/${MODEL}:generateContent?key=${API_KEY}"

$body = @{
    contents = @(
        @{
            parts = @(
                @{
                    text = @"
Extract security metadata from this document as valid JSON.

Document: This is a classified intelligence report regarding cyber operations in Eastern Europe. The document covers signals intelligence gathered from multiple sources.

Output valid JSON with these fields:
- title: string
- description: string
- domain: one of INTEL, CYBER, OPERATIONS
- tags: array of strings

Output JSON only:
"@
                }
            )
        }
    )
    generationConfig = @{
        temperature = 0.2
        maxOutputTokens = 1024
    }
} | ConvertTo-Json -Depth 10

Write-Host "Testing Gemini API with model: $MODEL" -ForegroundColor Cyan
Write-Host ""

try {
    $response = Invoke-RestMethod -Uri $url -Method POST -Body $body -ContentType "application/json"

    Write-Host "SUCCESS!" -ForegroundColor Green
    Write-Host ""
    Write-Host "LLM Response:" -ForegroundColor Yellow
    Write-Host $response.candidates[0].content.parts[0].text
}
catch {
    Write-Host "ERROR:" -ForegroundColor Red
    Write-Host $_.Exception.Message
    if ($_.ErrorDetails.Message) {
        Write-Host $_.ErrorDetails.Message
    }
}
