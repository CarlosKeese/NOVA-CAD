using System;
using System.Collections.Generic;
using System.Numerics;
using Avalonia.OpenGL;
using static NovaCad.Viewport.GlConstants;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Mesh for 3D rendering
    /// </summary>
    public unsafe class Mesh : IDisposable
    {
        private GlInterface _gl;
        private int _vao;
        private int _vbo;
        private int _ebo;
        private int _indexCount;
        private bool _initialized;
        private bool _disposed;

        // Mesh data
        public List<Vertex> Vertices { get; set; }
        public List<uint> Indices { get; set; }
        
        // Transform
        public Matrix4x4 Transform { get; set; } = Matrix4x4.Identity;
        
        // Appearance
        public Color Color { get; set; } = new Color(0.7f, 0.7f, 0.7f, 1.0f);
        public float Shininess { get; set; } = 32.0f;
        public bool Visible { get; set; } = true;
        public bool IsVisible { get => Visible; set => Visible = value; }
        public bool IsSelected { get; set; }
        public bool IsInitialized => _initialized;
        
        // Identification
        public uint EntityId { get; set; }
        public string Name { get; set; } = "Mesh";
        
        // Bounding box (cached)
        private BoundingBox _boundingBox;
        private bool _bboxDirty = true;

        public BoundingBox BoundingBox
        {
            get
            {
                if (_bboxDirty)
                {
                    RecalculateBoundingBox();
                }
                return _boundingBox;
            }
        }

        public BoundingBox GetBoundingBox() => BoundingBox;

        public Mesh()
        {
            Vertices = new List<Vertex>();
            Indices = new List<uint>();
        }

        public Mesh(List<Vertex> vertices, List<uint> indices)
        {
            Vertices = vertices;
            Indices = indices;
        }

        /// <summary>
        /// Initialize OpenGL buffers
        /// </summary>
        public void Initialize(GlInterface gl)
        {
            if (_initialized || Vertices.Count == 0) return;

            _gl = gl;
            GlExtensions.Initialize(gl);

            // Create VAO
            _vao = gl.GenVertexArray();
            gl.BindVertexArray(_vao);

            // Create VBO
            _vbo = gl.GenBuffer();
            gl.BindBuffer(GL_ARRAY_BUFFER, _vbo);
            
            // Upload vertex data
            var vertexData = GetVertexData();
            fixed (float* ptr = vertexData)
            {
                gl.BufferData(GL_ARRAY_BUFFER, vertexData.Length * sizeof(float), new IntPtr(ptr), GL_STATIC_DRAW);
            }

            // Create EBO
            _ebo = gl.GenBuffer();
            gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, _ebo);
            
            // Upload index data
            var indexData = Indices.ToArray();
            _indexCount = indexData.Length;
            fixed (uint* ptr = indexData)
            {
                gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, indexData.Length * sizeof(uint), new IntPtr(ptr), GL_STATIC_DRAW);
            }

            // Set up vertex attributes
            // Position (3 floats)
            gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 8 * sizeof(float), IntPtr.Zero);
            gl.EnableVertexAttribArray(0);

            // Normal (3 floats)
            gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 8 * sizeof(float), new IntPtr(3 * sizeof(float)));
            gl.EnableVertexAttribArray(1);

            // TexCoord (2 floats)
            gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, 8 * sizeof(float), new IntPtr(6 * sizeof(float)));
            gl.EnableVertexAttribArray(2);

            // Unbind
            gl.BindVertexArray(0);

            _initialized = true;
        }

        /// <summary>
        /// Render the mesh
        /// </summary>
        public void Render(GlInterface gl)
        {
            if (!_initialized || _disposed) return;

            gl.BindVertexArray(_vao);
            gl.DrawElements(GL_TRIANGLES, _indexCount, GL_UNSIGNED_INT, IntPtr.Zero);
            gl.BindVertexArray(0);
        }

        /// <summary>
        /// Update vertex data
        /// </summary>
        public void UpdateVertices(GlInterface gl)
        {
            if (!_initialized) return;

            gl.BindBuffer(GL_ARRAY_BUFFER, _vbo);
            var vertexData = GetVertexData();
            fixed (float* ptr = vertexData)
            {
                gl.BufferData(GL_ARRAY_BUFFER, vertexData.Length * sizeof(float), new IntPtr(ptr), GL_STATIC_DRAW);
            }

            _bboxDirty = true;
        }

        /// <summary>
        /// Ray intersection test
        /// </summary>
        public bool IntersectRay(Ray ray, out float distance, out uint faceId)
        {
            distance = float.MaxValue;
            faceId = 0;
            bool hit = false;

            // Test each triangle
            for (int i = 0; i < Indices.Count; i += 3)
            {
                uint i0 = Indices[i];
                uint i1 = Indices[i + 1];
                uint i2 = Indices[i + 2];

                Vector3 v0 = Vertices[(int)i0].Position;
                Vector3 v1 = Vertices[(int)i1].Position;
                Vector3 v2 = Vertices[(int)i2].Position;

                // Transform vertices
                v0 = Vector3.Transform(v0, Transform);
                v1 = Vector3.Transform(v1, Transform);
                v2 = Vector3.Transform(v2, Transform);

                if (RayTriangleIntersect(ray, v0, v1, v2, out float t))
                {
                    if (t < distance)
                    {
                        distance = t;
                        faceId = (uint)(i / 3);
                        hit = true;
                    }
                }
            }

            return hit;
        }

        private float[] GetVertexData()
        {
            var data = new float[Vertices.Count * 8];
            int index = 0;

            foreach (var v in Vertices)
            {
                data[index++] = v.Position.X;
                data[index++] = v.Position.Y;
                data[index++] = v.Position.Z;
                data[index++] = v.Normal.X;
                data[index++] = v.Normal.Y;
                data[index++] = v.Normal.Z;
                data[index++] = v.TexCoord.X;
                data[index++] = v.TexCoord.Y;
            }

            return data;
        }

        private void RecalculateBoundingBox()
        {
            if (Vertices.Count == 0)
            {
                _boundingBox = new BoundingBox(Vector3.Zero, Vector3.Zero);
                return;
            }

            Vector3 min = Vertices[0].Position;
            Vector3 max = Vertices[0].Position;

            foreach (var v in Vertices)
            {
                min = Vector3.Min(min, v.Position);
                max = Vector3.Max(max, v.Position);
            }

            _boundingBox = new BoundingBox(min, max);
            _bboxDirty = false;
        }

        private bool RayTriangleIntersect(Ray ray, Vector3 v0, Vector3 v1, Vector3 v2, out float t)
        {
            t = 0;

            Vector3 edge1 = v1 - v0;
            Vector3 edge2 = v2 - v0;
            Vector3 h = Vector3.Cross(ray.Direction, edge2);
            float a = Vector3.Dot(edge1, h);

            if (a > -float.Epsilon && a < float.Epsilon)
                return false;

            float f = 1.0f / a;
            Vector3 s = ray.Origin - v0;
            float u = f * Vector3.Dot(s, h);

            if (u < 0.0 || u > 1.0)
                return false;

            Vector3 q = Vector3.Cross(s, edge1);
            float v = f * Vector3.Dot(ray.Direction, q);

            if (v < 0.0 || u + v > 1.0)
                return false;

            t = f * Vector3.Dot(edge2, q);

            return t > float.Epsilon;
        }

        public void Dispose()
        {
            if (_disposed) return;

            if (_initialized)
            {
                _gl.DeleteVertexArray(_vao);
                _gl.DeleteBuffer(_vbo);
                _gl.DeleteBuffer(_ebo);
            }

            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }

    /// <summary>
    /// Vertex structure
    /// </summary>
    public struct Vertex
    {
        public Vector3 Position;
        public Vector3 Normal;
        public Vector2 TexCoord;

        public Vertex(Vector3 position, Vector3 normal, Vector2 texCoord)
        {
            Position = position;
            Normal = normal;
            TexCoord = texCoord;
        }
    }


}
