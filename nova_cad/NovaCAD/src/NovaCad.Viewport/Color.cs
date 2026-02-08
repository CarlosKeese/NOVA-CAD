namespace NovaCad.Viewport
{
    /// <summary>
    /// Color with RGBA components for OpenGL rendering
    /// </summary>
    public struct Color
    {
        public float R;
        public float G;
        public float B;
        public float A;

        public Color(float r, float g, float b, float a = 1.0f)
        {
            R = r;
            G = g;
            B = b;
            A = a;
        }

        public static readonly Color White = new Color(1, 1, 1);
        public static readonly Color Black = new Color(0, 0, 0);
        public static readonly Color Red = new Color(1, 0, 0);
        public static readonly Color Green = new Color(0, 1, 0);
        public static readonly Color Blue = new Color(0, 0, 1);
        public static readonly Color Yellow = new Color(1, 1, 0);
        public static readonly Color Cyan = new Color(0, 1, 1);
        public static readonly Color Magenta = new Color(1, 0, 1);
        public static readonly Color Gray = new Color(0.5f, 0.5f, 0.5f);
        public static readonly Color LightGray = new Color(0.8f, 0.8f, 0.8f);
        public static readonly Color DarkGray = new Color(0.3f, 0.3f, 0.3f);
    }
}
