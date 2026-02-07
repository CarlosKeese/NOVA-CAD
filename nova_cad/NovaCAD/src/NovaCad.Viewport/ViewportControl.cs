using System;
using System.Numerics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Platform;
using Avalonia.Rendering;
using Avalonia.Interactivity;
// Note: Avalonia.Silk integration requires Avalonia.Silk package or custom implementation
using Silk.NET.OpenGL;
using NovaCad.Core.Models;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Avalonia control for 3D viewport rendering
    /// </summary>
    public class ViewportControl : NativeControlHost, IDisposable
    {
        private Viewport3D? _viewport;
        private GL? _gl;
        private bool _isInitialized;
        private bool _disposed;

        // Document reference
        public NovaDocument? Document { get; set; }

        // Events
        public event EventHandler<ViewportClickEventArgs>? EntityPicked
        {
            add => AddHandler(EntityPickedEvent, value);
            remove => RemoveHandler(EntityPickedEvent, value);
        }

        public static readonly RoutedEvent EntityPickedEvent =
            RoutedEvent.Register<ViewportControl, RoutedEventArgs>(nameof(EntityPicked), RoutingStrategies.Bubble);

        public ViewportControl()
        {
            Focusable = true;
            Background = new SolidColorBrush(Colors.Transparent);
        }

        protected override IPlatformHandle CreateNativeControlCore(IPlatformHandle parent)
        {
            var handle = base.CreateNativeControlCore(parent);
            InitializeOpenGL();
            return handle;
        }

        private void InitializeOpenGL()
        {
            if (_isInitialized) return;

            try
            {
                // Create OpenGL context using Silk.NET
                var window = Silk.NET.Windowing.Window.Create(new WindowOptions
                {
                    Size = new Silk.NET.Maths.Vector2D<int>((int)Bounds.Width, (int)Bounds.Height),
                    Title = "NovaCAD Viewport",
                    API = GraphicsAPI.Default,
                    VSync = true
                });

                _gl = GL.GetApi(window);
                
                // Create viewport
                _viewport = new Viewport3D(_gl, (int)Bounds.Width, (int)Bounds.Height);
                _viewport.EntityPicked += OnViewportEntityPicked;

                _isInitialized = true;
                
                // Load document if available
                if (Document != null)
                {
                    LoadDocument(Document);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to initialize OpenGL: {ex.Message}");
            }
        }

        /// <summary>
        /// Load a document into the viewport
        /// </summary>
        public void LoadDocument(NovaDocument document)
        {
            Document = document;
            
            if (!_isInitialized || _viewport == null) return;

            _viewport.ClearMeshes();

            // Create meshes from document bodies
            foreach (var bodyRef in document.Bodies)
            {
                var mesh = Mesh.FromBody(bodyRef, _gl!);
                _viewport.AddMesh(mesh);
            }

            // Fit view to content
            _viewport.FitView();
        }

        /// <summary>
        /// Refresh the viewport
        /// </summary>
        public void Refresh()
        {
            if (Document != null)
            {
                LoadDocument(Document);
            }
        }

        /// <summary>
        /// Set standard view
        /// </summary>
        public void SetView(StandardView view)
        {
            _viewport?.SetStandardView(view);
            InvalidateVisual();
        }

        /// <summary>
        /// Fit view to content
        /// </summary>
        public void FitView()
        {
            _viewport?.FitView();
            InvalidateVisual();
        }

        protected override void OnPointerPressed(PointerPressedEventArgs e)
        {
            base.OnPointerPressed(e);
            
            if (_viewport == null) return;

            var point = e.GetPosition(this);
            var button = e.GetCurrentPoint(this).Properties.PointerUpdateKind switch
            {
                PointerUpdateKind.LeftButtonPressed => MouseButton.Left,
                PointerUpdateKind.MiddleButtonPressed => MouseButton.Middle,
                PointerUpdateKind.RightButtonPressed => MouseButton.Right,
                _ => MouseButton.Left
            };

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
        }

        protected override void OnPointerReleased(PointerReleasedEventArgs e)
        {
            base.OnPointerReleased(e);
            
            if (_viewport == null) return;

            var button = e.InitialPressMouseButton switch
            {
                MouseButton.Left => Viewport.MouseButton.Left,
                MouseButton.Middle => Viewport.MouseButton.Middle,
                MouseButton.Right => Viewport.MouseButton.Right,
                _ => Viewport.MouseButton.Left
            };

            _viewport.OnMouseUp(button);
        }

        protected override void OnPointerWheelChanged(PointerWheelEventArgs e)
        {
            base.OnPointerWheelChanged(e);
            
            if (_viewport == null) return;

            float delta = (float)e.Delta.Y;
            _viewport.OnMouseWheel(delta);
        }

        public override void Render(DrawingContext context)
        {
            base.Render(context);

            if (_viewport != null && _isInitialized)
            {
                _viewport.Render();
            }
        }

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);

            if (change.Property == BoundsProperty)
            {
                var newBounds = (Rect)change.NewValue!;
                _viewport?.Resize((int)newBounds.Width, (int)newBounds.Height);
            }
        }

        private void OnViewportEntityPicked(object? sender, ViewportClickEventArgs e)
        {
            RaiseEvent(new ViewportClickEventArgs(e.EntityId, e.X, e.Y)
            {
                RoutedEvent = EntityPickedEvent,
                Source = this
            });
        }

        public void Dispose()
        {
            if (_disposed) return;

            _viewport?.Dispose();
            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }
}
