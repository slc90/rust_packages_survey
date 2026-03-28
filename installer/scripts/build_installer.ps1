<#
.SYNOPSIS
构建安装包 staging 目录并调用 WiX 生成 MSI。
.DESCRIPTION
该脚本负责串联 release 构建、配置文件复制、GStreamer/Whisper/CUDA 运行时整理，
以及 WiX v5 打包命令。默认会执行构建流程，但不会自动运行安装包。
#>
param(
	# 工作区根目录，默认自动回退到当前脚本上两级目录。
	[string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,

	# 构建配置，当前默认使用 release。
	[string]$Configuration = "release",

	# staging 输出目录，未传入时使用 installer\staging。
	[string]$StageRoot = "",

	# MSI 输出目录，未传入时使用 installer\dist。
	[string]$OutputRoot = "",

	# GStreamer Runtime 根目录。
	[string]$GStreamerRoot = "",

	# CUDA 运行时 DLL 所在目录。
	[string]$CudaBinRoot = "",

	# Whisper Base 模型目录。
	[string]$WhisperBaseModelRoot = "",

	# Whisper Large V3 模型目录。
	[string]$WhisperLargeV3ModelRoot = "",

	# Whisper Large V3 模型分片目录。
	[string]$WhisperLargeV3PartsRoot = "",

	# 是否跳过 cargo release 构建。
	[switch]$SkipCargoBuild,

	# 是否只准备 staging 而不生成 MSI。
	[switch]$SkipMsiBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 统一处理可选路径参数，补齐默认目录。
function Get-NormalizedPath {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$RequestedPath,
		[string]$DefaultRelativePath
	)

	if ([string]::IsNullOrWhiteSpace($RequestedPath)) {
		return (Join-Path $ResolvedWorkspaceRoot $DefaultRelativePath)
	}

	return $RequestedPath
}

# 执行外部命令，并在失败时直接终止脚本。
function Invoke-Step {
	param(
		[string]$FilePath,
		[string[]]$ArgumentList
	)

	Write-Host "执行: $FilePath $($ArgumentList -join ' ')"
	& $FilePath @ArgumentList
	if ($LASTEXITCODE -ne 0) {
		throw "命令执行失败: $FilePath"
	}
}

# 在当前 PowerShell 进程中直接执行子脚本，并保留命名参数绑定。
function Invoke-ChildScript {
	param(
		[string]$ScriptPath,
		[hashtable]$Parameters
	)

	$displayArguments = $Parameters.GetEnumerator() | ForEach-Object {
		"-$($_.Key) $($_.Value)"
	}
	Write-Host "执行: $ScriptPath $($displayArguments -join ' ')"
	& $ScriptPath @Parameters
}

# 检查本机是否已安装 WiX v5 命令行工具。
function Assert-WixCommandAvailable {
	$wixCommand = Get-Command wix -ErrorAction SilentlyContinue
	if ($null -eq $wixCommand) {
		throw "找不到 wix 命令。请先执行: dotnet tool install --global wix --version 5.0.2"
	}
}

# 读取 installer 本地路径配置。
#
# 该配置文件只保存当前机器上的资源路径，
# 用于减少每次执行脚本时重复传参。
function Get-InstallerLocalPaths {
	param([string]$ResolvedWorkspaceRoot)

	$localPathsFile = Join-Path $ResolvedWorkspaceRoot "installer\local_paths.ps1"
	if (-not (Test-Path $localPathsFile)) {
		return @{}
	}

	. $localPathsFile
	if ($null -eq $InstallerLocalPaths) {
		throw "installer\local_paths.ps1 缺少 `$InstallerLocalPaths 配置"
	}

	return $InstallerLocalPaths
}

# 解析 GStreamer Runtime 根目录。
#
# 优先级依次为：
# 1. 命令行参数
# 2. installer/local_paths.ps1
# 3. 环境变量 GSTREAMER_RUNTIME_ROOT
function Resolve-GStreamerRoot {
	param(
		[string]$RequestedGStreamerRoot,
		[hashtable]$InstallerLocalPaths
	)

	if (-not [string]::IsNullOrWhiteSpace($RequestedGStreamerRoot)) {
		return $RequestedGStreamerRoot
	}

	$configuredPath = $InstallerLocalPaths["GStreamerRoot"]
	if (-not [string]::IsNullOrWhiteSpace($configuredPath)) {
		return $configuredPath
	}

	if (-not [string]::IsNullOrWhiteSpace($env:GSTREAMER_RUNTIME_ROOT)) {
		return $env:GSTREAMER_RUNTIME_ROOT
	}

	throw "请通过 installer\local_paths.ps1、-GStreamerRoot 或环境变量 GSTREAMER_RUNTIME_ROOT 提供 GStreamer Runtime 根目录"
}

# 解析 CUDA 运行时 DLL 目录。
#
# 优先级依次为：
# 1. 命令行参数
# 2. installer/local_paths.ps1
# 3. 环境变量 CUDA_PATH 推导出的目录
function Resolve-CudaBinRoot {
	param(
		[string]$RequestedCudaBinRoot,
		[hashtable]$InstallerLocalPaths
	)

	if (-not [string]::IsNullOrWhiteSpace($RequestedCudaBinRoot)) {
		return $RequestedCudaBinRoot
	}

	$configuredPath = $InstallerLocalPaths["CudaBinRoot"]
	if (-not [string]::IsNullOrWhiteSpace($configuredPath)) {
		return $configuredPath
	}

	if (-not [string]::IsNullOrWhiteSpace($env:CUDA_PATH)) {
		$defaultCudaBin = Join-Path $env:CUDA_PATH "bin"
		if (Test-Path $defaultCudaBin) {
			return $defaultCudaBin
		}

		$defaultCudaBinX64 = Join-Path $env:CUDA_PATH "bin\x64"
		if (Test-Path $defaultCudaBinX64) {
			return $defaultCudaBinX64
		}
	}

	throw "请通过 installer\local_paths.ps1、-CudaBinRoot 或环境变量 CUDA_PATH 提供 CUDA 运行时 DLL 目录"
}

# 解析 Whisper Base 模型目录。
#
# 优先级依次为：
# 1. 命令行参数
# 2. installer/local_paths.ps1
# 3. 工作区默认目录
function Resolve-WhisperBaseModelRoot {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$RequestedWhisperBaseModelRoot,
		[hashtable]$InstallerLocalPaths
	)

	if (-not [string]::IsNullOrWhiteSpace($RequestedWhisperBaseModelRoot)) {
		return (Resolve-Path $RequestedWhisperBaseModelRoot).Path
	}

	$configuredPath = $InstallerLocalPaths["WhisperBaseModelRoot"]
	if (-not [string]::IsNullOrWhiteSpace($configuredPath)) {
		return (Resolve-Path $configuredPath).Path
	}

	return (Join-Path $ResolvedWorkspaceRoot "deepl_models\whisper_base")
}

