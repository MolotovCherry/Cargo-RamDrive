function _Get-FNVHash {
    param(
        [string]$InputString
    )

    # Initial prime and offset chosen for 32-bit output
    # See https://en.wikipedia.org/wiki/Fowler–Noll–Vo_hash_function
    [uint32]$FNVPrime = 16777619
    [uint32]$offset = 2166136261

    # Convert string to byte array, may want to change based on input collation
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($InputString)

    # Copy offset as initial hash value
    [uint32]$hash = $offset

    foreach($octet in $bytes)
    {
        # Apply XOR, multiply by prime and mod with max output size
        $hash = $hash -bxor $octet
        $hash = $hash * $FNVPrime % [System.Math]::Pow(2,32)
    }
    return $hash
}

$rustBaseDir = "$env:TMP\rust"

function _Get-RustTmpDir {
    $currentDir = Get-Location
    $leaf = Split-Path (Get-Location) -Leaf

    # protect against the following
    # C:\ == C:\ -> $rustBaseDir\C:\-1032342480
    if ($currentDir.ToString() -eq $leaf) {
        $driveLetter = $currentDir.Drive.ToString()
        return "$rustBaseDir\$driveLetter-$(_Get-FNVHash $currentDir)"
    }

    return "$rustBaseDir\$leaf-$(_Get-FNVHash $currentDir)"
}


function Set-Location {
    [CmdletBinding(DefaultParameterSetName='Path', HelpUri='https://go.microsoft.com/fwlink/?LinkID=2097049')]
    param(
        [Parameter(ParameterSetName='Path', Position=0, ValueFromPipeline=$true, ValueFromPipelineByPropertyName=$true)]
        [string]
        ${Path},

        [Parameter(ParameterSetName='LiteralPath', Mandatory=$true, ValueFromPipelineByPropertyName=$true)]
        [Alias('PSPath','LP')]
        [string]
        ${LiteralPath},

        [switch]
        ${PassThru},

        [Parameter(ParameterSetName='Stack', ValueFromPipelineByPropertyName=$true)]
        [string]
        ${StackName})


    dynamicparam
    {
        try {
            $targetCmd = $ExecutionContext.InvokeCommand.GetCommand('Microsoft.PowerShell.Management\Set-Location', [System.Management.Automation.CommandTypes]::Cmdlet, $PSBoundParameters)
            $dynamicParams = @($targetCmd.Parameters.GetEnumerator() | Microsoft.PowerShell.Core\Where-Object { $_.Value.IsDynamic })
            if ($dynamicParams.Length -gt 0)
            {
                $paramDictionary = [Management.Automation.RuntimeDefinedParameterDictionary]::new()
                foreach ($param in $dynamicParams)
                {
                    $param = $param.Value

                    if(-not $MyInvocation.MyCommand.Parameters.ContainsKey($param.Name))
                    {
                        $dynParam = [Management.Automation.RuntimeDefinedParameter]::new($param.Name, $param.ParameterType, $param.Attributes)
                        $paramDictionary.Add($param.Name, $dynParam)
                    }
                }

                return $paramDictionary
            }
        } catch {
            throw
        }
    }

    begin
    {
        try {
            $outBuffer = $null
            if ($PSBoundParameters.TryGetValue('OutBuffer', [ref]$outBuffer))
            {
                $PSBoundParameters['OutBuffer'] = 1
            }

            $wrappedCmd = $ExecutionContext.InvokeCommand.GetCommand('Microsoft.PowerShell.Management\Set-Location', [System.Management.Automation.CommandTypes]::Cmdlet)
            $scriptCmd = {& $wrappedCmd @PSBoundParameters }

            $steppablePipeline = $scriptCmd.GetSteppablePipeline($myInvocation.CommandOrigin)
            $steppablePipeline.Begin($PSCmdlet)
        } catch {
            throw
        }
    }

    process
    {
        try {
            $steppablePipeline.Process($_)
        } catch {
            throw
        }
    }

    end
    {
        try {
            $steppablePipeline.End()
            $env:CARGO_BUILD_TARGET_DIR = _Get-RustTmpDir
        } catch {
            throw
        }
    }
    <#

    .ForwardHelpTargetName Microsoft.PowerShell.Management\Set-Location
    .ForwardHelpCategory Cmdlet

    #>
}

function Rust-TargetDir {
    $location = $env:CARGO_BUILD_TARGET_DIR

    if (Test-Path -Path $location) {
        explorer $location
    } else {
        Write-Host "No Rust target folder exists for this directory"
    }
}

function _Clean-Rust {
    param(
        [string]$location
    )

    if (Test-Path -Path $location) {
        Remove-Item $location -Recurse -Force
        Write-Host "Cleaned out temp rust folder: $location"
    } else {
        Write-Host "Temp Rust folder already clean"
    }
}

function Clean-Rust {
    _Clean-Rust "$rustBaseDir\*"
}

# Set the initial value
$env:CARGO_BUILD_TARGET_DIR = _Get-RustTmpDir
