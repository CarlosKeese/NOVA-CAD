using System;
using System.Collections.Generic;
using System.IO;
using Avalonia.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Diagnostic logging system for viewport troubleshooting
    /// </summary>
    public static class ViewportDiagnostics
    {
        private static readonly List<LogEntry> _logs = new();
        private static readonly string _logPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "NovaCAD",
            "viewport_logs.txt"
        );
        
        public static bool IsEnabled { get; set; } = true;
        public static LogLevel MinimumLevel { get; set; } = LogLevel.Debug;
        
        static ViewportDiagnostics()
        {
            // Ensure directory exists
            var dir = Path.GetDirectoryName(_logPath);
            if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }
            
            Log("=== Nova CAD Viewport Diagnostics Started ===", LogLevel.Info);
            Log($"Log file: {_logPath}", LogLevel.Info);
            Log($"OpenGL Version: {GetOpenGLVersion()}", LogLevel.Info);
        }
        
        public static void Log(string message, LogLevel level = LogLevel.Debug)
        {
            if (!IsEnabled || level < MinimumLevel) return;
            
            var entry = new LogEntry
            {
                Timestamp = DateTime.Now,
                Level = level,
                Message = message
            };
            
            _logs.Add(entry);
            
            // Also write to console for immediate feedback
            var color = level switch
            {
                LogLevel.Error => ConsoleColor.Red,
                LogLevel.Warning => ConsoleColor.Yellow,
                LogLevel.Info => ConsoleColor.White,
                _ => ConsoleColor.Gray
            };
            
            var previousColor = Console.ForegroundColor;
            Console.ForegroundColor = color;
            Console.WriteLine($"[VIEWPORT] [{level}] {message}");
            Console.ForegroundColor = previousColor;
            
            // Append to file
            try
            {
                File.AppendAllText(_logPath, $"{entry.Timestamp:yyyy-MM-dd HH:mm:ss.fff} [{level}] {message}{Environment.NewLine}");
            }
            catch { /* Ignore file write errors */ }
        }
        
        public static void LogOpenGLInfo(GlInterface gl)
        {
            if (gl == null)
            {
                Log("GlInterface is NULL!", LogLevel.Error);
                return;
            }
            
            Log("=== OpenGL Information ===", LogLevel.Info);
            try
            {
                var version = gl.GetString(GL_VERSION);
                Log($"OpenGL Version: {version}", LogLevel.Info);
            }
            catch (Exception ex)
            {
                Log($"Failed to get OpenGL version: {ex.Message}", LogLevel.Error);
            }
            
            try
            {
                var vendor = gl.GetString(GL_VENDOR);
                Log($"OpenGL Vendor: {vendor}", LogLevel.Info);
            }
            catch (Exception ex)
            {
                Log($"Failed to get OpenGL vendor: {ex.Message}", LogLevel.Error);
            }
            
            try
            {
                var renderer = gl.GetString(GL_RENDERER);
                Log($"OpenGL Renderer: {renderer}", LogLevel.Info);
            }
            catch (Exception ex)
            {
                Log($"Failed to get OpenGL renderer: {ex.Message}", LogLevel.Error);
            }
        }
        
        public static void LogViewportState(Viewport3D viewport)
        {
            if (viewport == null)
            {
                Log("Viewport3D is NULL!", LogLevel.Error);
                return;
            }
            
            Log("=== Viewport State ===", LogLevel.Debug);
            Log($"Size: {viewport.Width}x{viewport.Height}", LogLevel.Debug);
            Log($"ShowGrid: {viewport.ShowGrid}", LogLevel.Debug);
            Log($"ShowAxes: {viewport.ShowAxes}", LogLevel.Debug);
            Log($"Wireframe: {viewport.Wireframe}", LogLevel.Debug);
            Log($"Background: {viewport.BackgroundColor.R},{viewport.BackgroundColor.G},{viewport.BackgroundColor.B}", LogLevel.Debug);
        }
        
        public static void LogMeshInfo(Mesh mesh, string name)
        {
            if (mesh == null)
            {
                Log($"Mesh '{name}' is NULL!", LogLevel.Error);
                return;
            }
            
            Log($"=== Mesh: {name} ===", LogLevel.Debug);
            Log($"  Vertices: {mesh.Vertices?.Count ?? 0}", LogLevel.Debug);
            Log($"  Indices: {mesh.Indices?.Count ?? 0}", LogLevel.Debug);
            Log($"  IsInitialized: {mesh.IsInitialized}", LogLevel.Debug);
            Log($"  IsVisible: {mesh.IsVisible}", LogLevel.Debug);
            Log($"  EntityId: {mesh.EntityId}", LogLevel.Debug);
            
            if (mesh.Vertices?.Count > 0)
            {
                var bbox = mesh.GetBoundingBox();
                Log($"  BoundingBox: Min({bbox.Min.X:F2},{bbox.Min.Y:F2},{bbox.Min.Z:F2}) Max({bbox.Max.X:F2},{bbox.Max.Y:F2},{bbox.Max.Z:F2})", LogLevel.Debug);
            }
        }
        
        public static void LogException(Exception ex, string context)
        {
            Log($"EXCEPTION in {context}: {ex.GetType().Name}", LogLevel.Error);
            Log($"  Message: {ex.Message}", LogLevel.Error);
            Log($"  StackTrace: {ex.StackTrace}", LogLevel.Error);
            
            if (ex.InnerException != null)
            {
                Log($"  Inner Exception: {ex.InnerException.Message}", LogLevel.Error);
            }
        }
        
        public static void LogViewModelState(IViewportViewModel vm)
        {
            if (vm == null)
            {
                Log("ViewModel is NULL!", LogLevel.Error);
                return;
            }
            
            Log("=== ViewModel State ===", LogLevel.Debug);
            Log($"RenderMode: {vm.RenderMode}", LogLevel.Debug);
            Log($"ShowGrid: {vm.ShowGrid}", LogLevel.Debug);
            Log($"ShowAxes: {vm.ShowAxes}", LogLevel.Debug);
            Log($"VisualObjects count: {vm.VisualObjects?.Count ?? 0}", LogLevel.Debug);
            
            if (vm.VisualObjects != null)
            {
                int idx = 0;
                foreach (var obj in vm.VisualObjects)
                {
                    Log($"  [{idx}] {obj.Name} - Visible:{obj.IsVisible} Selected:{obj.IsSelected}", LogLevel.Debug);
                    idx++;
                }
            }
        }
        
        public static string GetLogPath() => _logPath;
        
        public static List<LogEntry> GetLogs() => new(_logs);
        
        public static void ClearLogs()
        {
            _logs.Clear();
            try
            {
                if (File.Exists(_logPath))
                {
                    File.Delete(_logPath);
                }
            }
            catch { }
        }
        
        private static string GetOpenGLVersion()
        {
            try
            {
                // This will be populated when GL context is available
                return "Not yet initialized";
            }
            catch
            {
                return "Unknown";
            }
        }
        
        // OpenGL constants for GetString
        private const int GL_VERSION = 0x1F02;
        private const int GL_VENDOR = 0x1F00;
        private const int GL_RENDERER = 0x1F01;
    }
    
    public enum LogLevel
    {
        Debug = 0,
        Info = 1,
        Warning = 2,
        Error = 3
    }
    
    public struct LogEntry
    {
        public DateTime Timestamp;
        public LogLevel Level;
        public string Message;
    }
}
