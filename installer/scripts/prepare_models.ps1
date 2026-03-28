param(
	[string]$WorkspaceRoot = "",
	[string]$StageRoot = "",
	[string]$WhisperBaseModelRoot = "",
	[string]$WhisperLargeV3ModelRoot = "",
	[string]$WhisperLargeV3PartsRoot = ""
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

if ([string]::IsNullOrWhiteSpace($WhisperBaseModelRoot)) {
	$WhisperBaseModelRoot = Join-Path $WorkspaceRoot "deepl_models\whisper_base"
} else {
	$WhisperBaseModelRoot = (Resolve-Path $WhisperBaseModelRoot).Path
}

if ([string]::IsNullOrWhiteSpace($WhisperLargeV3ModelRoot)) {
	$preferredLargeV3Root = Join-Path $WorkspaceRoot "deepl_models\whisper\whisper-large-v3"
	if (Test-Path $preferredLargeV3Root) {
		$WhisperLargeV3ModelRoot = $preferredLargeV3Root
	} else {
		$WhisperLargeV3ModelRoot = Join-Path $WorkspaceRoot "deepl_models\whisper"
	}
} else {
	$WhisperLargeV3ModelRoot = (Resolve-Path $WhisperLargeV3ModelRoot).Path
}

if (-not [string]::IsNullOrWhiteSpace($WhisperLargeV3PartsRoot)) {
	$WhisperLargeV3PartsRoot = (Resolve-Path $WhisperLargeV3PartsRoot).Path
}

$baseManifestPath = Join-Path $WorkspaceRoot "installer\manifests\whisper_base_required_files.txt"
$largeV3ManifestPath = Join-Path $WorkspaceRoot "installer\manifests\whisper_large_v3_required_files.txt"
$baseTargetRoot = Join-Path $StageRoot "app\deepl_models\whisper_base"
$largeV3TargetRoot = Join-Path $StageRoot "app\deepl_models\whisper\whisper-large-v3"

# 复制 Large V3 权重分片，并统一重命名为运行时约定文件名。
function Copy-WhisperLargeV3Parts {
	param(
		[string]$PartsRoot,
		[string]$TargetRoot
	)

	$partFiles = Get-ChildItem -Path $PartsRoot -Filter "whisper-large-v3-model.part*" -File | Sort-Object Name
	if (-not $partFiles) {
		throw "Whisper Large V3 分片目录为空: $PartsRoot"
	}

	foreach ($partFile in $partFiles) {
		if ($partFile.Name -notmatch '\.part(\d+)$') {
			throw "无法识别 Whisper Large V3 分片文件名: $($partFile.Name)"
		}

		$targetFileName = "model.safetensors.part$($Matches[1])"
		Copy-Item -Path $partFile.FullName -Destination (Join-Path $TargetRoot $targetFileName) -Force
	}
}

function Copy-RequiredFiles {
	param(
		[string]$SourceRoot,
		[string]$ManifestPath,
		[string]$TargetRoot,
		[string]$ModelLabel,
		[string]$PartsRoot = ""
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
		if ($fileName -eq "model.safetensors" -and -not [string]::IsNullOrWhiteSpace($PartsRoot)) {
			Copy-WhisperLargeV3Parts -PartsRoot $PartsRoot -TargetRoot $TargetRoot
			continue
		}

		$sourcePath = Join-Path $SourceRoot $fileName
		if (-not (Test-Path $sourcePath)) {
			throw "缺少 $ModelLabel 模型文件: $sourcePath"
		}

		Copy-Item -Path $sourcePath -Destination (Join-Path $TargetRoot $fileName) -Force
	}

	Write-Host "已整理 $ModelLabel 模型到 $TargetRoot"
}

Copy-RequiredFiles -SourceRoot $WhisperBaseModelRoot -ManifestPath $baseManifestPath -TargetRoot $baseTargetRoot -ModelLabel "Whisper Base"
Copy-RequiredFiles -SourceRoot $WhisperLargeV3ModelRoot -ManifestPath $largeV3ManifestPath -TargetRoot $largeV3TargetRoot -ModelLabel "Whisper Large V3" -PartsRoot $WhisperLargeV3PartsRoot
