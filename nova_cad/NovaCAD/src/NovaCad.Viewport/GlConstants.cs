namespace NovaCad.Viewport
{
    /// <summary>
    /// OpenGL constants for use with Avalonia's GlInterface
    /// </summary>
    public static class GlConstants
    {
        public const int GL_FALSE = 0;
        public const int GL_TRUE = 1;

        public const int GL_DEPTH_TEST = 0x0B71;
        public const int GL_BLEND = 0x0BE2;
        public const int GL_SRC_ALPHA = 0x0302;
        public const int GL_ONE_MINUS_SRC_ALPHA = 0x0303;

        public const int GL_ARRAY_BUFFER = 0x8892;
        public const int GL_ELEMENT_ARRAY_BUFFER = 0x8893;
        public const int GL_STATIC_DRAW = 0x88E4;

        public const int GL_FLOAT = 0x1406;
        public const int GL_UNSIGNED_INT = 0x1405;

        public const int GL_TRIANGLES = 0x0004;
        public const int GL_LINES = 0x0001;

        public const int GL_COLOR_BUFFER_BIT = 0x00004000;
        public const int GL_DEPTH_BUFFER_BIT = 0x00000100;

        public const int GL_VERTEX_SHADER = 0x8B31;
        public const int GL_FRAGMENT_SHADER = 0x8B30;
    }
}
