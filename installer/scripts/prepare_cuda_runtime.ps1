<#
.SYNOPSIS
整理程序依赖的 CUDA 运行时 DLL 到安装包 staging 区。
#>
param(
	# 工作区根目录，默认自动回退到当前脚本上两级目录。
	[string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,

	# CUDA 运行时 DLL 所在目录。
	[string]$CudaBinRoot = "",

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

# 解析 CUDA DLL 所在目录。
#
# 优先使用显式参数；
# 如果未传入，再尝试从 CUDA_PATH 推导。
function Resolve-CudaBinRoot {
	param([string]$RequestedCudaBinRoot)

	if (-not [string]::IsNullOrWhiteSpace($RequestedCudaBinRoot)) {
		return (Resolve-Path $RequestedCudaBinRoot).Path
	}

	if (-not [string]::IsNullOrWhiteSpace($env:CUDA_PATH)) {
		$defaultCudaBin = Join-Path $env:CUDA_PATH "bin"
		if (Test-Path $defaultCudaBin) {
			return (Resolve-Path $defaultCudaBin).Path
		}
	}

	throw "请通过 -CudaBinRoot 或环境变量 CUDA_PATH 提供 CUDA bin 目录"
}

# 解析工作目录、清单和输出目录。
$resolvedWorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
$resolvedStageRoot = Get-NormalizedStageRoot -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedStageRoot $StageRoot
$resolvedCudaBinRoot = Resolve-CudaBinRoot -RequestedCudaBinRoot $CudaBinRoot
$manifestPath = Join-Path $resolvedWorkspaceRoot "installer\manifests\cuda_runtime_dlls.txt"
$targetRoot = Join-Path $resolvedStageRoot "app\cuda\bin"

# 先清理旧目录，再按模式匹配复制 CUDA 运行时 DLL。
if (Test-Path $targetRoot) {
	Remove-Item -Path $targetRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null

$patterns = Get-Content $manifestPath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
foreach ($pattern in $patterns) {
	$matchedFiles = Get-ChildItem -Path (Join-Path $resolvedCudaBinRoot $pattern) -File -ErrorAction SilentlyContinue
	if (-not $matchedFiles) {
		throw "在 $resolvedCudaBinRoot 中找不到匹配的 CUDA 运行时 DLL: $pattern"
	}

	foreach ($matchedFile in $matchedFiles) {
		Copy-Item -Path $matchedFile.FullName -Destination (Join-Path $targetRoot $matchedFile.Name) -Force
	}
}

# 输出整理完成的结果路径。
Write-Host "已整理 CUDA 运行时 DLL 到 $targetRoot"
