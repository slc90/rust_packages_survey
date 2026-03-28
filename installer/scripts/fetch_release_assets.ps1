<#
.SYNOPSIS
根据 release_assets.json 下载、校验、解压并组装 GitHub Release assets。
#>
param(
	# 工作区根目录，默认自动回退到当前脚本上两级目录。
	[string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,

	# 资源清单文件路径。
	[string]$ManifestPath = "",

	# 下载输出目录，未传入时使用 installer\downloaded_assets。
	[string]$DownloadRoot = "",

	# 是否覆盖已存在的下载和解压目录。
	[switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 统一解析可选路径参数。
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

# 校验资源或分片是否填写了有效的 sha256。
function Assert-Sha256Value {
	param(
		[string]$ItemName,
		[string]$Sha256
	)

	if ([string]::IsNullOrWhiteSpace($Sha256) -or $Sha256 -eq "<sha256>") {
		throw "$ItemName 缺少有效的 sha256，请先更新 release_assets.json"
	}
}

# 按需创建目录，避免重复判断。
function Ensure-Directory {
	param([string]$Path)

	New-Item -ItemType Directory -Path $Path -Force | Out-Null
}

# 下载单个远端文件到本地缓存目录。
function Download-RemoteFile {
	param(
		[string]$ItemName,
		[string]$Url,
		[string]$OutputPath,
		[bool]$ShouldOverwrite
	)

	if ((Test-Path $OutputPath) -and -not $ShouldOverwrite) {
		Write-Host "复用已下载文件: $OutputPath"
		return
	}

	if (Test-Path $OutputPath) {
		Remove-Item -Path $OutputPath -Force
	}

	Write-Host "下载 $ItemName: $Url"
	Invoke-WebRequest -Uri $Url -OutFile $OutputPath
}

# 计算并校验本地文件的 sha256。
function Assert-FileHash {
	param(
		[string]$ItemName,
		[string]$Path,
		[string]$ExpectedSha256
	)

	Assert-Sha256Value -ItemName $ItemName -Sha256 $ExpectedSha256
	$actualSha256 = (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLowerInvariant()
	if ($actualSha256 -ne $ExpectedSha256.ToLowerInvariant()) {
		throw "$ItemName 的 sha256 校验失败。expected=$ExpectedSha256 actual=$actualSha256"
	}
}

# 解压 zip 资源包到目标目录。
function Expand-ArchiveAsset {
	param(
		[string]$ArchiveName,
		[string]$ArchivePath,
		[string]$ExtractRoot
	)

	Ensure-Directory -Path $ExtractRoot
	Write-Host "解压 $ArchiveName 到 $ExtractRoot"
	Expand-Archive -Path $ArchivePath -DestinationPath $ExtractRoot -Force
}

# 把分片按顺序拼接回原始大文件。
function Join-FileParts {
	param(
		[string]$AssetName,
		[object[]]$Parts,
		[string]$PartsRoot,
		[string]$TargetPath,
		[bool]$ShouldOverwrite
	)

	if ((Test-Path $TargetPath) -and -not $ShouldOverwrite) {
		Write-Host "复用已组装文件: $TargetPath"
		return
	}

	Ensure-Directory -Path (Split-Path -Parent $TargetPath)
	if (Test-Path $TargetPath) {
		Remove-Item -Path $TargetPath -Force
	}

	# 使用流式复制避免一次性把大文件读入内存。
	$buffer = New-Object byte[] (4MB)
	$targetStream = [System.IO.File]::Open($TargetPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
	try {
		foreach ($part in ($Parts | Sort-Object sequence)) {
			$partPath = Join-Path $PartsRoot ([string]$part.file_name)
			$partName = "$AssetName 分片 $($part.file_name)"
			Assert-FileHash -ItemName $partName -Path $partPath -ExpectedSha256 ([string]$part.sha256)

			$partStream = [System.IO.File]::OpenRead($partPath)
			try {
				while (($bytesRead = $partStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
					$targetStream.Write($buffer, 0, $bytesRead)
				}
			}
			finally {
				$partStream.Dispose()
			}
		}
	}
	finally {
		$targetStream.Dispose()
	}
}

# 校验最终解压或组装后的目录布局是否完整。
function Assert-ExtractedLayout {
	param(
		[string]$AssetName,
		[psobject]$Layout,
		[string]$ExtractRoot
	)

	foreach ($property in $Layout.PSObject.Properties) {
		$relativePath = [string]$property.Value
		$fullPath = Join-Path $ExtractRoot $relativePath
		if (-not (Test-Path $fullPath)) {
			throw "$AssetName 缺少解压布局路径: $fullPath"
		}
	}
}

# 下载并解压当前资源条目中的所有 archive。
function Restore-ArchiveList {
	param(
		[string]$AssetName,
		[object[]]$Archives,
		[string]$AssetRoot,
		[string]$ExtractRoot,
		[bool]$ShouldOverwrite
	)

	if ($null -eq $Archives -or $Archives.Count -eq 0) {
		return
	}

	$archivesRoot = Join-Path $AssetRoot "archives"
	Ensure-Directory -Path $archivesRoot

	foreach ($archive in $Archives) {
		$archiveFileName = [string]$archive.file_name
		$archivePath = Join-Path $archivesRoot $archiveFileName
		$archiveName = "$AssetName 压缩包 $archiveFileName"

		Download-RemoteFile -ItemName $archiveName -Url ([string]$archive.url) -OutputPath $archivePath -ShouldOverwrite $ShouldOverwrite
		Assert-FileHash -ItemName $archiveName -Path $archivePath -ExpectedSha256 ([string]$archive.sha256)
		Expand-ArchiveAsset -ArchiveName $archiveName -ArchivePath $archivePath -ExtractRoot $ExtractRoot
	}
}

# 下载当前资源条目中的分片，并写入本地缓存目录。
function Download-FileParts {
	param(
		[string]$AssetName,
		[object[]]$Parts,
		[string]$AssetRoot,
		[bool]$ShouldOverwrite
	)

	if ($null -eq $Parts -or $Parts.Count -eq 0) {
		return
	}

	$partsRoot = Join-Path $AssetRoot "parts"
	Ensure-Directory -Path $partsRoot

	foreach ($part in $Parts) {
		$fileName = [string]$part.file_name
		$partPath = Join-Path $partsRoot $fileName
		$partName = "$AssetName 分片 $fileName"

		Download-RemoteFile -ItemName $partName -Url ([string]$part.url) -OutputPath $partPath -ShouldOverwrite $ShouldOverwrite
		Assert-FileHash -ItemName $partName -Path $partPath -ExpectedSha256 ([string]$part.sha256)
	}
}

# 组装单个资源条目的 archive、分片和最终布局。
function Restore-Asset {
	param(
		[string]$AssetName,
		[psobject]$Asset,
		[string]$DownloadRoot,
		[bool]$ShouldOverwrite
	)

	$assetRoot = Join-Path $DownloadRoot $AssetName
	$extractRoot = Join-Path $assetRoot "extracted"

	if ((Test-Path $assetRoot) -and $ShouldOverwrite) {
		Remove-Item -Path $assetRoot -Recurse -Force
	}

	Ensure-Directory -Path $assetRoot
	Ensure-Directory -Path $extractRoot

	if ($Asset.PSObject.Properties.Name -contains "archives") {
		Restore-ArchiveList -AssetName $AssetName -Archives @($Asset.archives) -AssetRoot $assetRoot -ExtractRoot $extractRoot -ShouldOverwrite $ShouldOverwrite
	}

	if ($Asset.PSObject.Properties.Name -contains "parts") {
		Download-FileParts -AssetName $AssetName -Parts @($Asset.parts) -AssetRoot $assetRoot -ShouldOverwrite $ShouldOverwrite
		$assembledPath = Join-Path $extractRoot ([string]$Asset.assembled_file.target_path)
		Join-FileParts -AssetName $AssetName -Parts @($Asset.parts) -PartsRoot (Join-Path $assetRoot "parts") -TargetPath $assembledPath -ShouldOverwrite $ShouldOverwrite
		Assert-FileHash -ItemName "$AssetName 组装结果" -Path $assembledPath -ExpectedSha256 ([string]$Asset.assembled_file.sha256)
	}

	Assert-ExtractedLayout -AssetName $AssetName -Layout $Asset.layout -ExtractRoot $extractRoot

	$layoutMap = @{}
	foreach ($layoutProperty in $Asset.layout.PSObject.Properties) {
		$layoutMap[$layoutProperty.Name] = (Join-Path $extractRoot ([string]$layoutProperty.Value))
	}

	return @{
		version = [string]$Asset.version
		extract_root = $extractRoot
		layout = $layoutMap
	}
}

# 保存下载后生成的资源路径映射，供后续脚本直接消费。
function Save-ResolvedAssetMap {
	param(
		[string]$OutputPath,
		[hashtable]$ResolvedAssetMap
	)

	$ResolvedAssetMap |
		ConvertTo-Json -Depth 8 |
		Set-Content -Path $OutputPath -Encoding UTF8
}

# 解析工作目录、清单和下载输出目录。
$resolvedWorkspaceRoot = (Resolve-Path $WorkspaceRoot).Path
$resolvedManifestPath = if ([string]::IsNullOrWhiteSpace($ManifestPath)) {
	Join-Path $resolvedWorkspaceRoot "installer\manifests\release_assets.json"
} else {
	(Resolve-Path $ManifestPath).Path
}
$resolvedDownloadRoot = Get-NormalizedPath -ResolvedWorkspaceRoot $resolvedWorkspaceRoot -RequestedPath $DownloadRoot -DefaultRelativePath "installer\downloaded_assets"

if ((Test-Path $resolvedDownloadRoot) -and $Force) {
	Remove-Item -Path $resolvedDownloadRoot -Recurse -Force
}
Ensure-Directory -Path $resolvedDownloadRoot

$manifest = Get-Content $resolvedManifestPath -Raw | ConvertFrom-Json
$resolvedAssetMap = @{}

foreach ($property in $manifest.PSObject.Properties) {
	$assetName = $property.Name
	$asset = $property.Value
	$resolvedAssetMap[$assetName] = Restore-Asset -AssetName $assetName -Asset $asset -DownloadRoot $resolvedDownloadRoot -ShouldOverwrite $Force.IsPresent
}

$resolvedAssetMapPath = Join-Path $resolvedDownloadRoot "resolved_assets.json"
Save-ResolvedAssetMap -OutputPath $resolvedAssetMapPath -ResolvedAssetMap $resolvedAssetMap

Write-Host "已完成预制安装资源包下载与解压: $resolvedDownloadRoot"
Write-Host "资源路径清单: $resolvedAssetMapPath"
