using System;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// OpenGL Shader program
    /// </summary>
    public class Shader : IDisposable
    {
        private GL _gl;
        private uint _program;
        private bool _disposed;

        public Shader(GL gl, string vertexSource, string fragmentSource)
        {
            _gl = gl ?? throw new ArgumentNullException(nameof(gl));
            
            uint vertex = CompileShader(ShaderType.VertexShader, vertexSource);
            uint fragment = CompileShader(ShaderType.FragmentShader, fragmentSource);
            
            _program = LinkProgram(vertex, fragment);
            
            // Clean up individual shaders
            gl.DeleteShader(vertex);
            gl.DeleteShader(fragment);
        }

        public void Use()
        {
            _gl.UseProgram(_program);
        }

        public void SetBool(string name, bool value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform1(location, value ? 1 : 0);
        }

        public void SetInt(string name, int value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform1(location, value);
        }

        public void SetFloat(string name, float value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform1(location, value);
        }

        public void SetVector2(string name, Vector2 value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform2(location, value.X, value.Y);
        }

        public void SetVector3(string name, Vector3 value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform3(location, value.X, value.Y, value.Z);
        }

        public void SetVector4(string name, Vector4 value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
                _gl.Uniform4(location, value.X, value.Y, value.Z, value.W);
        }

        public unsafe void SetMatrix4(string name, Matrix4x4 value)
        {
            int location = _gl.GetUniformLocation(_program, name);
            if (location >= 0)
            {
                float* ptr = stackalloc float[16];
                ptr[0] = value.M11; ptr[1] = value.M12; ptr[2] = value.M13; ptr[3] = value.M14;
                ptr[4] = value.M21; ptr[5] = value.M22; ptr[6] = value.M23; ptr[7] = value.M24;
                ptr[8] = value.M31; ptr[9] = value.M32; ptr[10] = value.M33; ptr[11] = value.M34;
                ptr[12] = value.M41; ptr[13] = value.M42; ptr[14] = value.M43; ptr[15] = value.M44;
                _gl.UniformMatrix4(location, 1, false, ptr);
            }
        }

        private uint CompileShader(ShaderType type, string source)
        {
            uint shader = _gl.CreateShader(type);
            _gl.ShaderSource(shader, source);
            _gl.CompileShader(shader);

            // Check for compilation errors
            _gl.GetShader(shader, ShaderParameterName.CompileStatus, out int status);
            if (status != 1)
            {
                string infoLog = _gl.GetShaderInfoLog(shader);
                throw new Exception($"Shader compilation failed: {infoLog}");
            }

            return shader;
        }

        private uint LinkProgram(uint vertex, uint fragment)
        {
            uint program = _gl.CreateProgram();
            _gl.AttachShader(program, vertex);
            _gl.AttachShader(program, fragment);
            _gl.LinkProgram(program);

            // Check for linking errors
            _gl.GetProgram(program, ProgramPropertyARB.LinkStatus, out int status);
            if (status != 1)
            {
                string infoLog = _gl.GetProgramInfoLog(program);
                throw new Exception($"Program linking failed: {infoLog}");
            }

            return program;
        }

        public void Dispose()
        {
            if (_disposed) return;
            
            _gl.DeleteProgram(_program);
            
            _disposed = true;
            GC.SuppressFinalize(this);
        }
    }
}
