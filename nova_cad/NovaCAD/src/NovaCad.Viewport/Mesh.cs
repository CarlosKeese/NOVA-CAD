using System;
using System.Collections.Generic;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Mesh for 3D rendering
    /// </summary>
    public class Mesh : IDisposable
    {
        private GL _gl;
        private uint _vao;
        private uint _vbo;
        private uint _ebo;
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
        public bool IsSelected { get; set; }
        
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
        public void Initialize(GL gl)
        {
            if (_initialized || Vertices.Count == 0) return;

            _gl = gl;

            // Create VAO
            _vao = gl.GenVertexArray();
            gl.BindVertexArray(_vao);

            // Create VBO
            _vbo = gl.GenBuffer();
            gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            
            // Upload vertex data
            var vertexData = GetVertexData();
            gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertexData.Length * sizeof(float)), vertexData, BufferUsageARB.StaticDraw);

            // Create EBO
            _ebo = gl.GenBuffer();
            gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, _ebo);
            
            // Upload index data
            var indexData = Indices.ToArray();
            _indexCount = indexData.Length;
            gl.BufferData(BufferTargetARB.ElementArrayBuffer, (nuint)(indexData.Length * sizeof(uint)), indexData, BufferUsageARB.StaticDraw);

            // Set up vertex attributes
            // Position (3 floats)
            gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, (uint)(8 * sizeof(float)), (void*)0);
            gl.EnableVertexAttribArray(0);

            // Normal (3 floats)
            gl.VertexAttribPointer(1, 3, VertexAttribPointerType.Float, false, (uint)(8 * sizeof(float)), (void*)(3 * sizeof(float)));
            gl.EnableVertexAttribArray(1);

            // TexCoord (2 floats)
            gl.VertexAttribPointer(2, 2, VertexAttribPointerType.Float, false, (uint)(8 * sizeof(float)), (void*)(6 * sizeof(float)));
            gl.EnableVertexAttribArray(2);

            // Unbind
            gl.BindVertexArray(0);
            gl.BindBuffer(BufferTargetARB.ArrayBuffer, 0);
            gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, 0);

            _initialized = true;
        }

        /// <summary>
        /// Render the mesh
        /// </summary>
        public void Render(GL gl)
        {
            if (!_initialized || _disposed) return;

            gl.BindVertexArray(_vao);
            gl.DrawElements(PrimitiveType.Triangles, (uint)_indexCount, DrawElementsType.UnsignedInt, (void*)0);
            gl.BindVertexArray(0);
        }

        /// <summary>
        /// Update vertex data
        /// </summary>
        public void UpdateVertices(GL gl)
        {
            if (!_initialized) return;

            gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            var vertexData = GetVertexData();
            gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertexData.Length * sizeof(float)), vertexData, BufferUsageARB.StaticDraw);
            gl.BindBuffer(BufferTargetARB.ArrayBuffer, 0);

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

        /// <summary>
        /// Create mesh from kernel body
        /// </summary>
        public static Mesh FromBody(NovaKernel.NovaBodyRef body, GL gl)
        {
            var mesh = new Mesh();
            
            // Get tessellated mesh from kernel
            var kernelMesh = NovaKernel.TessellateBody(body);
            if (kernelMesh == null) return mesh;

            // Convert kernel mesh to viewport mesh
            for (int i = 0; i < kernelMesh.VertexCount; i++)
            {
                var v = kernelMesh.GetVertex(i);
                mesh.Vertices.Add(new Vertex
                {
                    Position = new Vector3(v.X, v.Y, v.Z),
                    Normal = new Vector3(v.NX, v.NY, v.NZ),
                    TexCoord = new Vector2(v.U, v.V)
                });
            }

            for (int i = 0; i < kernelMesh.IndexCount; i++)
            {
                mesh.Indices.Add(kernelMesh.GetIndex(i));
            }

            mesh.EntityId = (uint)body.Handle;
            mesh.Initialize(gl);

            return mesh;
        }

        /// <summary>
        /// Create a cube mesh
        /// </summary>
        public static Mesh CreateCube(GL gl, float size)
        {
            float half = size / 2.0f;
            
            var mesh = new Mesh();
            
            // Vertices
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, -half), Normal = new Vector3(0, 0, -1), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, -half), Normal = new Vector3(0, 0, -1), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, -half), Normal = new Vector3(0, 0, -1), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, -half), Normal = new Vector3(0, 0, -1), TexCoord = new Vector2(0, 1) });

            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, half), Normal = new Vector3(0, 0, 1), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, half), Normal = new Vector3(0, 0, 1), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, half), Normal = new Vector3(0, 0, 1), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, half), Normal = new Vector3(0, 0, 1), TexCoord = new Vector2(0, 1) });

            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, half), Normal = new Vector3(0, 1, 0), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, half), Normal = new Vector3(0, 1, 0), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, -half), Normal = new Vector3(0, 1, 0), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, -half), Normal = new Vector3(0, 1, 0), TexCoord = new Vector2(0, 1) });

            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, half), Normal = new Vector3(0, -1, 0), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, half), Normal = new Vector3(0, -1, 0), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, -half), Normal = new Vector3(0, -1, 0), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, -half), Normal = new Vector3(0, -1, 0), TexCoord = new Vector2(0, 1) });

            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, half), Normal = new Vector3(1, 0, 0), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, -half, -half), Normal = new Vector3(1, 0, 0), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, -half), Normal = new Vector3(1, 0, 0), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(half, half, half), Normal = new Vector3(1, 0, 0), TexCoord = new Vector2(0, 1) });

            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, -half), Normal = new Vector3(-1, 0, 0), TexCoord = new Vector2(0, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, -half, half), Normal = new Vector3(-1, 0, 0), TexCoord = new Vector2(1, 0) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, half), Normal = new Vector3(-1, 0, 0), TexCoord = new Vector2(1, 1) });
            mesh.Vertices.Add(new Vertex { Position = new Vector3(-half, half, -half), Normal = new Vector3(-1, 0, 0), TexCoord = new Vector2(0, 1) });

            // Indices
            for (uint i = 0; i < 6; i++)
            {
                uint base = i * 4;
                mesh.Indices.Add(base);
                mesh.Indices.Add(base + 1);
                mesh.Indices.Add(base + 2);
                mesh.Indices.Add(base);
                mesh.Indices.Add(base + 2);
                mesh.Indices.Add(base + 3);
            }

            mesh.Initialize(gl);
            return mesh;
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

            if (_initialized && _gl != null)
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

    /// <summary>
    /// Color structure
    /// </summary>
    public struct Color
    {
        public float R;
        public float G;
        public float B;
        public float A;

        public Color(float r, float g, float b, float a)
        {
            R = r;
            G = g;
            B = b;
            A = a;
        }

        public Vector4 ToVector4()
        {
            return new Vector4(R, G, B, A);
        }

        public static Color Red => new Color(1.0f, 0.0f, 0.0f, 1.0f);
        public static Color Green => new Color(0.0f, 1.0f, 0.0f, 1.0f);
        public static Color Blue => new Color(0.0f, 0.0f, 1.0f, 1.0f);
        public static Color White => new Color(1.0f, 1.0f, 1.0f, 1.0f);
        public static Color Black => new Color(0.0f, 0.0f, 0.0f, 1.0f);
        public static Color Gray => new Color(0.5f, 0.5f, 0.5f, 1.0f);
        public static Color Yellow => new Color(1.0f, 1.0f, 0.0f, 1.0f);
        public static Color Orange => new Color(1.0f, 0.5f, 0.0f, 1.0f);
    }
}
