using System.Numerics;

namespace NovaCad.Viewport
{
    /// <summary>
    /// 3D bounding box
    /// </summary>
    public struct BoundingBox
    {
        public Vector3 Min;
        public Vector3 Max;

        public Vector3 Center => (Min + Max) * 0.5f;
        public Vector3 Size => Max - Min;
        
        public float Radius
        {
            get
            {
                float dx = Max.X - Min.X;
                float dy = Max.Y - Min.Y;
                float dz = Max.Z - Min.Z;
                return float.Sqrt(dx * dx + dy * dy + dz * dz) * 0.5f;
            }
        }

        public BoundingBox(Vector3 min, Vector3 max)
        {
            Min = min;
            Max = max;
        }

        public void Expand(Vector3 point)
        {
            Min = Vector3.Min(Min, point);
            Max = Vector3.Max(Max, point);
        }

        public void Expand(BoundingBox other)
        {
            Min = Vector3.Min(Min, other.Min);
            Max = Vector3.Max(Max, other.Max);
        }
    }
}
