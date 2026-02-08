using System;
using System.Collections.Generic;
using System.Numerics;
using Silk.NET.OpenGL;
using Silk.NET.Maths;
using NovaCad.Core.Models;
using NovaCad.Kernel;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D Viewport for rendering CAD geometry using OpenGL
    /// </summary>
    public unsafe class Viewport3D : IDisposable
    {
        private GL _gl;
        private Camera3D _camera;
        private Renderer _renderer;
        
        private List<Mesh> _meshes;
        private bool _disposed;
        
        // Grid rendering
        private uint _gridVao;
        private uint _gridVbo;
        private int _gridVertexCount;
        private Shader _gridShader;
        
        // Axes rendering
        private uint _axesVao;
        private uint _axesVbo;
        private Shader _axesShader;
        
        // Viewport state
        public int Width { get; private set; }
        public int Height { get; private set; }
        public Color BackgroundColor { get; set; } = new Color(0.15f, 0.15f, 0.15f, 1.0f);
        
        // Selection
        public uint? SelectedEntityId { get; set; }
        public List<uint> HighlightedEntities { get; } = new();
        
        // Rendering options
        public bool ShowGrid { get; set; } = true;
        public bool ShowAxes { get; set; } = true;
        public bool Wireframe { get; set; } = false;
        
        // Events
        public event EventHandler<ViewportClickEventArgs>? EntityPicked;
        public event EventHandler<ViewportHoverEventArgs>? EntityHovered;

        public Viewport3D(GL gl, int width, int height)
        {
            _gl = gl ?? throw new ArgumentNullException(nameof(gl));
            Width = width;
            Height = height;
            
            _meshes = new List<Mesh>();
            
            Initialize();
        }

        private void Initialize()
        {
            // Setup OpenGL state
            _gl.Enable(EnableCap.DepthTest);
            _gl.Enable(EnableCap.Blend);
            _gl.BlendFunc(BlendingFactor.SrcAlpha, BlendingFactor.OneMinusSrcAlpha);
            
            // Initialize camera
            _camera = new Camera3D(
                new Vector3(10, 10, 10),  // Position
                new Vector3(0, 0, 0),      // Target
                Vector3.UnitY,             // Up
                45.0f,                     // FOV
                (float)Width / Height,     // Aspect
                0.1f,                      // Near
                1000.0f                    // Far
            );
            
            // Initialize renderer
            _renderer = new Renderer(_gl);
            
            // Create grid
            CreateGrid();
            
            // Create axes
            CreateAxes();
        }

        private void CreateGrid()
        {
            // Grid shader
            string vertexSource = @"
#version 330 core
layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aColor;

uniform mat4 uMVP;

out vec3 vColor;

void main()
{
    gl_Position = uMVP * vec4(aPosition, 1.0);
    vColor = aColor;
}
";

            string fragmentSource = @"
#version 330 core
in vec3 vColor;
out vec4 FragColor;

void main()
{
    FragColor = vec4(vColor, 1.0);
}
";

            _gridShader = new Shader(_gl, vertexSource, fragmentSource);

            // Generate grid vertices (lines on XZ plane)
            float size = 50.0f;
            float step = 5.0f;
            var vertices = new List<float>();
            
            // Major grid lines (darker)
            for (float i = -size; i <= size; i += step)
            {
                // Lines parallel to X axis
                vertices.Add(-size); vertices.Add(0); vertices.Add(i);
                vertices.Add(0.3f); vertices.Add(0.3f); vertices.Add(0.3f); // Color
                vertices.Add(size); vertices.Add(0); vertices.Add(i);
                vertices.Add(0.3f); vertices.Add(0.3f); vertices.Add(0.3f);
                
                // Lines parallel to Z axis
                vertices.Add(i); vertices.Add(0); vertices.Add(-size);
                vertices.Add(0.3f); vertices.Add(0.3f); vertices.Add(0.3f);
                vertices.Add(i); vertices.Add(0); vertices.Add(size);
                vertices.Add(0.3f); vertices.Add(0.3f); vertices.Add(0.3f);
            }
            
            _gridVertexCount = vertices.Count / 6;
            
            // Create VAO/VBO
            _gridVao = _gl.GenVertexArray();
            _gridVbo = _gl.GenBuffer();
            
            _gl.BindVertexArray(_gridVao);
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _gridVbo);
            
            fixed (float* ptr = vertices.ToArray())
            {
                _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Count * sizeof(float)), ptr, BufferUsageARB.StaticDraw);
            }
            
            // Position attribute
            _gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)0);
            _gl.EnableVertexAttribArray(0);
            
            // Color attribute
            _gl.VertexAttribPointer(1, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)(3 * sizeof(float)));
            _gl.EnableVertexAttribArray(1);
            
            _gl.BindVertexArray(0);
        }

        private void CreateAxes()
        {
            // Axes shader
            string vertexSource = @"
#version 330 core
layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aColor;

uniform mat4 uMVP;

out vec3 vColor;

void main()
{
    gl_Position = uMVP * vec4(aPosition, 1.0);
    vColor = aColor;
}
";

            string fragmentSource = @"
#version 330 core
in vec3 vColor;
out vec4 FragColor;

void main()
{
    FragColor = vec4(vColor, 1.0);
}
";

            _axesShader = new Shader(_gl, vertexSource, fragmentSource);

            // X (red), Y (green), Z (blue) axes
            float axisLength = 2.0f;
            var vertices = new float[]
            {
                // X axis - Red
                0, 0, 0,  1, 0, 0,
                axisLength, 0, 0,  1, 0, 0,
                
                // Y axis - Green
                0, 0, 0,  0, 1, 0,
                0, axisLength, 0,  0, 1, 0,
                
                // Z axis - Blue
                0, 0, 0,  0, 0, 1,
                0, 0, axisLength,  0, 0, 1,
            };
            
            // Create VAO/VBO
            _axesVao = _gl.GenVertexArray();
            _axesVbo = _gl.GenBuffer();
            
            _gl.BindVertexArray(_axesVao);
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _axesVbo);
            
            fixed (float* ptr = vertices)
            {
                _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), ptr, BufferUsageARB.StaticDraw);
            }
            
            // Position attribute
            _gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)0);
            _gl.EnableVertexAttribArray(0);
            
            // Color attribute
            _gl.VertexAttribPointer(1, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)(3 * sizeof(float)));
            _gl.EnableVertexAttribArray(1);
            
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Render the viewport
        /// </summary>
        public void Render()
        {
            if (_disposed) return;

            // Clear
            _gl.ClearColor(BackgroundColor.R, BackgroundColor.G, BackgroundColor.B, BackgroundColor.A);
            _gl.Clear(ClearBufferMask.ColorBufferBit | ClearBufferMask.DepthBufferBit);
            
            // Enable depth testing
            _gl.Enable(EnableCap.DepthTest);

            // Calculate view-projection matrix
            Matrix4x4 viewProj = _camera.GetViewMatrix() * _camera.GetProjectionMatrix();

            // Render grid
            if (ShowGrid)
            {
                RenderGrid(viewProj);
            }

            // Render axes
            if (ShowAxes)
            {
                RenderAxes(viewProj);
            }

            // Render meshes
            foreach (var mesh in _meshes)
            {
                if (mesh.IsVisible)
                {
                    mesh.Render(_gl);
                }
            }
        }

        private void RenderGrid(Matrix4x4 viewProj)
        {
            _gridShader.Use();
            _gridShader.SetMatrix4("uMVP", viewProj);
            
            _gl.BindVertexArray(_gridVao);
            _gl.DrawArrays(PrimitiveType.Lines, 0, (uint)_gridVertexCount);
            _gl.BindVertexArray(0);
        }

        private void RenderAxes(Matrix4x4 viewProj)
        {
            _axesShader.Use();
            _axesShader.SetMatrix4("uMVP", viewProj);
            
            _gl.BindVertexArray(_axesVao);
            _gl.DrawArrays(PrimitiveType.Lines, 0, 6); // 6 vertices (3 axes * 2 points)
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Resize viewport
        /// </summary>
        public void Resize(int width, int height)
        {
            Width = width;
            Height = height;
            _gl.Viewport(0, 0, (uint)width, (uint)height);
            _camera.Aspect = (float)width / height;
        }

        /// <summary>
        /// Add a mesh to the viewport
        /// </summary>
        public void AddMesh(Mesh mesh)
        {
            _meshes.Add(mesh);
        }

        /// <summary>
        /// Remove a mesh from the viewport
        /// </summary>
        public void RemoveMesh(Mesh mesh)
        {
            _meshes.Remove(mesh);
        }

        /// <summary>
        /// Clear all meshes
        /// </summary>
        public void ClearMeshes()
        {
            _meshes.Clear();
        }

        /// <summary>
        /// Fit view to show all objects
        /// </summary>
        public void FitView()
        {
            // Calculate bounding box of all meshes
            var bbox = new BoundingBox(
                new Vector3(float.MaxValue, float.MaxValue, float.MaxValue),
                new Vector3(float.MinValue, float.MinValue, float.MinValue)
            );
            bool hasMeshes = false;
            
            foreach (var mesh in _meshes)
            {
                if (mesh.IsVisible)
                {
                    bbox.Expand(mesh.GetBoundingBox());
                    hasMeshes = true;
                }
            }
            
            if (hasMeshes)
            {
                Vector3 center = bbox.Center;
                float size = bbox.Size.Length();
                float distance = size * 1.5f;
                
                _camera.Position = center + new Vector3(distance, distance * 0.5f, distance);
                _camera.Target = center;
            }
        }

        /// <summary>
        /// Set standard view
        /// </summary>
        public void SetStandardView(StandardView view)
        {
            _camera.SetStandardView(view);
        }

        /// <summary>
        /// Handle mouse down for camera control and picking
        /// </summary>
        public void OnMouseDown(ViewportMouseButton button, int x, int y)
        {
            if (button == ViewportMouseButton.Left)
            {
                // Left click - could be selection
                // For now, just log or implement picking later
            }
            else if (button == ViewportMouseButton.Middle)
            {
                // Middle button - start pan
                _camera.StartPan(x, y);
            }
            else if (button == ViewportMouseButton.Right)
            {
                // Right button - start rotate
                _camera.StartRotate(x, y);
            }
        }

        /// <summary>
        /// Handle mouse move
        /// </summary>
        public void OnMouseMove(int x, int y, bool leftButton, bool middleButton, bool rightButton)
        {
            if (middleButton)
            {
                _camera.Pan(x, y);
            }
            else if (rightButton)
            {
                _camera.Rotate(x, y);
            }
        }

        /// <summary>
        /// Handle mouse up
        /// </summary>
        public void OnMouseUp(ViewportMouseButton button)
        {
            _camera.EndInteraction();
        }

        /// <summary>
        /// Handle mouse wheel
        /// </summary>
        public void OnMouseWheel(float delta)
        {
            _camera.Zoom(delta * 0.1f);
        }

        /// <summary>
        /// Pick entity at screen coordinates
        /// </summary>
        private uint? PickEntity(int x, int y)
        {
            // TODO: Implement ray casting
            return null;
        }

        public void Dispose()
        {
            if (_disposed) return;

            _gridShader?.Dispose();
            _axesShader?.Dispose();
            _renderer?.Dispose();
            
            _gl.DeleteBuffer(_gridVbo);
            _gl.DeleteVertexArray(_gridVao);
            _gl.DeleteBuffer(_axesVbo);
            _gl.DeleteVertexArray(_axesVao);

            _disposed = true;
        }
    }

    public enum StandardView
    {
        Front,
        Back,
        Top,
        Bottom,
        Left,
        Right,
        Isometric,
        Dimetric,
        Trimetric
    }

    public class ViewportClickEventArgs : Avalonia.Interactivity.RoutedEventArgs
    {
        public uint EntityId { get; }
        public int X { get; }
        public int Y { get; }

        public ViewportClickEventArgs(uint entityId, int x, int y)
        {
            EntityId = entityId;
            X = x;
            Y = y;
        }
    }

    public class ViewportHoverEventArgs : EventArgs
    {
        public uint? EntityId { get; }
        public int X { get; }
        public int Y { get; }

        public ViewportHoverEventArgs(uint? entityId, int x, int y)
        {
            EntityId = entityId;
            X = x;
            Y = y;
        }
    }

    public enum ViewportMouseButton
    {
        Left,
        Middle,
        Right
    }
}
