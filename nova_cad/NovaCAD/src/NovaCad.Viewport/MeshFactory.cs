using System;
using System.Collections.Generic;
using System.Numerics;
using Avalonia.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Factory for creating primitive meshes
    /// </summary>
    public static class MeshFactory
    {
        /// <summary>
        /// Create a box mesh without OpenGL initialization (for deferred init)
        /// </summary>
        public static Mesh CreateBox(float width, float height, float depth)
        {
            float hx = width * 0.5f;
            float hy = height * 0.5f;
            float hz = depth * 0.5f;

            // Default color
            var color = new Color(0.8f, 0.8f, 0.8f, 1.0f);

            // 8 corners of the box
            Vector3[] corners = new Vector3[8]
            {
                new Vector3(-hx, -hy, -hz), // 0: bottom-left-back
                new Vector3( hx, -hy, -hz), // 1: bottom-right-back
                new Vector3( hx,  hy, -hz), // 2: top-right-back
                new Vector3(-hx,  hy, -hz), // 3: top-left-back
                new Vector3(-hx, -hy,  hz), // 4: bottom-left-front
                new Vector3( hx, -hy,  hz), // 5: bottom-right-front
                new Vector3( hx,  hy,  hz), // 6: top-right-front
                new Vector3(-hx,  hy,  hz), // 7: top-left-front
            };

            // Normals for each face
            Vector3 backNormal = new Vector3(0, 0, -1);
            Vector3 frontNormal = new Vector3(0, 0, 1);
            Vector3 leftNormal = new Vector3(-1, 0, 0);
            Vector3 rightNormal = new Vector3(1, 0, 0);
            Vector3 bottomNormal = new Vector3(0, -1, 0);
            Vector3 topNormal = new Vector3(0, 1, 0);

            var vertices = new List<Vertex>();
            var indices = new List<uint>();

            // Helper to add a face (2 triangles)
            void AddFace(Vector3 v0, Vector3 v1, Vector3 v2, Vector3 v3, Vector3 normal)
            {
                uint baseIndex = (uint)vertices.Count;
                
                vertices.Add(new Vertex { Position = v0, Normal = normal, TexCoord = new Vector2(0, 0) });
                vertices.Add(new Vertex { Position = v1, Normal = normal, TexCoord = new Vector2(1, 0) });
                vertices.Add(new Vertex { Position = v2, Normal = normal, TexCoord = new Vector2(1, 1) });
                vertices.Add(new Vertex { Position = v3, Normal = normal, TexCoord = new Vector2(0, 1) });

                // Triangle 1
                indices.Add(baseIndex);
                indices.Add(baseIndex + 1);
                indices.Add(baseIndex + 2);

                // Triangle 2
                indices.Add(baseIndex);
                indices.Add(baseIndex + 2);
                indices.Add(baseIndex + 3);
            }

            // Back face (z-)
            AddFace(corners[0], corners[1], corners[2], corners[3], backNormal);
            // Front face (z+)
            AddFace(corners[5], corners[4], corners[7], corners[6], frontNormal);
            // Left face (x-)
            AddFace(corners[4], corners[0], corners[3], corners[7], leftNormal);
            // Right face (x+)
            AddFace(corners[1], corners[5], corners[6], corners[2], rightNormal);
            // Bottom face (y-)
            AddFace(corners[4], corners[5], corners[1], corners[0], bottomNormal);
            // Top face (y+)
            AddFace(corners[3], corners[2], corners[6], corners[7], topNormal);

            var mesh = new Mesh(vertices, indices);
            mesh.Color = color;
            return mesh;
        }

        /// <summary>
        /// Create a box mesh with OpenGL initialization
        /// </summary>
        public static Mesh CreateBox(GlInterface gl, Vector3 center, Vector3 size, Color color)
        {
            var mesh = CreateBox(size.X, size.Y, size.Z);
            mesh.Color = color;
            mesh.Transform = Matrix4x4.CreateTranslation(center);
            mesh.Initialize(gl);
            return mesh;
        }

        /// <summary>
        /// Create a sphere mesh without OpenGL initialization
        /// </summary>
        public static Mesh CreateSphere(float radius, int segments, int rings)
        {
            var vertices = new List<Vertex>();
            var indices = new List<uint>();

            // Default color
            var color = new Color(0.8f, 0.8f, 0.8f, 1.0f);

            // Generate vertices
            for (int lat = 0; lat <= rings; lat++)
            {
                float theta = lat * MathF.PI / rings; // 0 to PI
                float sinTheta = MathF.Sin(theta);
                float cosTheta = MathF.Cos(theta);

                for (int lon = 0; lon <= segments; lon++)
                {
                    float phi = lon * 2.0f * MathF.PI / segments; // 0 to 2PI
                    float sinPhi = MathF.Sin(phi);
                    float cosPhi = MathF.Cos(phi);

                    // Spherical to Cartesian
                    float x = cosPhi * sinTheta;
                    float y = cosTheta;
                    float z = sinPhi * sinTheta;

                    Vector3 normal = new Vector3(x, y, z);
                    Vector3 position = normal * radius;

                    vertices.Add(new Vertex
                    {
                        Position = position,
                        Normal = normal,
                        TexCoord = new Vector2((float)lon / segments, (float)lat / rings)
                    });
                }
            }

            // Generate indices
            for (int lat = 0; lat < rings; lat++)
            {
                for (int lon = 0; lon < segments; lon++)
                {
                    uint current = (uint)(lat * (segments + 1) + lon);
                    uint next = (uint)(current + segments + 1);

                    // First triangle
                    indices.Add(current);
                    indices.Add(next);
                    indices.Add(current + 1);

                    // Second triangle
                    indices.Add(current + 1);
                    indices.Add(next);
                    indices.Add(next + 1);
                }
            }

            var mesh = new Mesh(vertices, indices);
            mesh.Color = color;
            return mesh;
        }

        /// <summary>
        /// Create a sphere mesh with OpenGL initialization
        /// </summary>
        public static Mesh CreateSphere(GlInterface gl, Vector3 center, float radius, int segments, Color color)
        {
            var mesh = CreateSphere(radius, segments, segments);
            mesh.Color = color;
            mesh.Transform = Matrix4x4.CreateTranslation(center);
            mesh.Initialize(gl);
            return mesh;
        }

        /// <summary>
        /// Create a cylinder mesh without OpenGL initialization
        /// </summary>
        public static Mesh CreateCylinder(float radius, float height, int segments)
        {
            var vertices = new List<Vertex>();
            var indices = new List<uint>();

            // Default color
            var color = new Color(0.8f, 0.8f, 0.8f, 1.0f);

            float halfHeight = height * 0.5f;

            // Center vertices for top and bottom caps
            Vector3 bottomCenter = new Vector3(0, -halfHeight, 0);
            Vector3 topCenter = new Vector3(0, halfHeight, 0);

            // Bottom center (index 0)
            vertices.Add(new Vertex 
            { 
                Position = bottomCenter, 
                Normal = new Vector3(0, -1, 0),
                TexCoord = new Vector2(0.5f, 0.5f)
            });

            // Top center (index 1)
            vertices.Add(new Vertex 
            { 
                Position = topCenter, 
                Normal = new Vector3(0, 1, 0),
                TexCoord = new Vector2(0.5f, 0.5f)
            });

            // Generate ring vertices
            for (int i = 0; i <= segments; i++)
            {
                float angle = i * 2.0f * MathF.PI / segments;
                float x = MathF.Cos(angle) * radius;
                float z = MathF.Sin(angle) * radius;

                Vector3 normal = new Vector3(MathF.Cos(angle), 0, MathF.Sin(angle));

                // Bottom ring vertex (index 2 + i*2)
                vertices.Add(new Vertex
                {
                    Position = bottomCenter + new Vector3(x, 0, z),
                    Normal = normal,
                    TexCoord = new Vector2((float)i / segments, 0)
                });

                // Top ring vertex (index 2 + i*2 + 1)
                vertices.Add(new Vertex
                {
                    Position = topCenter + new Vector3(x, 0, z),
                    Normal = normal,
                    TexCoord = new Vector2((float)i / segments, 1)
                });
            }

            // Generate indices for side faces
            for (int i = 0; i < segments; i++)
            {
                uint bottomCurrent = (uint)(2 + i * 2);
                uint topCurrent = (uint)(2 + i * 2 + 1);
                uint bottomNext = (uint)(2 + (i + 1) * 2);
                uint topNext = (uint)(2 + (i + 1) * 2 + 1);

                // Side quad (2 triangles)
                indices.Add(bottomCurrent);
                indices.Add(bottomNext);
                indices.Add(topCurrent);

                indices.Add(topCurrent);
                indices.Add(bottomNext);
                indices.Add(topNext);

                // Bottom cap triangle
                indices.Add(0); // bottom center
                indices.Add(bottomNext);
                indices.Add(bottomCurrent);

                // Top cap triangle
                indices.Add(1); // top center
                indices.Add(topCurrent);
                indices.Add(topNext);
            }

            var mesh = new Mesh(vertices, indices);
            mesh.Color = color;
            return mesh;
        }

        /// <summary>
        /// Create a cylinder mesh with OpenGL initialization
        /// </summary>
        public static Mesh CreateCylinder(GlInterface gl, Vector3 center, float radius, float height, int segments, Color color)
        {
            var mesh = CreateCylinder(radius, height, segments);
            mesh.Color = color;
            mesh.Transform = Matrix4x4.CreateTranslation(center);
            mesh.Initialize(gl);
            return mesh;
        }
    }
}
