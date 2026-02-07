using System;
using System.Numerics;
using Silk.NET.OpenGL;
using NovaCad.Core.Models;
using NovaCad.Kernel;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D Steering Wheel overlay for direct manipulation
    /// </summary>
    public class SteeringWheelOverlay : IDisposable
    {
        private GL _gl;
        private Shader _shader;
        private uint _vao;
        private uint _vbo;
        
        // Wheel state
        public Vector3 Position { get; set; }
        public Vector3 PrimaryAxis { get; set; }
        public Vector3 SecondaryAxis { get; set; }
        public Vector3 TertiaryAxis { get; set; }
        
        public float Radius { get; set; } = 20.0f;
        public bool Visible { get; set; } = true;
        public bool Active { get; set; } = true;
        
        // Interaction
        public SteeringWheelMode Mode { get; set; } = SteeringWheelMode.Idle;
        public SteeringWheelHandle? SelectedHandle { get; set; }
        public SteeringWheelHandle? HoveredHandle { get; set; }
        
        // Events
        public event EventHandler<SteeringWheelDragEventArgs>? DragStarted;
        public event EventHandler<SteeringWheelDragEventArgs>? Dragging;
        public event EventHandler<SteeringWheelDragEventArgs>? DragEnded;
        public event EventHandler<SteeringWheelActionEventArgs>? ActionTriggered;

        public SteeringWheelOverlay(GL gl)
        {
            _gl = gl;
            Initialize();
        }

        private void Initialize()
        {
            // Create shader for wheel rendering
            _shader = new Shader(_gl, GetVertexShader(), GetFragmentShader());
            
            // Create geometry buffers
            CreateWheelGeometry();
            
            // Default orientation
            PrimaryAxis = Vector3.UnitZ;
            SecondaryAxis = Vector3.UnitX;
            TertiaryAxis = Vector3.UnitY;
        }

        private void CreateWheelGeometry()
        {
            _vao = _gl.GenVertexArray();
            _vbo = _gl.GenBuffer();
            
            _gl.BindVertexArray(_vao);
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            
            // Position attribute
            _gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)0);
            _gl.EnableVertexAttribArray(0);
            
            // Color attribute
            _gl.VertexAttribPointer(1, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)(3 * sizeof(float)));
            _gl.EnableVertexAttribArray(1);
            
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Render the steering wheel
        /// </summary>
        public void Render(Camera3D camera)
        {
            if (!Visible || !Active) return;

            _shader.Use();
            
            // Set uniforms
            _shader.SetMatrix4("uView", camera.GetViewMatrix());
            _shader.SetMatrix4("uProjection", camera.GetProjectionMatrix());
            _shader.SetVector3("uWheelCenter", Position);
            _shader.SetFloat("uRadius", Radius);
            _shader.SetVector3("uPrimaryAxis", PrimaryAxis);
            _shader.SetVector3("uSecondaryAxis", SecondaryAxis);
            _shader.SetVector3("uTertiaryAxis", TertiaryAxis);

            // Draw torus for each axis
            DrawAxisRing(PrimaryAxis, new Color(0.0f, 0.0f, 1.0f, 1.0f), camera);    // Z - Blue
            DrawAxisRing(SecondaryAxis, new Color(1.0f, 0.0f, 0.0f, 1.0f), camera);   // X - Red
            DrawAxisRing(TertiaryAxis, new Color(0.0f, 1.0f, 0.0f, 1.0f), camera);    // Y - Green
            
            // Draw handles
            DrawHandle(Position + PrimaryAxis * Radius, new Color(0.0f, 0.0f, 1.0f, 1.0f), 
                HoveredHandle == SteeringWheelHandle.PrimaryAxis);
            DrawHandle(Position + SecondaryAxis * Radius, new Color(1.0f, 0.0f, 0.0f, 1.0f),
                HoveredHandle == SteeringWheelHandle.SecondaryAxis);
            DrawHandle(Position + TertiaryAxis * Radius, new Color(0.0f, 1.0f, 0.0f, 1.0f),
                HoveredHandle == SteeringWheelHandle.TertiaryAxis);
            
            // Draw plane handle
            Vector3 planePos = Position + (PrimaryAxis + SecondaryAxis) * Radius * 0.7f;
            DrawPlaneHandle(planePos, new Color(1.0f, 1.0f, 0.0f, 0.8f),
                HoveredHandle == SteeringWheelHandle.Plane);
        }

        private void DrawAxisRing(Vector3 axis, Color color, Camera3D camera)
        {
            // Generate ring vertices
            const int segments = 64;
            var vertices = new float[segments * 6];
            
            Vector3 perp1 = GetPerpendicular(axis);
            Vector3 perp2 = Vector3.Cross(axis, perp1);
            
            for (int i = 0; i < segments; i++)
            {
                float angle = (float)i / segments * MathF.PI * 2;
                float x = MathF.Cos(angle);
                float y = MathF.Sin(angle);
                
                Vector3 pos = Position + (perp1 * x + perp2 * y) * Radius;
                
                int idx = i * 6;
                vertices[idx] = pos.X;
                vertices[idx + 1] = pos.Y;
                vertices[idx + 2] = pos.Z;
                vertices[idx + 3] = color.R;
                vertices[idx + 4] = color.G;
                vertices[idx + 5] = color.B;
            }
            
            // Upload and draw
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), vertices, BufferUsageARB.StreamDraw);
            
            _gl.BindVertexArray(_vao);
            _gl.DrawArrays(PrimitiveType.LineLoop, 0, (uint)segments);
            _gl.BindVertexArray(0);
        }

        private void DrawHandle(Vector3 position, Color color, bool highlighted)
        {
            float size = highlighted ? 3.0f : 2.0f;
            
            // Draw a small sphere/cube at handle position
            var vertices = CreateCubeVertices(position, size, color);
            
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), vertices, BufferUsageARB.StreamDraw);
            
            _gl.BindVertexArray(_vao);
            _gl.DrawArrays(PrimitiveType.Triangles, 0, (uint)(vertices.Length / 6));
            _gl.BindVertexArray(0);
        }

        private void DrawPlaneHandle(Vector3 position, Color color, bool highlighted)
        {
            float size = highlighted ? 4.0f : 3.0f;
            
            // Draw a small square for plane manipulation
            var vertices = CreateSquareVertices(position, size, color);
            
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), vertices, BufferUsageARB.StreamDraw);
            
            _gl.BindVertexArray(_vao);
            _gl.DrawArrays(PrimitiveType.TriangleFan, 0, 4);
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Handle mouse down on wheel
        /// </summary>
        public bool OnMouseDown(Vector3 worldPos, Vector2 screenPos)
        {
            if (!Visible || !Active) return false;

            var handle = PickHandle(worldPos);
            if (handle.HasValue)
            {
                SelectedHandle = handle;
                Mode = GetModeFromHandle(handle.Value);
                
                DragStarted?.Invoke(this, new SteeringWheelDragEventArgs
                {
                    Handle = handle.Value,
                    StartPosition = worldPos,
                    CurrentPosition = worldPos,
                    Mode = Mode
                });
                
                return true;
            }
            
            return false;
        }

        /// <summary>
        /// Handle mouse move on wheel
        /// </summary>
        public bool OnMouseMove(Vector3 worldPos, Vector2 screenPos)
        {
            if (!Visible || !Active) return false;

            if (SelectedHandle.HasValue && Mode != SteeringWheelMode.Idle)
            {
                Dragging?.Invoke(this, new SteeringWheelDragEventArgs
                {
                    Handle = SelectedHandle.Value,
                    StartPosition = Position,
                    CurrentPosition = worldPos,
                    Mode = Mode,
                    Delta = worldPos - Position
                });
                
                return true;
            }
            else
            {
                // Update hover state
                HoveredHandle = PickHandle(worldPos);
                return HoveredHandle.HasValue;
            }
        }

        /// <summary>
        /// Handle mouse up on wheel
        /// </summary>
        public bool OnMouseUp(Vector3 worldPos)
        {
            if (SelectedHandle.HasValue)
            {
                DragEnded?.Invoke(this, new SteeringWheelDragEventArgs
                {
                    Handle = SelectedHandle.Value,
                    StartPosition = Position,
                    CurrentPosition = worldPos,
                    Mode = Mode
                });
                
                SelectedHandle = null;
                Mode = SteeringWheelMode.Idle;
                return true;
            }
            
            return false;
        }

        /// <summary>
        /// Pick a handle at world position
        /// </summary>
        private SteeringWheelHandle? PickHandle(Vector3 worldPos)
        {
            float tolerance = Radius * 0.15f;
            
            // Check primary axis handle
            if (Vector3.Distance(worldPos, Position + PrimaryAxis * Radius) < tolerance)
                return SteeringWheelHandle.PrimaryAxis;
            
            // Check secondary axis handle
            if (Vector3.Distance(worldPos, Position + SecondaryAxis * Radius) < tolerance)
                return SteeringWheelHandle.SecondaryAxis;
            
            // Check tertiary axis handle
            if (Vector3.Distance(worldPos, Position + TertiaryAxis * Radius) < tolerance)
                return SteeringWheelHandle.TertiaryAxis;
            
            // Check plane handle
            Vector3 planePos = Position + (PrimaryAxis + SecondaryAxis) * Radius * 0.7f;
            if (Vector3.Distance(worldPos, planePos) < tolerance)
                return SteeringWheelHandle.Plane;
            
            return null;
        }

        private SteeringWheelMode GetModeFromHandle(SteeringWheelHandle handle)
        {
            return handle switch
            {
                SteeringWheelHandle.PrimaryAxis => SteeringWheelMode.MovePrimary,
                SteeringWheelHandle.SecondaryAxis => SteeringWheelMode.MoveSecondary,
                SteeringWheelHandle.TertiaryAxis => SteeringWheelMode.MoveTertiary,
                SteeringWheelHandle.Plane => SteeringWheelMode.MovePlane,
                _ => SteeringWheelMode.Idle
            };
        }

        /// <summary>
        /// Relocate wheel to new position
        /// </summary>
        public void Relocate(Vector3 newPosition)
        {
            Position = newPosition;
        }

        /// <summary>
        /// Orient wheel to align with normal
        /// </summary>
        public void Orient(Vector3 normal)
        {
            PrimaryAxis = Vector3.Normalize(normal);
            
            // Generate perpendicular axes
            if (Math.Abs(Vector3.Dot(PrimaryAxis, Vector3.UnitZ)) < 0.9f)
            {
                SecondaryAxis = Vector3.Normalize(Vector3.Cross(PrimaryAxis, Vector3.UnitZ));
            }
            else
            {
                SecondaryAxis = Vector3.Normalize(Vector3.Cross(PrimaryAxis, Vector3.UnitY));
            }
            
            TertiaryAxis = Vector3.Cross(PrimaryAxis, SecondaryAxis);
        }

        /// <summary>
        /// Snap to nearest major axis
        /// </summary>
        public void SnapToMajorAxis()
        {
            Vector3[] majorAxes = new[]
            {
                Vector3.UnitX, -Vector3.UnitX,
                Vector3.UnitY, -Vector3.UnitY,
                Vector3.UnitZ, -Vector3.UnitZ
            };
            
            float maxDot = -1;
            Vector3 closest = Vector3.UnitZ;
            
            foreach (var axis in majorAxes)
            {
                float dot = Math.Abs(Vector3.Dot(PrimaryAxis, axis));
                if (dot > maxDot)
                {
                    maxDot = dot;
                    closest = axis;
                }
            }
            
            Orient(closest);
        }

        private Vector3 GetPerpendicular(Vector3 v)
        {
            if (Math.Abs(v.X) < Math.Abs(v.Y) && Math.Abs(v.X) < Math.Abs(v.Z))
                return Vector3.Normalize(Vector3.Cross(v, Vector3.UnitX));
            else if (Math.Abs(v.Y) < Math.Abs(v.Z))
                return Vector3.Normalize(Vector3.Cross(v, Vector3.UnitY));
            else
                return Vector3.Normalize(Vector3.Cross(v, Vector3.UnitZ));
        }

        private float[] CreateCubeVertices(Vector3 center, float size, Color color)
        {
            float h = size / 2;
            // Simplified cube vertices
            return new float[]
            {
                // Front face
                center.X - h, center.Y - h, center.Z + h, color.R, color.G, color.B,
                center.X + h, center.Y - h, center.Z + h, color.R, color.G, color.B,
                center.X + h, center.Y + h, center.Z + h, color.R, color.G, color.B,
                center.X + h, center.Y + h, center.Z + h, color.R, color.G, color.B,
                center.X - h, center.Y + h, center.Z + h, color.R, color.G, color.B,
                center.X - h, center.Y - h, center.Z + h, color.R, color.G, color.B,
            };
        }

        private float[] CreateSquareVertices(Vector3 center, float size, Color color)
        {
            float h = size / 2;
            return new float[]
            {
                center.X - h, center.Y, center.Z - h, color.R, color.G, color.B,
                center.X + h, center.Y, center.Z - h, color.R, color.G, color.B,
                center.X + h, center.Y, center.Z + h, color.R, color.G, color.B,
                center.X - h, center.Y, center.Z + h, color.R, color.G, color.B,
            };
        }

        private string GetVertexShader()
        {
            return @"
#version 330 core
layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aColor;
uniform mat4 uView;
uniform mat4 uProjection;
out vec3 vColor;
void main()
{
    gl_Position = uProjection * uView * vec4(aPosition, 1.0);
    vColor = aColor;
}
";
        }

        private string GetFragmentShader()
        {
            return @"
#version 330 core
in vec3 vColor;
out vec4 FragColor;
void main()
{
    FragColor = vec4(vColor, 1.0);
}
";
        }

        public void Dispose()
        {
            _shader?.Dispose();
            if (_vao != 0) _gl.DeleteVertexArray(_vao);
            if (_vbo != 0) _gl.DeleteBuffer(_vbo);
            GC.SuppressFinalize(this);
        }
    }

    public enum SteeringWheelHandle
    {
        PrimaryAxis,
        SecondaryAxis,
        TertiaryAxis,
        Plane,
        RotationRing
    }

    public enum SteeringWheelMode
    {
        Idle,
        MovePrimary,
        MoveSecondary,
        MoveTertiary,
        MovePlane,
        RotatePrimary,
        RotateSecondary,
        RotateTertiary
    }

    public class SteeringWheelDragEventArgs : EventArgs
    {
        public SteeringWheelHandle Handle { get; set; }
        public Vector3 StartPosition { get; set; }
        public Vector3 CurrentPosition { get; set; }
        public Vector3 Delta { get; set; }
        public SteeringWheelMode Mode { get; set; }
    }

    public class SteeringWheelActionEventArgs : EventArgs
    {
        public string Action { get; set; }
        public Vector3? Position { get; set; }
    }
}
