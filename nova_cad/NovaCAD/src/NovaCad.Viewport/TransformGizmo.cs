using System;
using System.Numerics;
using Silk.NET.OpenGL;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D transform gizmo for move, rotate, and scale operations
    /// </summary>
    public unsafe class TransformGizmo : IDisposable
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
            // Create shader
            _shader = new Shader(_gl,
                @"#version 330 core
                layout(location = 0) in vec3 aPos;
                layout(location = 1) in vec3 aColor;
                uniform mat4 mvp;
                out vec3 vColor;
                void main() {
                    gl_Position = mvp * vec4(aPos, 1.0);
                    vColor = aColor;
                }",
                @"#version 330 core
                in vec3 vColor;
                out vec4 FragColor;
                void main() {
                    FragColor = vec4(vColor, 1.0);
                }");

            // Create VAO/VBO
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
        /// Render the gizmo
        /// </summary>
        public void Render(Matrix4x4 viewProjMatrix)
        {
            if (!Visible) return;

            _shader.Use();
            _shader.SetMatrix4("mvp", viewProjMatrix);

            // TODO: Draw gizmo axes based on mode
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
                return true;
            }

            return false;
        }

        /// <summary>
        /// Handle mouse move
        /// </summary>
        public bool OnMouseMove(Vector3 worldPos, Vector2 screenPos, Camera3D camera)
        {
            if (!Visible || !Active) return false;

            if (SelectedAxis.HasValue)
            {
                // TODO: Calculate delta and invoke Dragging event
                return true;
            }
            else
            {
                // Update hover state
                HoveredAxis = PickAxis(worldPos, camera);
                return HoveredAxis.HasValue;
            }
        }

        /// <summary>
        /// Handle mouse up
        /// </summary>
        public void OnMouseUp()
        {
            SelectedAxis = null;
        }

        /// <summary>
        /// Try to pick an axis
        /// </summary>
        private GizmoAxis? PickAxis(Vector3 worldPos, Camera3D camera)
        {
            // TODO: Implement axis picking
            return null;
        }

        public void Dispose()
        {
            _shader?.Dispose();
            _gl.DeleteBuffer(_vbo);
            _gl.DeleteVertexArray(_vao);
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
        Uniform
    }

    public class GizmoDragEventArgs : EventArgs
    {
        public GizmoAxis Axis { get; set; }
        public Vector3 Delta { get; set; }
        public Vector3 WorldPosition { get; set; }
    }
}
