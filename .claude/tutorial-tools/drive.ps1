param([string]$Exe, [string]$Title = "Todo List", [string]$Steps, [int]$StartupSeconds = 4)
# Launch the app and run a list of steps against its window, then close it as a user would.
# Steps:  click:X,Y   left-click at window-relative pixels (measure them on a screenshot)
#         wait:MS     pause
#         shot:PATH   save a PNG of the window
# Steps are one string separated by semicolons, which survives `pwsh -File` argument passing.
# Example: drive.ps1 -Exe app.exe -Steps "click:27,43;wait:500;shot:out.png"
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinDrive {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
}
'@
[WinDrive]::SetProcessDPIAware() | Out-Null
# -NoNewWindow: a debug build would otherwise open its own console, which can cover the app.
$proc = Start-Process -FilePath $Exe -PassThru -NoNewWindow
Start-Sleep -Seconds $StartupSeconds
$proc.Refresh()
if ($proc.HasExited) { Write-Output "EXITED early with code $($proc.ExitCode)"; exit 1 }
$handle = [WinDrive]::FindWindow([NullString]::Value, $Title)
if ($handle -eq [IntPtr]::Zero) { Write-Output "no window titled '$Title'"; Stop-Process -Id $proc.Id -Force; exit 1 }
[WinDrive]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 700
$r = New-Object WinDrive+RECT
[WinDrive]::GetWindowRect($handle, [ref]$r) | Out-Null

foreach ($step in ($Steps -split ';')) {
    $kind, $value = $step -split ':', 2
    switch ($kind) {
        'click' {
            $x, $y = $value -split ','
            [WinDrive]::SetForegroundWindow($handle) | Out-Null
            [WinDrive]::SetCursorPos($r.L + [int]$x, $r.T + [int]$y) | Out-Null
            Start-Sleep -Milliseconds 150
            [WinDrive]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)  # left button down
            Start-Sleep -Milliseconds 60
            [WinDrive]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)  # left button up
            Start-Sleep -Milliseconds 300
        }
        'wait' { Start-Sleep -Milliseconds ([int]$value) }
        'shot' {
            $w = $r.R - $r.L; $h = $r.B - $r.T
            $bmp = New-Object System.Drawing.Bitmap $w, $h
            $g = [System.Drawing.Graphics]::FromImage($bmp)
            $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
            $bmp.Save($value, [System.Drawing.Imaging.ImageFormat]::Png)
            Write-Output "saved $value (${w}x${h})"
        }
    }
}

$proc.Refresh()
Write-Output "$(Split-Path $Exe -Leaf): ALIVE=$(-not $proc.HasExited)"
[WinDrive]::PostMessage($handle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null  # WM_CLOSE
if ($proc.WaitForExit(8000)) { Write-Output "closed gracefully, exit code $($proc.ExitCode)" }
else { Write-Output "did not close in time; killing"; Stop-Process -Id $proc.Id -Force }
