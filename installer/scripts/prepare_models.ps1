<#
.SYNOPSIS
整理 Whisper Base 模型目录到安装包 staging 区。
#>
param(
	# 工作区根目录，默认自动回退到当前脚本上两级目录。
	[string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,

	# Whisper Base 模型根目录，未传入时使用工作区默认位置。
	[string]$ModelRoot = "",

	# staging 输出目录，未传入时使用 installer\staging。
	[string]$StageRoot = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 统一计算 staging 根目录。
function Get-NormalizedStageRoot {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$RequestedStageRoot
	)

	if ([string]::IsNullOrWhiteSpace($RequestedStageRoot)) {
		return (Join-Path $ResolvedWorkspaceRoot "installer\staging")
	}

	return $RequestedStageRoot
}

# 解析工作目录、模型目录和输出目录。
$resolvedWorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
$resolvedStageRoot = Get-NormalizedStageRoot -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedStageRoot $StageRoot
$resolvedModelRoot = if ([string]::IsNullOrWhiteSpace($ModelRoot)) {
	Join-Path $resolvedWorkspaceRoot "deepl_models\whisper_base"
} else {
	(Resolve-Path $ModelRoot).Path
}
$manifestPath = Join-Path $resolvedWorkspaceRoot "installer\manifests\whisper_base_required_files.txt"
$targetRoot = Join-Path $resolvedStageRoot "app\deepl_models\whisper_base"

if (-not (Test-Path $resolvedModelRoot)) {
	throw "缺少 Whisper Base 模型目录: $resolvedModelRoot"
}

if (Test-Path $targetRoot) {
	Remove-Item -Path $targetRoot -Recurse -Force
}

# 只按清单复制 Whisper Base 的必需文件，避免把无关文件一起打包。
New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null

$requiredFiles = Get-Content $manifestPath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
foreach ($fileName in $requiredFiles) {
	$sourcePath = Join-Path $resolvedModelRoot $fileName
	if (-not (Test-Path $sourcePath)) {
		throw "缺少 Whisper Base 模型文件: $sourcePath"
	}

	Copy-Item -Path $sourcePath -Destination (Join-Path $targetRoot $fileName) -Force
}

# 输出整理完成的结果路径。
Write-Host "已整理 Whisper Base 模型到 $targetRoot"
