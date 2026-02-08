using System;
using System.Numerics;
using Avalonia.OpenGL;
using static NovaCad.Viewport.GlConstants;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Extension methods for Avalonia's GlInterface
    /// </summary>
    public static unsafe class GlExtensions
    {
        // Get function pointers for OpenGL functions not directly exposed by GlInterface
        private static delegate* unmanaged<int, int, void> _blendFunc;
        private static delegate* unmanaged<float, float, float, float, void> _clearColor;
        private static delegate* unmanaged<int, void> _clear;
        private static delegate* unmanaged<int, void> _enable;
        private static delegate* unmanaged<int, int*, void> _genBuffers;
        private static delegate* unmanaged<int, int*, void> _genVertexArrays;
        private static delegate* unmanaged<int, int*, void> _deleteBuffers;
        private static delegate* unmanaged<int, int*, void> _deleteVertexArrays;
        private static delegate* unmanaged<int, int, void> _bindBuffer;
        private static delegate* unmanaged<int, void> _bindVertexArray;
        private static delegate* unmanaged<int, int, void*, int, void> _bufferData;
        private static delegate* unmanaged<uint, int, int, int, int, void*, void> _vertexAttribPointer;
        private static delegate* unmanaged<uint, void> _enableVertexAttribArray;
        private static delegate* unmanaged<int, int, int, void*, void> _drawElements;
        private static delegate* unmanaged<int, int, int, void> _drawArrays;
        private static delegate* unmanaged<int, int, int, int, void> _viewport;
        private static delegate* unmanaged<int, int> _createShader;
        private static delegate* unmanaged<int, void> _deleteShader;
        private static delegate* unmanaged<int, int, string[], int*, void> _shaderSource;
        private static delegate* unmanaged<int, void> _compileShader;
        private static delegate* unmanaged<int> _createProgram;
        private static delegate* unmanaged<int, void> _deleteProgram;
        private static delegate* unmanaged<int, int, void> _attachShader;
        private static delegate* unmanaged<int, void> _linkProgram;
        private static delegate* unmanaged<int, void> _useProgram;
        private static delegate* unmanaged<int, byte*, int> _getUniformLocation;
        private static delegate* unmanaged<int, int, int, float*, void> _uniformMatrix4fv;

        private static bool _initialized = false;

        public static void Initialize(GlInterface gl)
        {
            if (_initialized) return;

            _blendFunc = (delegate* unmanaged<int, int, void>)gl.GetProcAddress("glBlendFunc");
            _clearColor = (delegate* unmanaged<float, float, float, float, void>)gl.GetProcAddress("glClearColor");
            _clear = (delegate* unmanaged<int, void>)gl.GetProcAddress("glClear");
            _enable = (delegate* unmanaged<int, void>)gl.GetProcAddress("glEnable");
            _genBuffers = (delegate* unmanaged<int, int*, void>)gl.GetProcAddress("glGenBuffers");
            _genVertexArrays = (delegate* unmanaged<int, int*, void>)gl.GetProcAddress("glGenVertexArrays");
            _deleteBuffers = (delegate* unmanaged<int, int*, void>)gl.GetProcAddress("glDeleteBuffers");
            _deleteVertexArrays = (delegate* unmanaged<int, int*, void>)gl.GetProcAddress("glDeleteVertexArrays");
            _bindBuffer = (delegate* unmanaged<int, int, void>)gl.GetProcAddress("glBindBuffer");
            _bindVertexArray = (delegate* unmanaged<int, void>)gl.GetProcAddress("glBindVertexArray");
            _bufferData = (delegate* unmanaged<int, int, void*, int, void>)gl.GetProcAddress("glBufferData");
            _vertexAttribPointer = (delegate* unmanaged<uint, int, int, int, int, void*, void>)gl.GetProcAddress("glVertexAttribPointer");
            _enableVertexAttribArray = (delegate* unmanaged<uint, void>)gl.GetProcAddress("glEnableVertexAttribArray");
            _drawElements = (delegate* unmanaged<int, int, int, void*, void>)gl.GetProcAddress("glDrawElements");
            _drawArrays = (delegate* unmanaged<int, int, int, void>)gl.GetProcAddress("glDrawArrays");
            _viewport = (delegate* unmanaged<int, int, int, int, void>)gl.GetProcAddress("glViewport");
            _createShader = (delegate* unmanaged<int, int>)gl.GetProcAddress("glCreateShader");
            _deleteShader = (delegate* unmanaged<int, void>)gl.GetProcAddress("glDeleteShader");
            _shaderSource = (delegate* unmanaged<int, int, string[], int*, void>)gl.GetProcAddress("glShaderSource");
            _compileShader = (delegate* unmanaged<int, void>)gl.GetProcAddress("glCompileShader");
            _createProgram = (delegate* unmanaged<int>)gl.GetProcAddress("glCreateProgram");
            _deleteProgram = (delegate* unmanaged<int, void>)gl.GetProcAddress("glDeleteProgram");
            _attachShader = (delegate* unmanaged<int, int, void>)gl.GetProcAddress("glAttachShader");
            _linkProgram = (delegate* unmanaged<int, void>)gl.GetProcAddress("glLinkProgram");
            _useProgram = (delegate* unmanaged<int, void>)gl.GetProcAddress("glUseProgram");
            _getUniformLocation = (delegate* unmanaged<int, byte*, int>)gl.GetProcAddress("glGetUniformLocation");
            _uniformMatrix4fv = (delegate* unmanaged<int, int, int, float*, void>)gl.GetProcAddress("glUniformMatrix4fv");

            _initialized = true;
        }

        public static void BlendFunc(this GlInterface gl, int sfactor, int dfactor)
        {
            if (_blendFunc != null) _blendFunc(sfactor, dfactor);
        }

        public static void ClearColor(this GlInterface gl, float r, float g, float b, float a)
        {
            if (_clearColor != null) _clearColor(r, g, b, a);
        }

        public static void Clear(this GlInterface gl, int mask)
        {
            if (_clear != null) _clear(mask);
        }

        public static void Enable(this GlInterface gl, int cap)
        {
            if (_enable != null) _enable(cap);
        }

        public static int GenBuffer(this GlInterface gl)
        {
            int id = 0;
            if (_genBuffers != null) 
            {
                _genBuffers(1, &id);
            }
            return id;
        }

        public static int GenVertexArray(this GlInterface gl)
        {
            int id = 0;
            if (_genVertexArrays != null) 
            {
                _genVertexArrays(1, &id);
            }
            return id;
        }

        public static void DeleteBuffer(this GlInterface gl, int buffer)
        {
            if (_deleteBuffers != null) 
            {
                _deleteBuffers(1, &buffer);
            }
        }

        public static void DeleteVertexArray(this GlInterface gl, int array)
        {
            if (_deleteVertexArrays != null) 
            {
                _deleteVertexArrays(1, &array);
            }
        }

        public static void BindBuffer(this GlInterface gl, int target, int buffer)
        {
            if (_bindBuffer != null) _bindBuffer(target, buffer);
        }

        public static void BindVertexArray(this GlInterface gl, int array)
        {
            if (_bindVertexArray != null) _bindVertexArray(array);
        }

        public static void BufferData(this GlInterface gl, int target, int size, IntPtr data, int usage)
        {
            if (_bufferData != null) _bufferData(target, size, data.ToPointer(), usage);
        }

        public static void VertexAttribPointer(this GlInterface gl, uint index, int size, int type, int normalized, int stride, IntPtr pointer)
        {
            if (_vertexAttribPointer != null) _vertexAttribPointer(index, size, type, normalized, stride, pointer.ToPointer());
        }

        public static void EnableVertexAttribArray(this GlInterface gl, uint index)
        {
            if (_enableVertexAttribArray != null) _enableVertexAttribArray(index);
        }

        public static void DrawElements(this GlInterface gl, int mode, int count, int type, IntPtr indices)
        {
            if (_drawElements != null) _drawElements(mode, count, type, indices.ToPointer());
        }

        public static void DrawArrays(this GlInterface gl, int mode, int first, int count)
        {
            if (_drawArrays != null) _drawArrays(mode, first, count);
        }

        public static void Viewport(this GlInterface gl, int x, int y, int width, int height)
        {
            if (_viewport != null) _viewport(x, y, width, height);
        }

        public static int CreateShader(this GlInterface gl, int type)
        {
            if (_createShader != null) return _createShader(type);
            return 0;
        }

        public static void DeleteShader(this GlInterface gl, int shader)
        {
            if (_deleteShader != null) _deleteShader(shader);
        }

        public static void ShaderSource(this GlInterface gl, int shader, string source)
        {
            if (_shaderSource != null)
            {
                var sources = new[] { source };
                var lengths = new[] { source.Length };
                fixed (int* pLengths = lengths)
                {
                    _shaderSource(shader, 1, sources, pLengths);
                }
            }
        }

        public static void CompileShader(this GlInterface gl, int shader)
        {
            if (_compileShader != null) _compileShader(shader);
        }

        public static int CreateProgram(this GlInterface gl)
        {
            if (_createProgram != null) return _createProgram();
            return 0;
        }

        public static void DeleteProgram(this GlInterface gl, int program)
        {
            if (_deleteProgram != null) _deleteProgram(program);
        }

        public static void AttachShader(this GlInterface gl, int program, int shader)
        {
            if (_attachShader != null) _attachShader(program, shader);
        }

        public static void LinkProgram(this GlInterface gl, int program)
        {
            if (_linkProgram != null) _linkProgram(program);
        }

        public static void UseProgram(this GlInterface gl, int program)
        {
            if (_useProgram != null) _useProgram(program);
        }

        public static int GetUniformLocationString(this GlInterface gl, int program, string name)
        {
            if (_getUniformLocation != null)
            {
                var bytes = System.Text.Encoding.UTF8.GetBytes(name + '\0');
                fixed (byte* pBytes = bytes)
                {
                    return _getUniformLocation(program, pBytes);
                }
            }
            return -1;
        }

        public static void UniformMatrix4fv(this GlInterface gl, int location, int count, int transpose, float* value)
        {
            if (_uniformMatrix4fv != null) _uniformMatrix4fv(location, count, transpose, value);
        }
    }
}
