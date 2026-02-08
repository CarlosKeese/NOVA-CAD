using System;
using System.Runtime.InteropServices;

namespace NovaCad.Kernel
{
    /// <summary>
    /// P/Invoke wrapper for the Nova Kernel 3D C-ABI
    /// Falls back to stub implementation when native DLL is not available.
    /// </summary>
    public static class NovaKernel
    {
        private const string LibraryName = "nova_ffi";
        private static bool _useStubs = false;
        private static bool _initialized = false;

        static NovaKernel()
        {
            // Try to detect if native DLL is available
            try
            {
                // Attempt a simple call to check if DLL exists
                var result = NativeMethods.nova_init();
                NativeMethods.nova_shutdown();
                _useStubs = false;
                Console.WriteLine("[NovaKernel] Native DLL detected and working");
            }
            catch (DllNotFoundException)
            {
                _useStubs = true;
                Console.WriteLine("[NovaKernel] Native DLL not found, using stub implementation");
            }
            catch (BadImageFormatException)
            {
                _useStubs = true;
                Console.WriteLine("[NovaKernel] Native DLL has wrong architecture, using stub implementation");
            }
            catch (Exception ex)
            {
                _useStubs = true;
                Console.WriteLine($"[NovaKernel] Error loading native DLL: {ex.Message}, using stub implementation");
            }
        }

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

        public static NovaResult nova_init()
        {
            if (_initialized) return NovaResult.Success;
            _initialized = true;
            return _useStubs ? NovaKernelStub.nova_init() : NativeMethods.nova_init();
        }

        public static NovaResult nova_shutdown()
        {
            _initialized = false;
            return _useStubs ? NovaKernelStub.nova_shutdown() : NativeMethods.nova_shutdown();
        }

        public static IntPtr nova_version()
        {
            return _useStubs ? NovaKernelStub.nova_version() : NativeMethods.nova_version();
        }

        public static NovaResult nova_set_tolerance(double tolerance)
        {
            return _useStubs ? NovaKernelStub.nova_set_tolerance(tolerance) : NativeMethods.nova_set_tolerance(tolerance);
        }

        public static double nova_get_tolerance()
        {
            return _useStubs ? NovaKernelStub.nova_get_tolerance() : NativeMethods.nova_get_tolerance();
        }

        #endregion

        #region Primitive Creation

        public static NovaResult nova_make_box(
            double width,
            double height,
            double depth,
            out NovaHandle outHandle)
        {
            return _useStubs 
                ? NovaKernelStub.nova_make_box(width, height, depth, out outHandle)
                : NativeMethods.nova_make_box(width, height, depth, out outHandle);
        }

        public static NovaResult nova_make_cylinder(
            double radius,
            double height,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_make_cylinder(radius, height, out outHandle)
                : NativeMethods.nova_make_cylinder(radius, height, out outHandle);
        }

        public static NovaResult nova_make_sphere(
            double radius,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_make_sphere(radius, out outHandle)
                : NativeMethods.nova_make_sphere(radius, out outHandle);
        }

        public static NovaResult nova_make_cone(
            double baseRadius,
            double topRadius,
            double height,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_make_cone(baseRadius, topRadius, height, out outHandle)
                : NativeMethods.nova_make_cone(baseRadius, topRadius, height, out outHandle);
        }

        public static NovaResult nova_make_torus(
            double majorRadius,
            double minorRadius,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_make_torus(majorRadius, minorRadius, out outHandle)
                : NativeMethods.nova_make_torus(majorRadius, minorRadius, out outHandle);
        }

        #endregion

        #region Body Operations

        public static NovaResult nova_body_release(NovaHandle handle)
        {
            return _useStubs
                ? NovaKernelStub.nova_body_release(handle)
                : NativeMethods.nova_body_release(handle);
        }

        public static NovaResult nova_body_transform(
            NovaHandle handle,
            ref NovaTransform transform)
        {
            return _useStubs
                ? NovaKernelStub.nova_body_transform(handle, ref transform)
                : NativeMethods.nova_body_transform(handle, ref transform);
        }

        public static NovaResult nova_body_bounding_box(
            NovaHandle handle,
            out NovaBBox3 outBBox)
        {
            return _useStubs
                ? NovaKernelStub.nova_body_bounding_box(handle, out outBBox)
                : NativeMethods.nova_body_bounding_box(handle, out outBBox);
        }

        public static NovaResult nova_body_copy(
            NovaHandle handle,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_body_copy(handle, out outHandle)
                : NativeMethods.nova_body_copy(handle, out outHandle);
        }

        #endregion

        #region Boolean Operations