# 解析 Whisper Large V3 模型目录。
#
# 优先级依次为：
# 1. 命令行参数
# 2. installer/local_paths.ps1
# 3. 工作区默认目录
function Resolve-WhisperLargeV3ModelRoot {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$RequestedWhisperLargeV3ModelRoot,
		[hashtable]$InstallerLocalPaths
	)

	if (-not [string]::IsNullOrWhiteSpace($RequestedWhisperLargeV3ModelRoot)) {
		return (Resolve-Path $RequestedWhisperLargeV3ModelRoot).Path
	}

	$configuredPath = $InstallerLocalPaths["WhisperLargeV3ModelRoot"]
	if (-not [string]::IsNullOrWhiteSpace($configuredPath)) {
		return (Resolve-Path $configuredPath).Path
	}

	$preferredLargeV3Root = Join-Path $ResolvedWorkspaceRoot "deepl_models\whisper\whisper-large-v3"
	if (Test-Path $preferredLargeV3Root) {
		return $preferredLargeV3Root
	}

	return (Join-Path $ResolvedWorkspaceRoot "deepl_models\whisper")
}

# 清理并重建目标目录，保证每次打包从干净目录开始。
function New-CleanDirectory {
	param([string]$Path)

	if (Test-Path $Path) {
		Remove-Item -Path $Path -Recurse -Force
	}

	New-Item -ItemType Directory -Path $Path -Force | Out-Null
}

# 复制主程序、配置文件和默认输出目录到 staging。
function Copy-ApplicationPayload {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$ResolvedStageRoot,
		[string]$ResolvedConfiguration
	)

	$targetRoot = Join-Path $ResolvedStageRoot "app"
	$entryExePath = Join-Path $ResolvedWorkspaceRoot "target\$ResolvedConfiguration\entry.exe"
	$configSourcePath = Join-Path $ResolvedWorkspaceRoot "config\config_file\config.json"
	$configTargetDir = Join-Path $targetRoot "config_file"
	$outputDirs = @(
		"logs",
		"screenshots",
		"deep_learning_output",
		"report_output"
	)

	if (-not (Test-Path $entryExePath)) {
		throw "缺少可执行文件: $entryExePath"
	}

	if (-not (Test-Path $configSourcePath)) {
		throw "缺少配置文件: $configSourcePath"
	}

	Copy-Item -Path $entryExePath -Destination (Join-Path $targetRoot "entry.exe") -Force
	New-Item -ItemType Directory -Path $configTargetDir -Force | Out-Null
	Copy-Item -Path $configSourcePath -Destination (Join-Path $configTargetDir "config.json") -Force

	foreach ($outputDir in $outputDirs) {
		New-Item -ItemType Directory -Path (Join-Path $targetRoot $outputDir) -Force | Out-Null
	}
}

