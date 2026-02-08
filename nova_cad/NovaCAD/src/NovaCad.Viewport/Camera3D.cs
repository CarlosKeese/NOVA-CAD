using System;
using System.Numerics;
using Silk.NET.Maths;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D Camera with orbit, pan, and zoom controls
    /// </summary>
    public class Camera3D
    {
        private Vector3 _position;
        private Vector3 _target;
        private Vector3 _up;
        private Vector3 _right;
        private Vector3 _front;
        
        private float _yaw;
        private float _pitch;
        private float _distance;
        
        private int _width;
        private int _height;
        private float _aspectRatio;
        
        // Camera parameters
        public float FieldOfView { get; set; } = 45.0f;
        public float NearPlane { get; set; } = 0.1f;
        public float FarPlane { get; set; } = 1000.0f;
        public float ZoomSpeed { get; set; } = 0.1f;
        public float PanSpeed { get; set; } = 0.01f;
        public float RotationSpeed { get; set; } = 0.005f;
        
        // Interaction state
        private bool _isPanning;
        private bool _isRotating;
        private int _lastMouseX;
        private int _lastMouseY;

        public Vector3 Position { get => _position; set { _position = value; UpdateVectors(); } }
        public Vector3 Target { get => _target; set { _target = value; UpdateVectors(); } }
        public Vector3 Up => _up;
        public Vector3 Front => _front;
        public Vector3 Right => _right;
        public float Aspect { get => _aspectRatio; set { _aspectRatio = value; } }

        public Camera3D(int width, int height)
        {
            _width = width;
            _height = height;
            _aspectRatio = (float)width / height;
            
            // Default position
            _distance = 50.0f;
            _yaw = -45.0f;
            _pitch = -30.0f;
            _target = Vector3.Zero;
            _up = Vector3.UnitY;
            
            UpdateVectors();
        }

        public Camera3D(Vector3 position, Vector3 target, Vector3 up, float fov, float aspect, float near, float far)
        {
            _position = position;
            _target = target;
            _up = up;
            FieldOfView = fov;
            _aspectRatio = aspect;
            NearPlane = near;
            FarPlane = far;
            
            // Calculate distance and angles from position/target
            Vector3 dir = position - target;
            _distance = dir.Length();
            
            Vector3 dirNorm = Vector3.Normalize(dir);
            _yaw = MathF.Atan2(dirNorm.X, dirNorm.Z) * 180.0f / MathF.PI;
            _pitch = MathF.Asin(-dirNorm.Y) * 180.0f / MathF.PI;
            
            UpdateVectors();
        }

        /// <summary>
        /// Resize the camera viewport
        /// </summary>
        public void Resize(int width, int height)
        {
            _width = width;
            _height = height;
            _aspectRatio = (float)width / height;
        }

        /// <summary>
        /// Get the view matrix
        /// </summary>
        public Matrix4x4 GetViewMatrix()
        {
            return Matrix4x4.CreateLookAt(_position, _target, _up);
        }

        /// <summary>
        /// Get the projection matrix
        /// </summary>
        public Matrix4x4 GetProjectionMatrix()
        {
            return Matrix4x4.CreatePerspectiveFieldOfView(
                FieldOfView * MathF.PI / 180.0f,
                _aspectRatio,
                NearPlane,
                FarPlane);
        }

        /// <summary>
        /// Start pan operation
        /// </summary>
        public void StartPan(int x, int y)
        {
            _isPanning = true;
            _lastMouseX = x;
            _lastMouseY = y;
        }

        /// <summary>
        /// Pan the camera
        /// </summary>
        public void Pan(int x, int y)
        {
            if (!_isPanning) return;

            float deltaX = (x - _lastMouseX) * PanSpeed * _distance * 0.1f;
            float deltaY = (y - _lastMouseY) * PanSpeed * _distance * 0.1f;

            // Pan in view plane
            Vector3 panX = _right * deltaX;
            Vector3 panY = _up * deltaY;
            
            _target -= panX;
            _target += panY;

            _lastMouseX = x;
            _lastMouseY = y;

            UpdateVectors();
        }

        /// <summary>
        /// Start rotate operation
        /// </summary>
        public void StartRotate(int x, int y)
        {
            _isRotating = true;
            _lastMouseX = x;
            _lastMouseY = y;
        }

        /// <summary>
        /// Orbit rotate around target
        /// </summary>
        public void Rotate(int x, int y)
        {
            if (!_isRotating) return;

            float deltaX = (x - _lastMouseX) * RotationSpeed * 180.0f / MathF.PI;
            float deltaY = (y - _lastMouseY) * RotationSpeed * 180.0f / MathF.PI;

            _yaw += deltaX;
            _pitch -= deltaY;

            // Clamp pitch to avoid gimbal lock
            _pitch = Math.Clamp(_pitch, -89.0f, 89.0f);

            _lastMouseX = x;
            _lastMouseY = y;

            UpdateVectors();
        }

        /// <summary>
        /// Zoom camera
        /// </summary>
        public void Zoom(float delta)
        {
            _distance -= delta * ZoomSpeed * _distance;
            _distance = Math.Clamp(_distance, 0.1f, 1000.0f);
            
            UpdateVectors();
        }

        /// <summary>
        /// End current interaction
        /// </summary>
        public void EndInteraction()
        {
            _isPanning = false;
            _isRotating = false;
        }

        /// <summary>
        /// Fit camera to bounding box
        /// </summary>
        public void FitToBoundingBox(BoundingBox bbox)
        {
            Vector3 center = bbox.Center;
            float radius = bbox.Radius;
            
            _target = center;
            _distance = radius * 2.5f;
            
            UpdateVectors();
        }

        /// <summary>
        /// Set camera to standard view
        /// </summary>
        public void SetStandardView(StandardView view)
        {
            switch (view)
            {
                case StandardView.Front:
                    _yaw = 0.0f;
                    _pitch = 0.0f;
                    break;
                case StandardView.Back:
                    _yaw = 180.0f;
                    _pitch = 0.0f;
                    break;
                case StandardView.Top:
                    _yaw = 0.0f;
                    _pitch = -90.0f;
                    break;
                case StandardView.Bottom:
                    _yaw = 0.0f;
                    _pitch = 90.0f;
                    break;
                case StandardView.Left:
                    _yaw = -90.0f;
                    _pitch = 0.0f;
                    break;
                case StandardView.Right:
                    _yaw = 90.0f;
                    _pitch = 0.0f;
                    break;
                case StandardView.Isometric:
                    _yaw = -45.0f;
                    _pitch = -35.264f;
                    break;
                case StandardView.Dimetric:
                    _yaw = -45.0f;
                    _pitch = -30.0f;
                    break;
                case StandardView.Trimetric:
                    _yaw = -60.0f;
                    _pitch = -30.0f;
                    break;
            }
            
            UpdateVectors();
        }

        /// <summary>
        /// Get a ray from camera through screen point
        /// </summary>
        public Ray GetRay(float ndcX, float ndcY)
        {
            // Convert NDC to clip space
            Vector4 clipSpace = new Vector4(ndcX, ndcY, -1.0f, 1.0f);
            
            // To view space
            Matrix4x4.Invert(GetProjectionMatrix(), out Matrix4x4 invProjection);
            Vector4 viewSpace = Vector4.Transform(clipSpace, invProjection);
            viewSpace.Z = -1.0f;
            viewSpace.W = 0.0f;
            
            // To world space
            Matrix4x4.Invert(GetViewMatrix(), out Matrix4x4 invView);
            Vector4 worldSpace = Vector4.Transform(viewSpace, invView);
            Vector3 rayDir = new Vector3(worldSpace.X, worldSpace.Y, worldSpace.Z);
            rayDir = Vector3.Normalize(rayDir);
            
            return new Ray(_position, rayDir);
        }

        /// <summary>
        /// Look at a specific point
        /// </summary>
        public void LookAt(Vector3 target)
        {
            _target = target;
            UpdateVectors();
        }

        /// <summary>
        /// Set camera position directly
        /// </summary>
        public void SetPosition(Vector3 position)
        {
            _position = position;
            _distance = Vector3.Distance(_position, _target);
            
            // Recalculate yaw and pitch
            Vector3 dir = Vector3.Normalize(_target - _position);
            _yaw = MathF.Atan2(dir.X, dir.Z) * 180.0f / MathF.PI;
            _pitch = MathF.Asin(dir.Y) * 180.0f / MathF.PI;
            
            UpdateVectors();
        }

        private void UpdateVectors()
        {
            // Calculate new front vector from yaw and pitch
            float yawRad = _yaw * MathF.PI / 180.0f;
            float pitchRad = _pitch * MathF.PI / 180.0f;

            _front.X = MathF.Cos(pitchRad) * MathF.Sin(yawRad);
            _front.Y = MathF.Sin(pitchRad);
            _front.Z = MathF.Cos(pitchRad) * MathF.Cos(yawRad);
            _front = Vector3.Normalize(_front);

            // Recalculate right and up vectors
            _right = Vector3.Normalize(Vector3.Cross(_front, Vector3.UnitY));
            _up = Vector3.Normalize(Vector3.Cross(_right, _front));

            // Update position based on target and distance
            _position = _target - _front * _distance;
        }
    }

    /// <summary>
    /// Ray for picking
    /// </summary>
    public struct Ray
    {
        public Vector3 Origin;
        public Vector3 Direction;

        public Ray(Vector3 origin, Vector3 direction)
        {
            Origin = origin;
            Direction = direction;
        }

        public Vector3 GetPoint(float distance)
        {
            return Origin + Direction * distance;
        }
    }

}
