# Define helper functions

function Reload {
  resim publish --address $PACKAGE .
}

filter Get-Account-Address {
  $_ | Select-String 'Account address: (\w+)' | %{ $_.Matches.Groups[1].Value }
}

filter Get-Public-Key {
  $_ | Select-String 'Public key: (\w+)' | %{ $_.Matches.Groups[1].Value }
}

filter Get-Component {
  $_ | Select-String 'Component: (\w+)' | %{$_.Matches.Groups[1].Value }
}

filter Get-Resource-Def([String]$name) {
  $_ | Select-String "resource_def: (\w+), name: `"$name`"" | %{ $_.Matches.Groups[1].Value }
}

filter Get-Resource-Amount([String]$name) {
  $_ | Select-String "amount: (\d+)\..*, name: `"$name`"" | %{ $_.Matches.Groups[1].Value }
}

filter Get-New-Def([String]$name) {
  $_ | Select-String 'ResourceDef: (\w+)' | %{ $_.Matches.Groups[1].Value }
}

filter Get-Market-Price([String]$symbol) {
  $_ | Select-String "$symbol \|\s*(\d\.\d+).*" | %{ $_.Matches.Groups[1].Value }
}

function Wait-For-User([String]$message) {
  Write-Output ''
  Write-Host -NoNewLine "$message";
  $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
  Write-Output ''
  Write-Output ''
}

function Exit-Unless-Equal([String]$a, [String]$b, [String]$message) {
  if ("$a" -ne "$b") {
    Write-Output ''
    Write-Error $message
    Exit
  }
}

.\tests\steps.ps1