# 安装 WiX UI 扩展，供安装目录选择和许可页使用。
function Invoke-WixExtensionInstall {
	param(
		[string]$ExtensionId
	)

	Invoke-Step -FilePath "wix" -ArgumentList @(
		"extension",
		"add",
		"--global",
		$ExtensionId
	)
}

# 调用 WiX v5 根据 staging 目录生成 MSI。
function Invoke-WixBuild {
	param(
		[string]$ResolvedWorkspaceRoot,
		[string]$ResolvedStageRoot,
		[string]$ResolvedOutputRoot
	)

	$outputFile = Join-Path $ResolvedOutputRoot "rust_packages_survey.msi"
	$inputFile = Join-Path $ResolvedWorkspaceRoot "installer\wix\main.wxs"
	$stageAppDir = Join-Path $ResolvedStageRoot "app"

	Invoke-Step -FilePath "wix" -ArgumentList @(
		"build",
		$inputFile,
		"-arch",
		"x64",
		"-ext",
		"WixToolset.UI.wixext",
		"-bindpath",
		"StageAppDir=$stageAppDir",
		"-out",
		$outputFile
	)
}

# 解析工作区目录和输出目录配置。
$resolvedWorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
$resolvedStageRoot = Get-NormalizedPath -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedPath $StageRoot -DefaultRelativePath "installer\staging"
$resolvedOutputRoot = Get-NormalizedPath -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedPath $OutputRoot -DefaultRelativePath "installer\dist"
$installerLocalPaths = Get-InstallerLocalPaths -ResolvedWorkspaceRoot $resolvedWorkspaceRoot
$resolvedGStreamerRoot = Resolve-GStreamerRoot -RequestedGStreamerRoot $GStreamerRoot -InstallerLocalPaths $installerLocalPaths
$resolvedCudaBinRoot = Resolve-CudaBinRoot -RequestedCudaBinRoot $CudaBinRoot -InstallerLocalPaths $installerLocalPaths
$resolvedWhisperBaseModelRoot = Resolve-WhisperBaseModelRoot -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedWhisperBaseModelRoot $WhisperBaseModelRoot -InstallerLocalPaths $installerLocalPaths
$resolvedWhisperLargeV3ModelRoot = Resolve-WhisperLargeV3ModelRoot -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedWhisperLargeV3ModelRoot $WhisperLargeV3ModelRoot -InstallerLocalPaths $installerLocalPaths

# 准备干净的输出目录，并按需构建 release 可执行文件。
Assert-WixCommandAvailable
New-CleanDirectory -Path $resolvedStageRoot
New-Item -ItemType Directory -Path $resolvedOutputRoot -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $resolvedStageRoot "app") -Force | Out-Null

if (-not $SkipCargoBuild) {
	Invoke-Step -FilePath "cargo" -ArgumentList @("build", "-p", "entry", "--release")
}

# 复制程序主体，并整理 GStreamer / Whisper / CUDA 运行时资源。
Copy-ApplicationPayload -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -ResolvedStageRoot $resolvedStageRoot -ResolvedConfiguration $Configuration

Invoke-ChildScript -ScriptPath (Join-Path $PSScriptRoot "prepare_runtime.ps1") -Parameters @{
	WorkspaceRoot = $resolvedWorkspaceRoot
	GStreamerRoot = $resolvedGStreamerRoot
	StageRoot = $resolvedStageRoot
}
Invoke-ChildScript -ScriptPath (Join-Path $PSScriptRoot "prepare_models.ps1") -Parameters @{
	WorkspaceRoot = $resolvedWorkspaceRoot
	StageRoot = $resolvedStageRoot
	WhisperBaseModelRoot = $resolvedWhisperBaseModelRoot
	WhisperLargeV3ModelRoot = $resolvedWhisperLargeV3ModelRoot
	WhisperLargeV3PartsRoot = $WhisperLargeV3PartsRoot
}
Invoke-ChildScript -ScriptPath (Join-Path $PSScriptRoot "prepare_cuda_runtime.ps1") -Parameters @{
	WorkspaceRoot = $resolvedWorkspaceRoot
	CudaBinRoot = $resolvedCudaBinRoot
	StageRoot = $resolvedStageRoot
}

Invoke-WixExtensionInstall -ExtensionId "WixToolset.UI.wixext/5.0.2"

# 如果没有显式跳过，就继续生成 MSI。
if (-not $SkipMsiBuild) {
	Invoke-WixBuild -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -ResolvedStageRoot $resolvedStageRoot -ResolvedOutputRoot $resolvedOutputRoot
}

# 输出最终结果，区分“只准备 staging”和“已生成 MSI”两种模式。
Write-Host "安装包 staging 已准备完成: $resolvedStageRoot"
if ($SkipMsiBuild) {
	Write-Host "已跳过 MSI 构建，仅完成 staging 准备。"
} else {
	Write-Host "MSI 输出目录: $resolvedOutputRoot"
}
