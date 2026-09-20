param(
  [Parameter(Mandatory = $true)][string]$Url,
  [string]$OutDir = ".\\benchmark-output",
  [int[]]$Connections = @(1, 2, 4, 8, 16),
  [int]$Repeats = 3,
  [string]$Hydra = "hydra",
  [string]$Aria2 = "aria2c"
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$cargo = Get-Command cargo -ErrorAction Stop
$apocalipse = Join-Path $OutDir "apocalipse-cli.exe"

Write-Host "Building Apocalipse benchmark CLI..."
& $cargo.Source build --release -p apocalipse-cli
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
Copy-Item ".\\target\\release\\apocalipse-cli.exe" $apocalipse -Force

$results = @()
foreach ($n in $Connections) {
  foreach ($repeat in 1..$Repeats) {
    $runs = @(
      @{ Name = "apocalipse"; Command = $apocalipse; Args = @("benchmark", $Url, (Join-Path $OutDir "apocalipse-$n-$repeat.bin"), "$n") },
      @{ Name = "aria2c"; Command = $Aria2; Args = @("-q", "-x$n", "-s$n", "--file-allocation=none", "--allow-overwrite=true", "-d", $OutDir, "-o", "aria2c-$n-$repeat.bin", $Url) },
      @{ Name = "hydra"; Command = $Hydra; Args = @("-q", "-x", "$n", "-O", (Join-Path $OutDir "hydra-$n-$repeat.bin"), $Url) }
    )

    foreach ($run in $runs) {
      Remove-Item (Join-Path $OutDir "$($run.Name)-$n-$repeat.bin") -Force -ErrorAction SilentlyContinue
      $watch = [System.Diagnostics.Stopwatch]::StartNew()
      $output = & $run.Command @($run.Args) 2>&1
      $exit = $LASTEXITCODE
      $watch.Stop()
      if ($exit -ne 0) {
        Write-Warning "$($run.Name) failed at $n connections (repeat $repeat): $output"
        continue
      }

      $bytes = 0
      $avg = 0
      if ($run.Name -eq "apocalipse") {
        try {
          $json = ($output -join [Environment]::NewLine) | ConvertFrom-Json
          $bytes = [int64]$json.bytes
          $avg = [int64]$json.averageBytesPerSecond
        } catch {
          Write-Warning "Could not parse Apocalipse JSON output"
        }
      } else {
        $path = Join-Path $OutDir "$($run.Name)-$n-$repeat.bin"
        if (Test-Path $path) { $bytes = (Get-Item $path).Length }
        if ($watch.Elapsed.TotalSeconds -gt 0) {
          $avg = [int64]($bytes / $watch.Elapsed.TotalSeconds)
        }
      }

      $results += [pscustomobject]@{
        engine = $run.Name
        connections = $n
        repeat = $repeat
        elapsed_ms = [int64]$watch.Elapsed.TotalMilliseconds
        bytes = $bytes
        average_bytes_per_second = $avg
      }
      Write-Host ("{0,-11} x{1,-2} rep {2}: {3:N1} MiB/s" -f $run.Name, $n, $repeat, ($avg / 1MB))
    }
  }
}

$csv = Join-Path $OutDir "http-engine-benchmark.csv"
$results | Export-Csv -NoTypeInformation -Encoding UTF8 $csv
$results | Group-Object engine,connections | ForEach-Object {
  $g = $_.Group
  [pscustomobject]@{
    engine = $g[0].engine
    connections = $g[0].connections
    runs = $g.Count
    avg_mib_s = [Math]::Round((($g | Measure-Object average_bytes_per_second -Average).Average / 1MB), 2)
    avg_elapsed_ms = [Math]::Round(($g | Measure-Object elapsed_ms -Average).Average, 0)
  }
} | Sort-Object connections,engine | Format-Table -AutoSize

Write-Host "CSV: $csv"
