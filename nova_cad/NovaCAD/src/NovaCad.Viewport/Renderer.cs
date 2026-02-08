using System;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Helper renderer for drawing common elements
    /// </summary>
    public unsafe class Renderer : IDisposable
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

            // Upload and draw
            var vertexArray = vertices.ToArray();
            fixed (float* ptr = vertexArray)
            {
                _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _lineVbo);
                _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertexArray.Length * sizeof(float)), ptr, BufferUsageARB.StreamDraw);
            }

            _lineShader.Use();
            // TODO: Set uniforms for line shader

            _gl.BindVertexArray(_lineVao);
            _gl.DrawArrays(PrimitiveType.Lines, 0, (uint)(vertexArray.Length / 3));
            _gl.BindVertexArray(0);
        }

        public void Dispose()
        {
            if (_disposed) return;

            _lineShader?.Dispose();
            _gl.DeleteBuffer(_lineVbo);
            _gl.DeleteVertexArray(_lineVao);

            _disposed = true;
        }
    }
}
