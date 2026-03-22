<#
.SYNOPSIS
整理 GStreamer 运行时目录到安装包 staging 区。
#>
param(
	# 工作区根目录，默认自动回退到当前脚本上两级目录。
	[string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,

	# GStreamer Runtime 根目录。
	[string]$GStreamerRoot = $env:GSTREAMER_RUNTIME_ROOT,

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

# 按清单复制 GStreamer 运行时目录。
function Copy-GStreamerRuntime {
	param(
		[string]$SourceRoot,
		[string]$TargetRoot,
		[string[]]$RelativeRoots
	)

	foreach ($relativeRoot in $RelativeRoots) {
		$sourcePath = Join-Path $SourceRoot $relativeRoot
		if (-not (Test-Path $sourcePath)) {
			throw "缺少 GStreamer 运行时目录: $sourcePath"
		}

		$targetPath = Join-Path $TargetRoot $relativeRoot
		New-Item -ItemType Directory -Path $targetPath -Force | Out-Null

		$filterPatterns = switch ($relativeRoot) {
			"bin" { @("*.dll") }
			"lib\gstreamer-1.0" { @("*.dll") }
			"libexec\gstreamer-1.0" { @("*.exe", "*.dll") }
			default { @("*") }
		}

		foreach ($filterPattern in $filterPatterns) {
			Get-ChildItem -Path (Join-Path $sourcePath $filterPattern) -File -ErrorAction SilentlyContinue |
				ForEach-Object {
					Copy-Item -Path $_.FullName -Destination (Join-Path $targetPath $_.Name) -Force
				}
		}
	}
}

# 解析工作目录、清单和输出目录。
$resolvedWorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
$resolvedStageRoot = Get-NormalizedStageRoot -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedStageRoot $StageRoot
$manifestPath = Join-Path $resolvedWorkspaceRoot "installer\manifests\gstreamer_runtime_roots.txt"
$targetRoot = Join-Path $resolvedStageRoot "app\gstreamer"

if ([string]::IsNullOrWhiteSpace($GStreamerRoot)) {
	throw "请通过 -GStreamerRoot 或环境变量 GSTREAMER_RUNTIME_ROOT 提供 GStreamer Runtime 根目录"
}

$resolvedGStreamerRoot = (Resolve-Path $GStreamerRoot).Path
$relativeRoots = Get-Content $manifestPath | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

# 先清理旧目录，再复制新的 GStreamer Runtime。
if (Test-Path $targetRoot) {
	Remove-Item -Path $targetRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null
Copy-GStreamerRuntime -SourceRoot $resolvedGStreamerRoot -TargetRoot $targetRoot -RelativeRoots $relativeRoots

# 输出整理完成的结果路径。
Write-Host "已整理 GStreamer Runtime 到 $targetRoot"
