using System;
using System.Collections.Generic;
using System.Numerics;
using Avalonia.OpenGL;
using static NovaCad.Viewport.GlConstants;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D Viewport for rendering CAD geometry using OpenGL
    /// </summary>
    public unsafe class Viewport3D : IDisposable
    {
        private GlInterface _gl;
        private Camera3D _camera;
        
        private List<Mesh> _meshes;
        private bool _disposed;
        
        // Grid rendering
        private int _gridVao;
        private int _gridVbo;
        private int _gridVertexCount;
        private int _gridShader;
        
        // Axes rendering
        private int _axesVao;
        private int _axesVbo;
        private int _axesShader;
        
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

        public Viewport3D(GlInterface gl, int width, int height)
        {
            _gl = gl ?? throw new ArgumentNullException(nameof(gl));
            Width = width;
            Height = height;
            
            _meshes = new List<Mesh>();
            
            Initialize();
        }

        private void Initialize()
        {
            // Initialize extension methods
            GlExtensions.Initialize(_gl);

            // Setup OpenGL state
            _gl.Enable(GL_DEPTH_TEST);
            _gl.Enable(GL_BLEND);
            _gl.BlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
            
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

            _gridShader = CreateShaderProgram(vertexSource, fragmentSource);

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
            _gl.BindBuffer(GL_ARRAY_BUFFER, _gridVbo);
            
            fixed (float* ptr = vertices.ToArray())
            {
                _gl.BufferData(GL_ARRAY_BUFFER, vertices.Count * sizeof(float), new IntPtr(ptr), GL_STATIC_DRAW);
            }
            
            // Position attribute
            _gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), IntPtr.Zero);
            _gl.EnableVertexAttribArray(0);
            
            // Color attribute
            _gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), new IntPtr(3 * sizeof(float)));
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

            _axesShader = CreateShaderProgram(vertexSource, fragmentSource);

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
            _gl.BindBuffer(GL_ARRAY_BUFFER, _axesVbo);
            
            fixed (float* ptr = vertices)
            {
                _gl.BufferData(GL_ARRAY_BUFFER, vertices.Length * sizeof(float), new IntPtr(ptr), GL_STATIC_DRAW);
            }
            
            // Position attribute
            _gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), IntPtr.Zero);
            _gl.EnableVertexAttribArray(0);
            
            // Color attribute
            _gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 6 * sizeof(float), new IntPtr(3 * sizeof(float)));
            _gl.EnableVertexAttribArray(1);
            
            _gl.BindVertexArray(0);
        }

        private int CreateShaderProgram(string vertexSource, string fragmentSource)
        {
            int vertexShader = _gl.CreateShader(GL_VERTEX_SHADER);
            _gl.ShaderSource(vertexShader, vertexSource);
            _gl.CompileShader(vertexShader);

            int fragmentShader = _gl.CreateShader(GL_FRAGMENT_SHADER);
            _gl.ShaderSource(fragmentShader, fragmentSource);
            _gl.CompileShader(fragmentShader);

            int program = _gl.CreateProgram();
            _gl.AttachShader(program, vertexShader);
            _gl.AttachShader(program, fragmentShader);
            _gl.LinkProgram(program);

            _gl.DeleteShader(vertexShader);
            _gl.DeleteShader(fragmentShader);

            return program;
        }

        /// <summary>
        /// Render the viewport
        /// </summary>
        public void Render(GlInterface gl)
        {
            if (_disposed) return;

            _gl = gl; // Update GL interface

            // Clear
            _gl.ClearColor(BackgroundColor.R, BackgroundColor.G, BackgroundColor.B, BackgroundColor.A);
            _gl.Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            
            // Enable depth testing
            _gl.Enable(GL_DEPTH_TEST);

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
            _gl.UseProgram(_gridShader);
            
            int mvpLocation = _gl.GetUniformLocationString(_gridShader, "uMVP");
            if (mvpLocation >= 0)
            {
                float* matrixData = stackalloc float[16];
                for (int i = 0; i < 16; i++)
                    matrixData[i] = viewProj[i / 4, i % 4];
                _gl.UniformMatrix4fv(mvpLocation, 1, GL_FALSE, matrixData);
            }
            
            _gl.BindVertexArray(_gridVao);
            _gl.DrawArrays(GL_LINES, 0, _gridVertexCount);
            _gl.BindVertexArray(0);
        }

        private void RenderAxes(Matrix4x4 viewProj)
        {
            _gl.UseProgram(_axesShader);
            
            int mvpLocation = _gl.GetUniformLocationString(_axesShader, "uMVP");
            if (mvpLocation >= 0)
            {
                float* matrixData = stackalloc float[16];
                for (int i = 0; i < 16; i++)
                    matrixData[i] = viewProj[i / 4, i % 4];
                _gl.UniformMatrix4fv(mvpLocation, 1, GL_FALSE, matrixData);
            }
            
            _gl.BindVertexArray(_axesVao);
            _gl.DrawArrays(GL_LINES, 0, 6); // 6 vertices (3 axes * 2 points)
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Resize viewport
        /// </summary>
        public void Resize(int width, int height)
        {
            Width = width;
            Height = height;
            _gl.Viewport(0, 0, width, height);
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

        public void Dispose()
        {
            if (_disposed) return;

            _gl.DeleteProgram(_gridShader);
            _gl.DeleteProgram(_axesShader);
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
}
