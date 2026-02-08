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
    public class Viewport3D : IDisposable
    {
        private GL _gl;
        private uint _vao;
        private uint _vbo;
        private uint _ebo;
        private uint _shaderProgram;
        
        private Camera3D _camera;
        private Renderer _renderer;
        private Shader _shader;
        
        private List<Mesh> _meshes;
        private bool _disposed;
        
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
        public bool ShowNormals { get; set; } = false;
        public bool EnableLighting { get; set; } = true;
        public bool EnableShadows { get; set; } = false;
        
        // Events
        public event EventHandler<ViewportClickEventArgs>? EntityPicked;
        public event EventHandler<ViewportHoverEventArgs>? EntityHovered;

        public Viewport3D(GL gl, int width, int height)
        {
            _gl = gl ?? throw new ArgumentNullException(nameof(gl));
            Width = width;
            Height = height;
            
            _meshes = new List<Mesh>();
            _camera = new Camera3D(width, height);
            _renderer = new Renderer(gl);
            
            Initialize();
        }

        private void Initialize()
        {
            // Create shader
            _shader = new Shader(_gl, GetVertexShaderSource(), GetFragmentShaderSource());
            _shader.Use();
            
            // Create default meshes
            CreateGrid();
            CreateAxes();
        }

        /// <summary>
        /// Resize the viewport
        /// </summary>
        public void Resize(int width, int height)
        {
            Width = width;
            Height = height;
            _camera.Resize(width, height);
            _gl.Viewport(0, 0, (uint)width, (uint)height);
        }

        /// <summary>
        /// Render the viewport
        /// </summary>
        public void Render()
        {
            if (_disposed) return;

            // Clear
            _gl.ClearColor(BackgroundColor.R, BackgroundColor.G, BackgroundColor.B, BackgroundColor.A);
            _gl.Clear((uint)(ClearBufferMask.ColorBufferBit | ClearBufferMask.DepthBufferBit));
            
            // Enable depth testing
            _gl.Enable(EnableCap.DepthTest);
            
            // Set polygon mode (disabled - MaterialFace not available in current Silk.NET version)
            // _gl.PolygonMode(MaterialFace.FrontAndBack, 
            //     Wireframe ? PolygonMode.Line : PolygonMode.Fill);

            // Update camera
            _shader.Use();
            _shader.SetMatrix4("uView", _camera.GetViewMatrix());
            _shader.SetMatrix4("uProjection", _camera.GetProjectionMatrix());
            _shader.SetVector3("uCameraPosition", _camera.Position);

            // Render grid
            if (ShowGrid)
            {
                RenderGrid();
            }

            // Render axes
            if (ShowAxes)
            {
                RenderAxes();
            }

            // Render meshes
            foreach (var mesh in _meshes)
            {
                RenderMesh(mesh);
            }

            // Render selection highlight
            if (SelectedEntityId.HasValue)
            {
                RenderSelectionHighlight(SelectedEntityId.Value);
            }
        }

        /// <summary>
        /// Add a mesh to the viewport
        /// </summary>
        public void AddMesh(Mesh mesh)
        {
            _meshes.Add(mesh);
            mesh.Initialize(_gl);
        }

        /// <summary>
        /// Remove a mesh from the viewport
        /// </summary>
        public void RemoveMesh(Mesh mesh)
        {
            _meshes.Remove(mesh);
            mesh.Dispose();
        }

        /// <summary>
        /// Clear all meshes
        /// </summary>
        public void ClearMeshes()
        {
            foreach (var mesh in _meshes)
            {
                mesh.Dispose();
            }
            _meshes.Clear();
        }

        /// <summary>
        /// Handle mouse down for camera control and picking
        /// </summary>
        public void OnMouseDown(ViewportMouseButton button, int x, int y)
        {
            if (button == ViewportMouseButton.Left)
            {
                // Pick entity
                var pickedId = PickEntity(x, y);
                if (pickedId.HasValue)
                {
                    SelectedEntityId = pickedId;
                    EntityPicked?.Invoke(this, new ViewportClickEventArgs(pickedId.Value, x, y));
                }
            }
            else if (button == ViewportMouseButton.Middle)
            {
                // Start pan
                _camera.StartPan(x, y);
            }
            else if (button == ViewportMouseButton.Right)
            {
                // Start rotate
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
            else
            {
                // Hover
                var hoveredId = PickEntity(x, y);
                if (hoveredId.HasValue)
                {
                    EntityHovered?.Invoke(this, new ViewportHoverEventArgs(hoveredId.Value, x, y));
                }
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
        /// Handle mouse wheel for zoom
        /// </summary>
        public void OnMouseWheel(float delta)
        {
            _camera.Zoom(delta);
        }

        /// <summary>
        /// Fit view to show all geometry
        /// </summary>
        public void FitView()
        {
            if (_meshes.Count == 0) return;

            var bbox = new BoundingBox();
            foreach (var mesh in _meshes)
            {
                bbox.Expand(mesh.BoundingBox);
            }

            _camera.FitToBoundingBox(bbox);
        }

        /// <summary>
        /// Set camera to standard view
        /// </summary>
        public void SetStandardView(StandardView view)
        {
            _camera.SetStandardView(view);
        }

        /// <summary>
        /// Pick entity at screen coordinates
        /// </summary>
        private uint? PickEntity(int x, int y)
        {
            // Convert screen coordinates to normalized device coordinates
            float ndcX = (2.0f * x / Width) - 1.0f;
            float ndcY = 1.0f - (2.0f * y / Height);

            // Create ray from camera
            var ray = _camera.GetRay(ndcX, ndcY);

            // Find closest intersection
            float closestDistance = float.MaxValue;
            uint? closestId = null;

            foreach (var mesh in _meshes)
            {
                if (mesh.IntersectRay(ray, out float distance, out uint faceId))
                {
                    if (distance < closestDistance)
                    {
                        closestDistance = distance;
                        closestId = mesh.EntityId;
                    }
                }
            }

            return closestId;
        }

        private void RenderMesh(Mesh mesh)
        {
            if (!mesh.Visible) return;

            // Set model matrix
            _shader.SetMatrix4("uModel", mesh.Transform);
            
            // Set material properties
            var color = mesh.IsSelected ? new Color(1.0f, 0.5f, 0.0f, 1.0f) : mesh.Color;
            _shader.SetVector4("uColor", color.ToVector4());
            _shader.SetFloat("uShininess", mesh.Shininess);
            _shader.SetBool("uEnableLighting", EnableLighting);

            // Render
            mesh.Render(_gl);
        }

        private void RenderGrid()
        {
            // Render ground grid
            _renderer.RenderGrid(_camera, 100.0f, 10.0f);
        }

        private void RenderAxes()
        {
            // Render XYZ axes (TODO: Implement)
            // _renderer.RenderAxes(_camera, 10.0f);
        }

        private void RenderSelectionHighlight(uint entityId)
        {
            // Render highlight around selected entity (TODO: Implement)
            // _renderer.RenderHighlight(entityId, new Color(1.0f, 0.5f, 0.0f, 0.5f));
        }

        private void CreateGrid()
        {
            // Create default grid geometry
        }

        private void CreateAxes()
        {
            // Create default axes geometry
        }

        private string GetVertexShaderSource()
        {
            return @"
#version 330 core
layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 uModel;
uniform mat4 uView;
uniform mat4 uProjection;

out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vTexCoord;

void main()
{
    vec4 worldPos = uModel * vec4(aPosition, 1.0);
    vWorldPos = worldPos.xyz;
    vNormal = mat3(transpose(inverse(uModel))) * aNormal;
    vTexCoord = aTexCoord;
    gl_Position = uProjection * uView * worldPos;
}
";
        }

        private string GetFragmentShaderSource()
        {
            return @"
#version 330 core
in vec3 vWorldPos;
in vec3 vNormal;
in vec2 vTexCoord;

uniform vec4 uColor;
uniform vec3 uCameraPosition;
uniform bool uEnableLighting;
uniform float uShininess;

out vec4 FragColor;

void main()
{
    vec4 color = uColor;
    
    if (uEnableLighting)
    {
        // Simple Phong lighting
        vec3 normal = normalize(vNormal);
        vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
        vec3 viewDir = normalize(uCameraPosition - vWorldPos);
        vec3 reflectDir = reflect(-lightDir, normal);
        
        float ambient = 0.3;
        float diffuse = max(dot(normal, lightDir), 0.0) * 0.5;
        float specular = pow(max(dot(viewDir, reflectDir), 0.0), uShininess) * 0.2;
        
        float lighting = ambient + diffuse + specular;
        color.rgb *= lighting;
    }
    
    FragColor = color;
}
";
        }

        public void Dispose()
        {
            if (_disposed) return;
            
            _shader?.Dispose();
            _renderer?.Dispose();
            
            foreach (var mesh in _meshes)
            {
                mesh.Dispose();
            }
            _meshes.Clear();
            
            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }

    public enum ViewportMouseButton
    {
        Left,
        Middle,
        Right
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
        public uint EntityId { get; }
        public int X { get; }
        public int Y { get; }

        public ViewportHoverEventArgs(uint entityId, int x, int y)
        {
            EntityId = entityId;
            X = x;
            Y = y;
        }
    }
}
