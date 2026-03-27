param(
	[string]$WorkspaceRoot = "",
	[string]$StageRoot = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($WorkspaceRoot)) {
	$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
} else {
	$WorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
}

if ([string]::IsNullOrWhiteSpace($StageRoot)) {
	$StageRoot = Join-Path $WorkspaceRoot "installer\staging"
}

$baseModelRoot = Join-Path $WorkspaceRoot "deepl_models\whisper_base"
$largeV3ModelRoot = Join-Path $WorkspaceRoot "deepl_models\whisper\whisper-large-v3"
if (-not (Test-Path $largeV3ModelRoot)) {
	$largeV3ModelRoot = Join-Path $WorkspaceRoot "deepl_models\whisper"
}

$baseManifestPath = Join-Path $WorkspaceRoot "installer\manifests\whisper_base_required_files.txt"
$largeV3ManifestPath = Join-Path $WorkspaceRoot "installer\manifests\whisper_large_v3_required_files.txt"
$baseTargetRoot = Join-Path $StageRoot "app\deepl_models\whisper_base"
$largeV3TargetRoot = Join-Path $StageRoot "app\deepl_models\whisper\whisper-large-v3"

function Copy-RequiredFiles {
	param(
		[string]$SourceRoot,
		[string]$ManifestPath,
		[string]$TargetRoot,
		[string]$ModelLabel
	)

	if (-not (Test-Path $SourceRoot)) {
		throw "缺少 $ModelLabel 模型目录: $SourceRoot"
	}

	if (Test-Path $TargetRoot) {
		Remove-Item -Path $TargetRoot -Recurse -Force
	}

	New-Item -ItemType Directory -Path $TargetRoot -Force | Out-Null

	$requiredFiles = Get-Content $ManifestPath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
	foreach ($fileName in $requiredFiles) {
		$sourcePath = Join-Path $SourceRoot $fileName
		if (-not (Test-Path $sourcePath)) {
			throw "缺少 $ModelLabel 模型文件: $sourcePath"
		}

		Copy-Item -Path $sourcePath -Destination (Join-Path $TargetRoot $fileName) -Force
	}

	Write-Host "已整理 $ModelLabel 模型到 $TargetRoot"
}

Copy-RequiredFiles -SourceRoot $baseModelRoot -ManifestPath $baseManifestPath -TargetRoot $baseTargetRoot -ModelLabel "Whisper Base"
Copy-RequiredFiles -SourceRoot $largeV3ModelRoot -ManifestPath $largeV3ManifestPath -TargetRoot $largeV3TargetRoot -ModelLabel "Whisper Large V3"