        public static NovaResult nova_boolean_unite(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult)
        {
            return _useStubs
                ? NovaKernelStub.nova_boolean_unite(bodyA, bodyB, out outResult)
                : NativeMethods.nova_boolean_unite(bodyA, bodyB, out outResult);
        }

        public static NovaResult nova_boolean_subtract(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult)
        {
            return _useStubs
                ? NovaKernelStub.nova_boolean_subtract(bodyA, bodyB, out outResult)
                : NativeMethods.nova_boolean_subtract(bodyA, bodyB, out outResult);
        }

        public static NovaResult nova_boolean_intersect(
            NovaHandle bodyA,
            NovaHandle bodyB,
            out NovaHandle outResult)
        {
            return _useStubs
                ? NovaKernelStub.nova_boolean_intersect(bodyA, bodyB, out outResult)
                : NativeMethods.nova_boolean_intersect(bodyA, bodyB, out outResult);
        }

        #endregion

        #region Feature Operations

        public static NovaResult nova_fillet(
            NovaHandle body,
            NovaHandle[] edges,
            uint edgeCount,
            double radius,
            out NovaHandle outResult)
        {
            return _useStubs
                ? NovaKernelStub.nova_fillet(body, edges, edgeCount, radius, out outResult)
                : NativeMethods.nova_fillet(body, edges, edgeCount, radius, out outResult);
        }

        public static NovaResult nova_chamfer(
            NovaHandle body,
            NovaHandle[] edges,
            uint edgeCount,
            double distance1,
            double distance2,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_chamfer(body, edges, edgeCount, distance1, distance2, out outHandle)
                : NativeMethods.nova_chamfer(body, edges, edgeCount, distance1, distance2, out outHandle);
        }

        public static NovaResult nova_shell(
            NovaHandle body,
            NovaHandle[] faces,
            uint faceCount,
            double thickness,
            out NovaHandle outResult)
        {
            return _useStubs
                ? NovaKernelStub.nova_shell(body, faces, faceCount, thickness, out outResult)
                : NativeMethods.nova_shell(body, faces, faceCount, thickness, out outResult);
        }

        #endregion

        #region Tessellation

        public static NovaResult nova_tessellate_body(
            NovaHandle body,
            double chordTolerance,
            double angleTolerance,
            out NovaMesh outMesh)
        {
            return _useStubs
                ? NovaKernelStub.nova_tessellate_body(body, chordTolerance, angleTolerance, out outMesh)
                : NativeMethods.nova_tessellate_body(body, chordTolerance, angleTolerance, out outMesh);
        }

        public static NovaResult nova_mesh_free(ref NovaMesh mesh)
        {
            return _useStubs
                ? NovaKernelStub.nova_mesh_free(ref mesh)
                : NativeMethods.nova_mesh_free(ref mesh);
        }

        #endregion

        #region File I/O

        public static NovaResult nova_import_step(
            string filepath,
            out NovaHandle outHandle)
        {
            return _useStubs
                ? NovaKernelStub.nova_import_step(filepath, out outHandle)
                : NativeMethods.nova_import_step(filepath, out outHandle);
        }

        public static NovaResult nova_export_step(
            NovaHandle body,
            string filepath)
        {
            return _useStubs
                ? NovaKernelStub.nova_export_step(body, filepath)
                : NativeMethods.nova_export_step(body, filepath);
        }

        public static NovaResult nova_export_stl(
            NovaHandle body,
            string filepath)
        {
            return _useStubs
                ? NovaKernelStub.nova_export_stl(body, filepath)
                : NativeMethods.nova_export_stl(body, filepath);
        }

        #endregion

        #region Error Handling

        public static IntPtr nova_last_error()
        {
            return _useStubs
                ? NovaKernelStub.nova_last_error()
                : NativeMethods.nova_last_error();
        }

        public static void nova_clear_error()
        {
            if (_useStubs)
                NovaKernelStub.nova_clear_error();
            else
                NativeMethods.nova_clear_error();
        }

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

        /// <summary>
        /// Returns true if using stub implementation (no native DLL)
        /// </summary>
        public static bool IsUsingStubs => _useStubs;

        #endregion

        #region Native Methods (Private)

        private static class NativeMethods
        {
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
                out NovaHandle outHandle);

            [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
            public static extern NovaResult nova_shell(
                NovaHandle body,
                [MarshalAs(UnmanagedType.LPArray)] NovaHandle[] faces,
                uint faceCount,
                double thickness,
                out NovaHandle outResult);

            [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
            public static extern NovaResult nova_tessellate_body(
                NovaHandle body,
                double chordTolerance,
                double angleTolerance,
                out NovaMesh outMesh);

            [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
            public static extern NovaResult nova_mesh_free(ref NovaMesh mesh);

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

            [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr nova_last_error();

            [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
            public static extern void nova_clear_error();
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
