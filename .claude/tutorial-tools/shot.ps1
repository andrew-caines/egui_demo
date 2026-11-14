param([string]$Exe, [string]$Out, [string]$Title = "Todo List", [int]$WaitSeconds = 4, [switch]$Graceful, [string]$ExeArgs = "")
# Launch the app, wait, capture its window to a PNG, report whether it is still alive, then stop it.
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
}
'@
[Win]::SetProcessDPIAware() | Out-Null
# -ExeArgs is passed to the app as its command line, e.g. -ExeArgs '"App shell"' for the concepts gallery.
if ($ExeArgs) { $proc = Start-Process -FilePath $Exe -ArgumentList $ExeArgs -PassThru }
else { $proc = Start-Process -FilePath $Exe -PassThru }
Start-Sleep -Seconds $WaitSeconds
$proc.Refresh()
if ($proc.HasExited) { Write-Output "EXITED early with code $($proc.ExitCode)"; exit 1 }
# Debug builds also own a console window, so look the GUI window up by its title.
$handle = [Win]::FindWindow([NullString]::Value, $Title)
if ($handle -eq [IntPtr]::Zero) { Write-Output "no window titled '$Title'"; Stop-Process -Id $proc.Id -Force; exit 1 }
[Win]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 700
$r = New-Object Win+RECT
[Win]::GetWindowRect($handle, [ref]$r) | Out-Null
$w = $r.R - $r.L; $h = $r.B - $r.T
$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
# Ask the window to paint itself into the bitmap. Copying the screen instead would capture
# whatever window happens to be on top, which can be private (a terminal, a browser).
$hdc = $g.GetHdc()
$ok = [Win]::PrintWindow($handle, $hdc, 2)  # PW_RENDERFULLCONTENT: needed for GPU-drawn windows
$g.ReleaseHdc($hdc)
if (-not $ok) { Write-Output "PrintWindow failed; no image written"; Stop-Process -Id $proc.Id -Force; exit 1 }
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$proc.Refresh()
Write-Output "$(Split-Path $Exe -Leaf): ALIVE=$(-not $proc.HasExited) title='$Title' size=${w}x${h}"
if ($Graceful) {
    # Ask the window to close, as a user clicking the X would, so the app can save.
    [Win]::PostMessage($handle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null  # WM_CLOSE
    if ($proc.WaitForExit(8000)) { Write-Output "closed gracefully, exit code $($proc.ExitCode)" }
    else { Write-Output "did not close in time; killing"; Stop-Process -Id $proc.Id -Force }
} else {
    Stop-Process -Id $proc.Id -Force
}
