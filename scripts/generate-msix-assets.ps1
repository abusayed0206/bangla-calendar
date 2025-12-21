# Generate MSIX Assets Script
# This script creates required PNG assets for Windows Store MSIX package

Add-Type -AssemblyName System.Drawing

$sourceIco = "c:\Users\Sayed\Desktop\bongo\assets\Flag_of_Bangladesh.ico"
$assetsDir = "c:\Users\Sayed\Desktop\bongo\msix-package\Assets"

# Required image sizes for Windows Store
$sizes = @{
    "StoreLogo.png" = 50
    "Square44x44Logo.png" = 44
    "Square44x44Logo.targetsize-44.png" = 44
    "Square44x44Logo.targetsize-44_altform-unplated.png" = 44
    "Square71x71Logo.png" = 71
    "SmallTile.png" = 71
    "Square150x150Logo.png" = 150
    "Wide310x150Logo.png" = @(310, 150)
    "Square310x310Logo.png" = 310
    "SplashScreen.png" = @(620, 300)
    "LargeTile.png" = 310
}

# Scale factors for additional sizes
$scales = @(100, 125, 150, 200, 400)

# Load the icon
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon($sourceIco)

function Resize-Image {
    param(
        [System.Drawing.Icon]$sourceIcon,
        [int]$width,
        [int]$height,
        [string]$outputPath
    )
    
    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    
    # Fill with Bangladesh flag green background
    $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0, 106, 78))
    $graphics.FillRectangle($brush, 0, 0, $width, $height)
    
    # Calculate centered position for the icon
    $iconSize = [Math]::Min($width, $height) * 0.7
    $x = ($width - $iconSize) / 2
    $y = ($height - $iconSize) / 2
    
    # Draw the icon centered
    $graphics.DrawIcon($sourceIcon, [System.Drawing.Rectangle]::FromLTRB($x, $y, $x + $iconSize, $y + $iconSize))
    
    $bitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    
    $graphics.Dispose()
    $bitmap.Dispose()
    $brush.Dispose()
}

Write-Host "Generating MSIX assets..." -ForegroundColor Cyan

foreach ($name in $sizes.Keys) {
    $size = $sizes[$name]
    $outputPath = Join-Path $assetsDir $name
    
    if ($size -is [array]) {
        $width = $size[0]
        $height = $size[1]
    } else {
        $width = $size
        $height = $size
    }
    
    Resize-Image -sourceIcon $icon -width $width -height $height -outputPath $outputPath
    Write-Host "  Created: $name ($width x $height)" -ForegroundColor Green
}

# Generate scaled versions for key assets
$scaledAssets = @(
    @{ Name = "Square44x44Logo"; BaseSize = 44 },
    @{ Name = "Square150x150Logo"; BaseSize = 150 },
    @{ Name = "SmallTile"; BaseSize = 71 },
    @{ Name = "StoreLogo"; BaseSize = 50 }
)

foreach ($asset in $scaledAssets) {
    foreach ($scale in $scales) {
        $scaledSize = [int]($asset.BaseSize * $scale / 100)
        $scaledName = "$($asset.Name).scale-$scale.png"
        $outputPath = Join-Path $assetsDir $scaledName
        
        Resize-Image -sourceIcon $icon -width $scaledSize -height $scaledSize -outputPath $outputPath
        Write-Host "  Created: $scaledName ($scaledSize x $scaledSize)" -ForegroundColor Green
    }
}

# Generate wide and large tile scaled versions
$wideSizes = @(100, 125, 150, 200, 400)
foreach ($scale in $wideSizes) {
    $width = [int](310 * $scale / 100)
    $height = [int](150 * $scale / 100)
    $name = "Wide310x150Logo.scale-$scale.png"
    $outputPath = Join-Path $assetsDir $name
    
    Resize-Image -sourceIcon $icon -width $width -height $height -outputPath $outputPath
    Write-Host "  Created: $name ($width x $height)" -ForegroundColor Green
}

# SplashScreen scaled versions
foreach ($scale in $wideSizes) {
    $width = [int](620 * $scale / 100)
    $height = [int](300 * $scale / 100)
    $name = "SplashScreen.scale-$scale.png"
    $outputPath = Join-Path $assetsDir $name
    
    Resize-Image -sourceIcon $icon -width $width -height $height -outputPath $outputPath
    Write-Host "  Created: $name ($width x $height)" -ForegroundColor Green
}

# Square310x310Logo scaled versions
foreach ($scale in $wideSizes) {
    $size = [int](310 * $scale / 100)
    $name = "Square310x310Logo.scale-$scale.png"
    $outputPath = Join-Path $assetsDir $name
    
    Resize-Image -sourceIcon $icon -width $size -height $size -outputPath $outputPath
    Write-Host "  Created: $name ($size x $size)" -ForegroundColor Green
}

$icon.Dispose()

Write-Host ""
Write-Host "All assets generated successfully!" -ForegroundColor Cyan
Write-Host "Assets directory: $assetsDir" -ForegroundColor Yellow
