# Windows Store Submission Guide

## Package Identity (From Partner Center)

These values are configured in `AppxManifest.xml`:

| Field | Value |
|-------|-------|
| Package/Identity/Name | `abusayed.dev.BanglaCalenderforWindows` |
| Package/Identity/Publisher | `CN=8EA73F0A-0DC1-449F-A896-493A34649C49` |
| Package/Properties/PublisherDisplayName | `abusayed.dev` |
| Package Family Name (PFN) | `abusayed.dev.BanglaCalenderforWindows_a7ccwk4vg4vf2` |
| Store ID | `9NFWS2MHSH0T` |

## Generated Files

After running the build script, you'll have:

| File | Purpose |
|------|---------|
| `BanglaCalendar_0.0.2.0_x64.msix` | Main package (can be installed directly) |
| `BanglaCalendar_0.0.2.0_x64.msixupload` | Upload file for Partner Center |

## How to Submit to Windows Store

### 1. Go to Partner Center
- Visit: https://partner.microsoft.com/dashboard
- Sign in with your Microsoft Developer account

### 2. Navigate to Your App
- Go to **Apps and games** → **Bangla Calendar for Windows**
- Or search for Store ID: `9NFWS2MHSH0T`

### 3. Create New Submission
1. Click **Start update** or **Create new submission**
2. Go to **Packages** section
3. Upload `BanglaCalendar_0.0.2.0_x64.msixupload` or `.msix` file
4. Fill in the Store listing details (if needed)

### 4. Required Store Listing Assets
The Store may require additional assets not in the MSIX:
- **1400×2000** promotional image
- **2400×1200** hero image
- Screenshots (various sizes)
- Store description and keywords

### 5. Certification
- Submit for certification
- Wait 1-3 business days for review
- Address any certification issues if flagged

## Building New Versions

1. Update version in `Cargo.toml`
2. Build the release: `cargo build --release`
3. Run: `.\scripts\build-msix.ps1 -Version "0.0.3.0"`
4. Upload the new `.msixupload` file to Partner Center

## Package Structure

```
msix-package/
├── AppxManifest.xml       # Package manifest with identity
├── bangla-calendar.exe    # Your compiled binary
├── resources.pri          # Resource index
└── Assets/
    ├── StoreLogo.png      # 50x50 (with scales)
    ├── Square44x44Logo.png
    ├── Square150x150Logo.png
    ├── Square310x310Logo.png
    ├── Wide310x150Logo.png
    ├── SmallTile.png
    └── SplashScreen.png
```

## Local Testing

To test the MSIX package locally before submission:

```powershell
# Install for testing (requires developer mode or signing)
Add-AppxPackage -Path "releases\BanglaCalendar_0.0.2.0_x64.msix"

# To uninstall
Get-AppxPackage *BanglaCalendar* | Remove-AppxPackage
```

**Note:** For local installation without Store signing, you may need to:
1. Enable Developer Mode in Windows Settings
2. Or sign the package with a test certificate

## Minimum Windows Version

- **Minimum:** Windows 10 version 1809 (Build 17763)
- **Tested up to:** Windows 11 (Build 22631)

## Notes

- The package uses `runFullTrust` capability for Win32 API access
- Background color (#006A4E) matches Bangladesh flag green
- Supports both English (en-us) and Bangla (bn-BD) resources
