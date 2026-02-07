using System;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D transform gizmo for move, rotate, and scale operations
    /// </summary>
    public class TransformGizmo : IDisposable
    {
        private GL _gl;
        private Shader _shader;
        private uint _vao;
        private uint _vbo;
        
        // Gizmo state
        public Vector3 Position { get; set; }
        public Quaternion Rotation { get; set; } = Quaternion.Identity;
        public Vector3 Scale { get; set; } = Vector3.One;
        
        public GizmoMode Mode { get; set; } = GizmoMode.Translate;
        public GizmoSpace Space { get; set; } = GizmoSpace.World;
        
        public float Size { get; set; } = 15.0f;
        public bool Visible { get; set; } = true;
        public bool Active { get; set; } = true;
        
        // Axis colors
        public static readonly Color XAxisColor = new Color(1.0f, 0.0f, 0.0f, 1.0f);
        public static readonly Color YAxisColor = new Color(0.0f, 1.0f, 0.0f, 1.0f);
        public static readonly Color ZAxisColor = new Color(0.0f, 0.0f, 1.0f, 1.0f);
        public static readonly Color CenterColor = new Color(1.0f, 1.0f, 1.0f, 1.0f);
        
        // Interaction
        public GizmoAxis? SelectedAxis { get; set; }
        public GizmoAxis? HoveredAxis { get; set; }
        
        // Events
        public event EventHandler<GizmoDragEventArgs>? DragStarted;
        public event EventHandler<GizmoDragEventArgs>? Dragging;
        public event EventHandler<GizmoDragEventArgs>? DragEnded;

        public TransformGizmo(GL gl)
        {
            _gl = gl;
            Initialize();
        }

        private void Initialize()
        {
            _shader = new Shader(_gl, GetVertexShader(), GetFragmentShader());
            
            _vao = _gl.GenVertexArray();
            _vbo = _gl.GenBuffer();
            
            _gl.BindVertexArray(_vao);
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            
            _gl.VertexAttribPointer(0, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)0);
            _gl.EnableVertexAttribArray(0);
            
            _gl.VertexAttribPointer(1, 3, VertexAttribPointerType.Float, false, 6 * sizeof(float), (void*)(3 * sizeof(float)));
            _gl.EnableVertexAttribArray(1);
            
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Render the gizmo
        /// </summary>
        public void Render(Camera3D camera)
        {
            if (!Visible || !Active) return;

            _shader.Use();
            _shader.SetMatrix4("uView", camera.GetViewMatrix());
            _shader.SetMatrix4("uProjection", camera.GetProjectionMatrix());
            _shader.SetVector3("uGizmoPosition", Position);
            _shader.SetFloat("uGizmoSize", Size);

            // Calculate screen-space size
            float distance = Vector3.Distance(camera.Position, Position);
            float screenSize = Size * (distance * 0.01f);
            
            switch (Mode)
            {
                case GizmoMode.Translate:
                    DrawTranslateGizmo(camera, screenSize);
                    break;
                case GizmoMode.Rotate:
                    DrawRotateGizmo(camera, screenSize);
                    break;
                case GizmoMode.Scale:
                    DrawScaleGizmo(camera, screenSize);
                    break;
            }
        }

        private void DrawTranslateGizmo(Camera3D camera, float size)
        {
            // Get axes based on space
            Vector3 xAxis = GetAxisDirection(Vector3.UnitX);
            Vector3 yAxis = GetAxisDirection(Vector3.UnitY);
            Vector3 zAxis = GetAxisDirection(Vector3.UnitZ);
            
            // Draw axis lines
            DrawAxisLine(Vector3.Zero, xAxis * size, XAxisColor, HoveredAxis == GizmoAxis.X);
            DrawAxisLine(Vector3.Zero, yAxis * size, YAxisColor, HoveredAxis == GizmoAxis.Y);
            DrawAxisLine(Vector3.Zero, zAxis * size, ZAxisColor, HoveredAxis == GizmoAxis.Z);
            
            // Draw arrow heads
            DrawArrowHead(Position + xAxis * size, xAxis, XAxisColor, HoveredAxis == GizmoAxis.X);
            DrawArrowHead(Position + yAxis * size, yAxis, YAxisColor, HoveredAxis == GizmoAxis.Y);
            DrawArrowHead(Position + zAxis * size, zAxis, ZAxisColor, HoveredAxis == GizmoAxis.Z);
            
            // Draw center cube
            DrawCenterCube(CenterColor, HoveredAxis == GizmoAxis.Center);
            
            // Draw plane handles
            float planeSize = size * 0.3f;
            DrawPlaneHandle(xAxis, yAxis, planeSize, new Color(1.0f, 1.0f, 0.0f, 0.5f), HoveredAxis == GizmoAxis.XY);
            DrawPlaneHandle(yAxis, zAxis, planeSize, new Color(0.0f, 1.0f, 1.0f, 0.5f), HoveredAxis == GizmoAxis.YZ);
            DrawPlaneHandle(zAxis, xAxis, planeSize, new Color(1.0f, 0.0f, 1.0f, 0.5f), HoveredAxis == GizmoAxis.ZX);
        }

        private void DrawRotateGizmo(Camera3D camera, float size)
        {
            Vector3 xAxis = GetAxisDirection(Vector3.UnitX);
            Vector3 yAxis = GetAxisDirection(Vector3.UnitY);
            Vector3 zAxis = GetAxisDirection(Vector3.UnitZ);
            
            // Draw rotation rings
            DrawRotationRing(xAxis, size, XAxisColor, HoveredAxis == GizmoAxis.X);
            DrawRotationRing(yAxis, size, YAxisColor, HoveredAxis == GizmoAxis.Y);
            DrawRotationRing(zAxis, size, ZAxisColor, HoveredAxis == GizmoAxis.Z);
            
            // Draw view-aligned ring for free rotation
            Vector3 viewDir = Vector3.Normalize(camera.Position - Position);
            DrawRotationRing(viewDir, size * 1.1f, CenterColor, HoveredAxis == GizmoAxis.View);
        }

        private void DrawScaleGizmo(Camera3D camera, float size)
        {
            Vector3 xAxis = GetAxisDirection(Vector3.UnitX);
            Vector3 yAxis = GetAxisDirection(Vector3.UnitY);
            Vector3 zAxis = GetAxisDirection(Vector3.UnitZ);
            
            // Draw axis lines
            DrawAxisLine(Vector3.Zero, xAxis * size, XAxisColor, HoveredAxis == GizmoAxis.X);
            DrawAxisLine(Vector3.Zero, yAxis * size, YAxisColor, HoveredAxis == GizmoAxis.Y);
            DrawAxisLine(Vector3.Zero, zAxis * size, ZAxisColor, HoveredAxis == GizmoAxis.Z);
            
            // Draw scale boxes at ends
            DrawScaleBox(Position + xAxis * size, XAxisColor, HoveredAxis == GizmoAxis.X);
            DrawScaleBox(Position + yAxis * size, YAxisColor, HoveredAxis == GizmoAxis.Y);
            DrawScaleBox(Position + zAxis * size, ZAxisColor, HoveredAxis == GizmoAxis.Z);
            
            // Draw center box
            DrawCenterCube(CenterColor, HoveredAxis == GizmoAxis.Center);
            
            // Draw uniform scale box at end of each axis (connected)
            DrawUniformScaleHandle(size);
        }

        private void DrawAxisLine(Vector3 start, Vector3 end, Color color, bool highlighted)
        {
            float thickness = highlighted ? 3.0f : 2.0f;
            
            var vertices = new[]
            {
                Position.X + start.X, Position.Y + start.Y, Position.Z + start.Z, color.R, color.G, color.B,
                Position.X + end.X, Position.Y + end.Y, Position.Z + end.Z, color.R, color.G, color.B,
            };
            
            UploadAndDraw(vertices, PrimitiveType.Lines);
        }

        private void DrawArrowHead(Vector3 position, Vector3 direction, Color color, bool highlighted)
        {
            float size = highlighted ? 4.0f : 3.0f;
            // Simplified arrow head as small cone
            DrawHandle(position, size, color, highlighted);
        }

        private void DrawRotationRing(Vector3 axis, float radius, Color color, bool highlighted)
        {
            int segments = highlighted ? 64 : 32;
            var vertices = new float[segments * 6];
            
            Vector3 perp1 = GetPerpendicular(axis);
            Vector3 perp2 = Vector3.Cross(axis, perp1);
            
            for (int i = 0; i < segments; i++)
            {
                float angle = (float)i / segments * MathF.PI * 2;
                float x = MathF.Cos(angle);
                float y = MathF.Sin(angle);
                
                Vector3 pos = Position + (perp1 * x + perp2 * y) * radius;
                
                int idx = i * 6;
                vertices[idx] = pos.X;
                vertices[idx + 1] = pos.Y;
                vertices[idx + 2] = pos.Z;
                vertices[idx + 3] = color.R;
                vertices[idx + 4] = color.G;
                vertices[idx + 5] = color.B;
            }
            
            UploadAndDraw(vertices, PrimitiveType.LineLoop);
        }

        private void DrawPlaneHandle(Vector3 axis1, Vector3 axis2, float size, Color color, bool highlighted)
        {
            if (highlighted) color = new Color(color.R * 1.5f, color.G * 1.5f, color.B * 1.5f, color.A);
            
            var vertices = new[]
            {
                Position.X, Position.Y, Position.Z, color.R, color.G, color.B,
                Position.X + axis1.X * size, Position.Y + axis1.Y * size, Position.Z + axis1.Z * size, color.R, color.G, color.B,
                Position.X + (axis1.X + axis2.X) * size, Position.Y + (axis1.Y + axis2.Y) * size, Position.Z + (axis1.Z + axis2.Z) * size, color.R, color.G, color.B,
                Position.X + axis2.X * size, Position.Y + axis2.Y * size, Position.Z + axis2.Z * size, color.R, color.G, color.B,
            };
            
            UploadAndDraw(vertices, PrimitiveType.TriangleFan);
        }

        private void DrawCenterCube(Color color, bool highlighted)
        {
            float size = highlighted ? 3.0f : 2.0f;
            DrawHandle(Position, size, color, highlighted);
        }

        private void DrawScaleBox(Vector3 position, Color color, bool highlighted)
        {
            float size = highlighted ? 4.0f : 3.0f;
            DrawHandle(position, size, color, highlighted);
        }

        private void DrawHandle(Vector3 position, float size, Color color, bool highlighted)
        {
            // Simplified as small cube
            float h = size / 2;
            var vertices = CreateCubeVertices(position, h, color);
            UploadAndDraw(vertices, PrimitiveType.Triangles);
        }

        private void DrawUniformScaleHandle(float size)
        {
            // Draw small cube at corner
            Vector3 corner = new Vector3(size, size, size);
            DrawHandle(Position + corner, 2.0f, CenterColor, HoveredAxis == GizmoAxis.Uniform);
        }

        private void UploadAndDraw(float[] vertices, PrimitiveType mode)
        {
            _gl.BindBuffer(BufferTargetARB.ArrayBuffer, _vbo);
            _gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * sizeof(float)), vertices, BufferUsageARB.StreamDraw);
            
            _gl.BindVertexArray(_vao);
            _gl.DrawArrays(mode, 0, (uint)(vertices.Length / 6));
            _gl.BindVertexArray(0);
        }

        /// <summary>
        /// Handle mouse down on gizmo
        /// </summary>
        public bool OnMouseDown(Vector3 worldPos, Vector2 screenPos, Camera3D camera)
        {
            if (!Visible || !Active) return false;

            var axis = PickAxis(worldPos, camera);
            if (axis.HasValue)
            {
                SelectedAxis = axis;
                
                DragStarted?.Invoke(this, new GizmoDragEventArgs
                {
                    Axis = axis.Value,
                    Mode = Mode,
                    StartPosition = Position,
                    CurrentPosition = Position,
                    GizmoSpace = Space
                });
                
                return true;
            }
            
            return false;
        }

        /// <summary>
        /// Handle mouse move on gizmo
        /// </summary>
        public bool OnMouseMove(Vector3 worldPos, Vector2 screenPos, Camera3D camera)
        {
            if (!Visible || !Active) return false;

            if (SelectedAxis.HasValue)
            {
                // Calculate delta based on axis and mode
                Vector3 delta = CalculateDelta(worldPos, SelectedAxis.Value);
                
                Dragging?.Invoke(this, new GizmoDragEventArgs
                {
                    Axis = SelectedAxis.Value,
                    Mode = Mode,
                    StartPosition = Position,
                    CurrentPosition = worldPos,
                    Delta = delta,
                    GizmoSpace = Space
                });
                
                return true;
            }
            else
            {
                HoveredAxis = PickAxis(worldPos, camera);
                return HoveredAxis.HasValue;
            }
        }

        /// <summary>
        /// Handle mouse up on gizmo
        /// </summary>
        public bool OnMouseUp(Vector3 worldPos)
        {
            if (SelectedAxis.HasValue)
            {
                DragEnded?.Invoke(this, new GizmoDragEventArgs
                {
                    Axis = SelectedAxis.Value,
                    Mode = Mode,
                    StartPosition = Position,
                    CurrentPosition = worldPos,
                    GizmoSpace = Space
                });
                
                SelectedAxis = null;
                return true;
            }
            
            return false;
        }

        private GizmoAxis? PickAxis(Vector3 worldPos, Camera3D camera)
        {
            float tolerance = Size * 0.1f;
            
            // Check each axis
            Vector3 xAxis = GetAxisDirection(Vector3.UnitX);
            Vector3 yAxis = GetAxisDirection(Vector3.UnitY);
            Vector3 zAxis = GetAxisDirection(Vector3.UnitZ);
            
            // Project point onto axis lines
            float distX = DistancePointToLine(worldPos, Position, Position + xAxis * Size);
            float distY = DistancePointToLine(worldPos, Position, Position + yAxis * Size);
            float distZ = DistancePointToLine(worldPos, Position, Position + zAxis * Size);
            
            if (distX < tolerance) return GizmoAxis.X;
            if (distY < tolerance) return GizmoAxis.Y;
            if (distZ < tolerance) return GizmoAxis.Z;
            
            // Check center
            if (Vector3.Distance(worldPos, Position) < tolerance)
                return GizmoAxis.Center;
            
            return null;
        }

        private Vector3 CalculateDelta(Vector3 worldPos, GizmoAxis axis)
        {
            Vector3 axisDir = axis switch
            {
                GizmoAxis.X => GetAxisDirection(Vector3.UnitX),
                GizmoAxis.Y => GetAxisDirection(Vector3.UnitY),
                GizmoAxis.Z => GetAxisDirection(Vector3.UnitZ),
                _ => Vector3.Zero
            };
            
            Vector3 offset = worldPos - Position;
            return axisDir * Vector3.Dot(offset, axisDir);
        }

        private Vector3 GetAxisDirection(Vector3 localAxis)
        {
            return Space == GizmoSpace.Local
                ? Vector3.Transform(localAxis, Rotation)
                : localAxis;
        }

        private float DistancePointToLine(Vector3 point, Vector3 lineStart, Vector3 lineEnd)
        {
            Vector3 lineDir = lineEnd - lineStart;
            float lineLength = lineDir.Length();
            if (lineLength < 0.0001f) return Vector3.Distance(point, lineStart);
            
            lineDir /= lineLength;
            Vector3 toPoint = point - lineStart;
            float projection = Vector3.Dot(toPoint, lineDir);
            
            if (projection < 0) return Vector3.Distance(point, lineStart);
            if (projection > lineLength) return Vector3.Distance(point, lineEnd);
            
            Vector3 closest = lineStart + lineDir * projection;
            return Vector3.Distance(point, closest);
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

        private float[] CreateCubeVertices(Vector3 center, float halfSize, Color color)
        {
            // Simplified - just return corners
            return new float[]
            {
                center.X - halfSize, center.Y - halfSize, center.Z - halfSize, color.R, color.G, color.B,
                center.X + halfSize, center.Y - halfSize, center.Z - halfSize, color.R, color.G, color.B,
                center.X + halfSize, center.Y + halfSize, center.Z - halfSize, color.R, color.G, color.B,
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
uniform vec3 uGizmoPosition;
uniform float uGizmoSize;
out vec3 vColor;
void main()
{
    vec3 worldPos = uGizmoPosition + aPosition * uGizmoSize * 0.01;
    gl_Position = uProjection * uView * vec4(worldPos, 1.0);
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

    public enum GizmoMode
    {
        Translate,
        Rotate,
        Scale
    }

    public enum GizmoSpace
    {
        World,
        Local
    }

    public enum GizmoAxis
    {
        X,
        Y,
        Z,
        XY,
        YZ,
        ZX,
        Center,
        Uniform,
        View
    }

    public class GizmoDragEventArgs : EventArgs
    {
        public GizmoAxis Axis { get; set; }
        public GizmoMode Mode { get; set; }
        public GizmoSpace GizmoSpace { get; set; }
        public Vector3 StartPosition { get; set; }
        public Vector3 CurrentPosition { get; set; }
        public Vector3 Delta { get; set; }
    }
}
