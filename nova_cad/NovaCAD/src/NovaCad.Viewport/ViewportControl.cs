using System;
using System.Numerics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.OpenGL;
using Avalonia.OpenGL.Controls;
using Avalonia.Rendering;
using Avalonia.Threading;
using NovaCad.Core.Models;
using static Avalonia.OpenGL.GlConsts;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Avalonia OpenGL control for 3D viewport rendering
    /// </summary>
    public class ViewportControl : OpenGlControlBase, IDisposable
    {
        private Viewport3D? _viewport;
        private bool _isInitialized;
        private bool _disposed;

        // ViewModel
        private IViewportViewModel? _viewModel;
        public IViewportViewModel? ViewModel
        {
            get => _viewModel;
            set
            {
                ViewportDiagnostics.Log($"ViewModel setter called: {(value != null ? "not null" : "NULL")}", LogLevel.Debug);
                
                if (_viewModel != null)
                {
                    _viewModel.VisualObjectCreated -= OnVisualObjectCreated;
                    _viewModel.VisualObjectsCleared -= OnVisualObjectsCleared;
                    _viewModel.RenderModeChanged -= OnRenderModeChanged;
                    _viewModel.ViewPresetChanged -= OnViewPresetChanged;
                    _viewModel.FitAllRequested -= OnFitAllRequested;
                }

                _viewModel = value;

                if (_viewModel != null)
                {
                    _viewModel.VisualObjectCreated += OnVisualObjectCreated;
                    _viewModel.VisualObjectsCleared += OnVisualObjectsCleared;
                    _viewModel.RenderModeChanged += OnRenderModeChanged;
                    _viewModel.ViewPresetChanged += OnViewPresetChanged;
                    _viewModel.FitAllRequested += OnFitAllRequested;
                    
                    ViewportDiagnostics.LogViewModelState(_viewModel);
                }
            }
        }

        public ViewportControl()
        {
            ViewportDiagnostics.Log("ViewportControl constructor called", LogLevel.Info);
            Focusable = true;
        }

        protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
        {
            base.OnAttachedToVisualTree(e);
            ViewportDiagnostics.Log("ViewportControl attached to visual tree", LogLevel.Info);
        }

        protected override void OnOpenGlInit(GlInterface gl)
        {
            base.OnOpenGlInit(gl);
            ViewportDiagnostics.Log("OnOpenGlInit called", LogLevel.Info);

            if (_isInitialized)
            {
                ViewportDiagnostics.Log("Already initialized, skipping", LogLevel.Warning);
                return;
            }

            try
            {
                // Initialize extension methods
                GlExtensions.Initialize(gl);
                ViewportDiagnostics.Log("GlExtensions initialized", LogLevel.Debug);

                // Log OpenGL info
                ViewportDiagnostics.LogOpenGLInfo(gl);

                // Create viewport
                var width = (int)Bounds.Width;
                var height = (int)Bounds.Height;
                ViewportDiagnostics.Log($"Creating Viewport3D with size {width}x{height}", LogLevel.Debug);
                
                _viewport = new Viewport3D(gl, width, height);
                _isInitialized = true;
                ViewportDiagnostics.Log("Viewport3D created successfully", LogLevel.Info);

                // Sync with ViewModel if available
                if (ViewModel != null)
                {
                    ViewportDiagnostics.Log("Syncing with existing ViewModel", LogLevel.Debug);
                    _viewport.ShowGrid = ViewModel.ShowGrid;
                    _viewport.ShowAxes = ViewModel.ShowAxes;
                    
                    // Add existing visual objects
                    if (ViewModel.VisualObjects != null)
                    {
                        ViewportDiagnostics.Log($"Adding {ViewModel.VisualObjects.Count} existing visual objects", LogLevel.Debug);
                        foreach (var visualObject in ViewModel.VisualObjects)
                        {
                            if (visualObject?.Mesh != null)
                            {
                                if (!visualObject.Mesh.IsInitialized)
                                {
                                    ViewportDiagnostics.Log($"Initializing mesh for {visualObject.Name}", LogLevel.Debug);
                                    visualObject.Mesh.Initialize(gl);
                                }
                                _viewport.AddMesh(visualObject.Mesh);
                                ViewportDiagnostics.LogMeshInfo(visualObject.Mesh, visualObject.Name);
                            }
                        }
                    }
                }
                else
                {
                    ViewportDiagnostics.Log("No ViewModel attached yet", LogLevel.Warning);
                }
                
                ViewportDiagnostics.LogViewportState(_viewport);
            }
            catch (Exception ex)
            {
                ViewportDiagnostics.LogException(ex, "OnOpenGlInit");
            }
        }

        protected override void OnOpenGlRender(GlInterface gl, int fb)
        {
            if (_viewport == null || _disposed)
            {
                // Log only occasionally to avoid spam
                if (_viewport == null && DateTime.Now.Second % 5 == 0)
                {
                    ViewportDiagnostics.Log("Render called but viewport is null", LogLevel.Warning);
                }
                return;
            }

            try
            {
                // Set viewport size
                gl.Viewport(0, 0, (int)Bounds.Width, (int)Bounds.Height);

                // Render
                _viewport.Render(gl);
            }
            catch (Exception ex)
            {
                ViewportDiagnostics.LogException(ex, "OnOpenGlRender");
            }
        }

        protected override void OnOpenGlDeinit(GlInterface gl)
        {
            base.OnOpenGlDeinit(gl);
            ViewportDiagnostics.Log("OnOpenGlDeinit called", LogLevel.Info);
            _viewport?.Dispose();
            _isInitialized = false;
        }

        private void OnVisualObjectCreated(object? sender, VisualObjectCreatedEventArgs e)
        {
            ViewportDiagnostics.Log($"VisualObjectCreated: {e.Name}", LogLevel.Info);
            
            if (_viewport == null)
            {
                ViewportDiagnostics.Log("Viewport is null, cannot add mesh yet", LogLevel.Warning);
                return;
            }

            Dispatcher.UIThread.Post(() =>
            {
                try
                {
                    if (e.Mesh == null)
                    {
                        ViewportDiagnostics.Log($"Mesh for {e.Name} is null!", LogLevel.Error);
                        return;
                    }

                    // Get GL interface from current context if needed
                    if (!e.Mesh.IsInitialized)
                    {
                        ViewportDiagnostics.Log($"Initializing mesh {e.Name}", LogLevel.Debug);
                        // Mesh will be initialized on next render
                    }

                    _viewport.AddMesh(e.Mesh);
                    ViewportDiagnostics.LogMeshInfo(e.Mesh, e.Name);
                    ViewportDiagnostics.Log($"Mesh {e.Name} added to viewport", LogLevel.Info);
                    
                    RequestNextFrameRendering();
                }
                catch (Exception ex)
                {
                    ViewportDiagnostics.LogException(ex, "OnVisualObjectCreated");
                }
            });
        }

        private void OnVisualObjectsCleared(object? sender, EventArgs e)
        {
            ViewportDiagnostics.Log("VisualObjectsCleared", LogLevel.Info);
            _viewport?.ClearMeshes();
            RequestNextFrameRendering();
        }

        private void OnRenderModeChanged(object? sender, RenderMode renderMode)
        {
            ViewportDiagnostics.Log($"RenderModeChanged to {renderMode}", LogLevel.Debug);
            if (_viewport == null) return;
            _viewport.Wireframe = (renderMode == RenderMode.Wireframe);
            RequestNextFrameRendering();
        }

        private void OnViewPresetChanged(object? sender, string viewName)
        {
            ViewportDiagnostics.Log($"ViewPresetChanged to {viewName}", LogLevel.Debug);
            
            StandardView? view = viewName switch
            {
                "Front" => StandardView.Front,
                "Top" => StandardView.Top,
                "Right" => StandardView.Right,
                "Iso" => StandardView.Isometric,
                _ => null
            };

            if (view.HasValue && _viewport != null)
            {
                _viewport.SetStandardView(view.Value);
                RequestNextFrameRendering();
            }
        }

        private void OnFitAllRequested(object? sender, EventArgs e)
        {
            ViewportDiagnostics.Log("FitAllRequested", LogLevel.Debug);
            _viewport?.FitView();
            RequestNextFrameRendering();
        }

        protected override void OnPointerPressed(PointerPressedEventArgs e)
        {
            base.OnPointerPressed(e);
            
            if (_viewport == null) return;

            var point = e.GetPosition(this);
            var button = e.GetCurrentPoint(this).Properties.PointerUpdateKind switch
            {
                PointerUpdateKind.LeftButtonPressed => ViewportMouseButton.Left,
                PointerUpdateKind.MiddleButtonPressed => ViewportMouseButton.Middle,
                PointerUpdateKind.RightButtonPressed => ViewportMouseButton.Right,
                _ => ViewportMouseButton.Left
            };

            ViewportDiagnostics.Log($"MouseDown: {button} at ({point.X:F0},{point.Y:F0})", LogLevel.Debug);
            _viewport.OnMouseDown(button, (int)point.X, (int)point.Y);
            Focus();
        }

        protected override void OnPointerMoved(PointerEventArgs e)
        {
            base.OnPointerMoved(e);
            
            if (_viewport == null) return;

            var point = e.GetPosition(this);
            var props = e.GetCurrentPoint(this).Properties;
            
            _viewport.OnMouseMove(
                (int)point.X, 
                (int)point.Y,
                props.IsLeftButtonPressed,
                props.IsMiddleButtonPressed,
                props.IsRightButtonPressed);

            if (props.IsLeftButtonPressed || props.IsMiddleButtonPressed || props.IsRightButtonPressed)
            {
                RequestNextFrameRendering();
            }
        }

        protected override void OnPointerReleased(PointerReleasedEventArgs e)
        {
            base.OnPointerReleased(e);
            
            if (_viewport == null) return;

            var button = e.InitialPressMouseButton switch
            {
                Avalonia.Input.MouseButton.Left => ViewportMouseButton.Left,
                Avalonia.Input.MouseButton.Middle => ViewportMouseButton.Middle,
                Avalonia.Input.MouseButton.Right => ViewportMouseButton.Right,
                _ => ViewportMouseButton.Left
            };

            ViewportDiagnostics.Log($"MouseUp: {button}", LogLevel.Debug);
            _viewport.OnMouseUp(button);
        }

        protected override void OnPointerWheelChanged(PointerWheelEventArgs e)
        {
            base.OnPointerWheelChanged(e);
            
            if (_viewport == null) return;

            float delta = (float)e.Delta.Y;
            ViewportDiagnostics.Log($"MouseWheel: {delta:F1}", LogLevel.Debug);
            _viewport.OnMouseWheel(delta);
            RequestNextFrameRendering();
        }

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);

            if (change.Property == BoundsProperty)
            {
                var newBounds = (Rect)change.NewValue!;
                ViewportDiagnostics.Log($"Bounds changed to {newBounds.Width:F0}x{newBounds.Height:F0}", LogLevel.Debug);
                _viewport?.Resize((int)newBounds.Width, (int)newBounds.Height);
                RequestNextFrameRendering();
            }
        }

        public void Dispose()
        {
            if (_disposed) return;

            ViewportDiagnostics.Log("ViewportControl disposing", LogLevel.Info);

            if (_viewModel != null)
            {
                _viewModel.VisualObjectCreated -= OnVisualObjectCreated;
                _viewModel.VisualObjectsCleared -= OnVisualObjectsCleared;
                _viewModel.RenderModeChanged -= OnRenderModeChanged;
                _viewModel.ViewPresetChanged -= OnViewPresetChanged;
                _viewModel.FitAllRequested -= OnFitAllRequested;
            }

            _viewport?.Dispose();
            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }
}
