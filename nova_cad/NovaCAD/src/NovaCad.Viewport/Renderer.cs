using System;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Helper renderer for drawing common elements
    /// </summary>
    public class Renderer : IDisposable
    {
        private GL _gl;
        private Shader _lineShader;
        private uint _lineVao;
        private uint _lineVbo;
        private bool _disposed;

        public Renderer(GL gl)
        {
            _gl = gl;
            InitializeLineRenderer();
        }

        private void InitializeLineRenderer()
        {
            // Simple line shader
            string vertexSource = @"
#version 330 core
layout (location = 0) in vec3 aPosition;
uniform mat4 uMVP;
void main()
{
    gl_Position = uMVP * vec4(aPosition, 1.0);
}
";

            string fragmentSource = @"
#version 330 core
uniform vec4 uColor;
out vec4 FragColor;
void main()
{
    FragColor = uColor;
}
";

            _lineShader = new Shader(_gl, vertexSource, fragmentSource);

            // Create VAO/VBO for lines
            _lineVao = _gl.GenVertexArray();
            _lineVbo = _gl.GenBuffer();

            _gl.BindVertexArray(_lineVao);
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _lineVbo);
            _gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, 3 * sizeof(float), (void*)0);
            _gl.EnableVertexAttribArray(0);
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Render a grid on the XZ plane
        /// </summary>
        public void RenderGrid(Camera3D camera, float size, float spacing)
        {
            int lines = (int)(size / spacing);
            var vertices = new System.Collections.Generic.List<float>();

            // Grid lines parallel to X axis
            for (int i = -lines; i <= lines; i++)
            {
                float z = i * spacing;
                vertices.Add(-size);
                vertices.Add(0);
                vertices.Add(z);
                vertices.Add(size);
                vertices.Add(0);
                vertices.Add(z);
            }

            // Grid lines parallel to Z axis
            for (int i = -lines; i <= lines; i++)
            {
                float x = i * spacing;
                vertices.Add(x);
                vertices.Add(0);
                vertices.Add(-size);
                vertices.Add(x);
                vertices.Add(0);
                vertices.Add(size);
            }

            DrawLines(vertices.ToArray(), new Color(0.3f, 0.3f, 0.3f, 1.0f), camera);
        }

        /// <summary>
        /// Render XYZ axes
        /// </summary>
        public void RenderAxes(Camera3D camera, float length)
        {
            // X axis (red)
            DrawLine(Vector3.Zero, new Vector3(length, 0, 0), Color.Red, camera);
            
            // Y axis (green)
            DrawLine(Vector3.Zero, new Vector3(0, length, 0), Color.Green, camera);
            
            // Z axis (blue)
            DrawLine(Vector3.Zero, new Vector3(0, 0, length), Color.Blue, camera);
        }

        /// <summary>
        /// Render selection highlight
        /// </summary>
        public void RenderHighlight(uint entityId, Color color)
        {
            // TODO: Implement selection highlight rendering
        }

        /// <summary>
        /// Draw a single line
        /// </summary>
        public void DrawLine(Vector3 start, Vector3 end, Color color, Camera3D camera)
        {
            float[] vertices =
            {
                start.X, start.Y, start.Z,
                end.X, end.Y, end.Z
            };

            DrawLines(vertices, color, camera, PrimitiveType.Lines);
        }

        /// <summary>
        /// Draw multiple lines
        /// </summary>
        public void DrawLines(float[] vertices, Color color, Camera3D camera, PrimitiveType mode = PrimitiveType.Lines)
        {
            if (vertices.Length == 0) return;

            _lineShader.Use();

            // Calculate MVP matrix
            var mvp = camera.GetViewMatrix() * camera.GetProjectionMatrix();
            _lineShader.SetMatrix4("uMVP", mvp);
            _lineShader.SetVector4("uColor", color.ToVector4());

            // Upload vertices
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _lineVbo);
            _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), vertices, BufferUsageARB.StreamDraw);

            // Draw
            _gl.BindVertexArray(_lineVao);
            _gl.DrawArrays(mode, 0, (uint)(vertices.Length / 3));
            _gl.BindVertexArray(0);
        }

        public void Dispose()
        {
            if (_disposed) return;

            _lineShader?.Dispose();
            _gl.DeleteVertexArray(_lineVao);
            _gl.DeleteBuffer(_lineVbo);

            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }
}
