using System;
using System.Runtime.InteropServices;

namespace NovaCad.Kernel
{
    /// <summary>
    /// P/Invoke wrapper for the Nova Kernel 3D C-ABI
    /// </summary>
    public static class NovaKernel
    {
        private const string LibraryName = "nova_ffi";

        #region Types

        /// <summary>
        /// Handle to a Nova kernel object
        /// </summary>
        public readonly record struct NovaHandle(ulong Value)
        {
            public static readonly NovaHandle Null = new(0);
            public bool IsValid => Value != 0;
        }

        /// <summary>
        /// Result codes from Nova operations
        /// </summary>
        public enum NovaResult : int
        {
            Success = 0,
            InvalidHandle = 1,
            InvalidParameter = 2,
            OutOfMemory = 3,
            GeometryError = 4,
            TopologyError = 5,
            NotImplemented = 6,
            UnknownError = 7,
        }

        /// <summary>
        /// 3D point structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaPoint3
        {
            public double X;
            public double Y;
            public double Z;

            public NovaPoint3(double x, double y, double z)
            {
                X = x;
                Y = y;
                Z = z;
            }

            public override string ToString() => $"({X:F6}, {Y:F6}, {Z:F6})";
        }

        /// <summary>
        /// 3D vector structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaVec3
        {
            public double X;
            public double Y;
            public double Z;

            public NovaVec3(double x, double y, double z)
            {
                X = x;
                Y = y;
                Z = z;
            }

            public override string ToString() => $"[{X:F6}, {Y:F6}, {Z:F6}]";
        }

        /// <summary>
        /// 4x4 matrix (row-major)
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaMat4
        {
            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 16)]
            public double[] M;

            public static NovaMat4 Identity => new()
            {
                M = new double[] {
                    1, 0, 0, 0,
                    0, 1, 0, 0,
                    0, 0, 1, 0,
                    0, 0, 0, 1
                }
            };
        }

        /// <summary>
        /// Transform structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaTransform
        {
            public NovaPoint3 Translation;
            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
            public double[] Rotation; // Quaternion (w, x, y, z)

            public static NovaTransform Identity => new()
            {
                Translation = new NovaPoint3(0, 0, 0),
                Rotation = new double[] { 1, 0, 0, 0 }
            };
        }

        /// <summary>
        /// Bounding box structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaBBox3
        {
            public NovaPoint3 Min;
            public NovaPoint3 Max;
        }

        /// <summary>
        /// Mesh vertex structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaMeshVertex
        {
            public NovaPoint3 Position;
            public NovaVec3 Normal;
            public double U;
            public double V;
        }

        /// <summary>
        /// Mesh structure
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct NovaMesh
        {
            public IntPtr Vertices;
            public uint VertexCount;
            public IntPtr Indices;
            public uint IndexCount;
        }

        #endregion

        #region Initialization

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_init();

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_shutdown();

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr nova_version();

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_set_tolerance(double tolerance);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern double nova_get_tolerance();

        #endregion

        #region Primitive Creation

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_make_box(
            double width,
            double height,
            double depth,
            out NovaHandle outHandle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_make_cylinder(
            double radius,
            double height,
            out NovaHandle outHandle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_make_sphere(
            double radius,
            out NovaHandle outHandle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_make_cone(
            double baseRadius,
            double topRadius,
            double height,
            out NovaHandle outHandle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_make_torus(
            double majorRadius,
            double minorRadius,
            out NovaHandle outHandle);

        #endregion

        #region Body Operations

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_body_release(NovaHandle handle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_body_transform(
            NovaHandle handle,
            ref NovaTransform transform);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_body_bounding_box(
            NovaHandle handle,
            out NovaBBox3 outBBox);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_body_copy(
            NovaHandle handle,
            out NovaHandle outHandle);

        #endregion

        #region Boolean Operations

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_boolean_unite(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_boolean_subtract(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_boolean_intersect(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult);

        #endregion

        #region Feature Operations

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_fillet(
            NovaHandle body,
            [MarshalAs(UnmanagedType.LPArray)] NovaHandle[] edges,
            uint edgeCount,
            double radius,
            out NovaHandle outResult);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_chamfer(
            NovaHandle body,
            [MarshalAs(UnmanagedType.LPArray)] NovaHandle[] edges,
            uint edgeCount,
            double distance1,
            double distance2,
            out NovaHandle outResult);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_shell(
            NovaHandle body,
            [MarshalAs(UnmanagedType.LPArray)] NovaHandle[] faces,
            uint faceCount,
            double thickness,
            out NovaHandle outResult);

        #endregion

        #region Tessellation

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_tessellate_body(
            NovaHandle body,
            double chordTolerance,
            double angleTolerance,
            out NovaMesh outMesh);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern NovaResult nova_mesh_free(ref NovaMesh mesh);

        #endregion

        #region File I/O

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        public static extern NovaResult nova_import_step(
            [MarshalAs(UnmanagedType.LPStr)] string filepath,
            out NovaHandle outHandle);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        public static extern NovaResult nova_export_step(
            NovaHandle body,
            [MarshalAs(UnmanagedType.LPStr)] string filepath);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        public static extern NovaResult nova_export_stl(
            NovaHandle body,
            [MarshalAs(UnmanagedType.LPStr)] string filepath);

        #endregion

        #region Error Handling

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr nova_last_error();

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void nova_clear_error();

        #endregion

        #region Helper Methods

        /// <summary>
        /// Get the version string
        /// </summary>
        public static string GetVersion()
        {
            var ptr = nova_version();
            return Marshal.PtrToStringAnsi(ptr) ?? "Unknown";
        }

        /// <summary>
        /// Get the last error message
        /// </summary>
        public static string GetLastError()
        {
            var ptr = nova_last_error();
            return Marshal.PtrToStringAnsi(ptr) ?? "Unknown error";
        }

        /// <summary>
        /// Throw an exception if the result is not success
        /// </summary>
        public static void ThrowOnError(NovaResult result)
        {
            if (result != NovaResult.Success)
            {
                throw new NovaKernelException(result, GetLastError());
            }
        }

        #endregion
    }

    /// <summary>
    /// Exception thrown when a Nova kernel operation fails
    /// </summary>
    public class NovaKernelException : Exception
    {
        public NovaKernel.NovaResult ResultCode { get; }

        public NovaKernelException(NovaKernel.NovaResult resultCode, string message)
            : base($"Nova kernel error ({resultCode}): {message}")
        {
            ResultCode = resultCode;
        }
    }
}
