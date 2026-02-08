using System;
using System.Runtime.InteropServices;

namespace NovaCad.Kernel
{
    /// <summary>
    /// Stub implementation of Nova Kernel for when the native DLL is not available.
    /// This allows the UI to function without the Rust kernel.
    /// </summary>
    public static class NovaKernelStub
    {
        private static bool _initialized = false;
        private static ulong _nextHandle = 1;
        private static readonly System.Collections.Generic.Dictionary<ulong, StubBody> _bodies = new();

        private class StubBody
        {
            public ulong Handle { get; set; }
            public string Type { get; set; } = "";
            public double Width { get; set; }
            public double Height { get; set; }
            public double Depth { get; set; }
            public double Radius { get; set; }
        }

        #region Initialization

        public static NovaKernel.NovaResult nova_init()
        {
            if (!_initialized)
            {
                _initialized = true;
                Console.WriteLine("[NovaKernelStub] Initialized (stub mode)");
            }
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_shutdown()
        {
            _initialized = false;
            _bodies.Clear();
            Console.WriteLine("[NovaKernelStub] Shutdown (stub mode)");
            return NovaKernel.NovaResult.Success;
        }

        public static IntPtr nova_version()
        {
            // Return a static version string
            return Marshal.StringToHGlobalAnsi("0.1.0-stub");
        }

        public static NovaKernel.NovaResult nova_set_tolerance(double tolerance)
        {
            return NovaKernel.NovaResult.Success;
        }

        public static double nova_get_tolerance()
        {
            return 0.001;
        }

        #endregion

        #region Primitive Creation

        public static NovaKernel.NovaResult nova_make_box(
            double width,
            double height,
            double depth,
            out NovaKernel.NovaHandle outHandle)
        {
            var handle = _nextHandle++;
            _bodies[handle] = new StubBody
            {
                Handle = handle,
                Type = "Box",
                Width = width,
                Height = height,
                Depth = depth
            };
            outHandle = new NovaKernel.NovaHandle(handle);
            Console.WriteLine($"[NovaKernelStub] Created Box: {width}x{height}x{depth}, handle={handle}");
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_make_cylinder(
            double radius,
            double height,
            out NovaKernel.NovaHandle outHandle)
        {
            var handle = _nextHandle++;
            _bodies[handle] = new StubBody
            {
                Handle = handle,
                Type = "Cylinder",
                Radius = radius,
                Height = height
            };
            outHandle = new NovaKernel.NovaHandle(handle);
            Console.WriteLine($"[NovaKernelStub] Created Cylinder: r={radius}, h={height}, handle={handle}");
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_make_sphere(
            double radius,
            out NovaKernel.NovaHandle outHandle)
        {
            var handle = _nextHandle++;
            _bodies[handle] = new StubBody
            {
                Handle = handle,
                Type = "Sphere",
                Radius = radius
            };
            outHandle = new NovaKernel.NovaHandle(handle);
            Console.WriteLine($"[NovaKernelStub] Created Sphere: r={radius}, handle={handle}");
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_make_cone(
            double baseRadius,
            double topRadius,
            double height,
            out NovaKernel.NovaHandle outHandle)
        {
            var handle = _nextHandle++;
            _bodies[handle] = new StubBody
            {
                Handle = handle,
                Type = "Cone",
                Radius = baseRadius,
                Height = height
            };
            outHandle = new NovaKernel.NovaHandle(handle);
            Console.WriteLine($"[NovaKernelStub] Created Cone: handle={handle}");
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_make_torus(
            double majorRadius,
            double minorRadius,
            out NovaKernel.NovaHandle outHandle)
        {
            var handle = _nextHandle++;
            _bodies[handle] = new StubBody
            {
                Handle = handle,
                Type = "Torus",
                Radius = majorRadius
            };
            outHandle = new NovaKernel.NovaHandle(handle);
            Console.WriteLine($"[NovaKernelStub] Created Torus: handle={handle}");
            return NovaKernel.NovaResult.Success;
        }

        #endregion

        #region Body Operations

        public static NovaKernel.NovaResult nova_body_release(NovaKernel.NovaHandle handle)
        {
            _bodies.Remove(handle.Value);
            Console.WriteLine($"[NovaKernelStub] Released handle={handle.Value}");
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_body_transform(
            NovaKernel.NovaHandle handle,
            ref NovaKernel.NovaTransform transform)
        {
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_body_bounding_box(
            NovaKernel.NovaHandle handle,
            out NovaKernel.NovaBBox3 outBBox)
        {
            // Return a default bounding box
            outBBox = new NovaKernel.NovaBBox3
            {
                Min = new NovaKernel.NovaPoint3(-50, -50, -50),
                Max = new NovaKernel.NovaPoint3(50, 50, 50)
            };
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_body_copy(
            NovaKernel.NovaHandle handle,
            out NovaKernel.NovaHandle outHandle)
        {
            outHandle = handle; // Just return same handle for stub
            return NovaKernel.NovaResult.Success;
        }

        #endregion

        #region Boolean Operations

        public static NovaKernel.NovaResult nova_boolean_unite(
            NovaKernel.NovaHandle bodyA,
            NovaKernel.NovaHandle bodyB,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = bodyA; // Just return first body for stub
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_boolean_subtract(
            NovaKernel.NovaHandle bodyA,
            NovaKernel.NovaHandle bodyB,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = bodyA;
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_boolean_intersect(
            NovaKernel.NovaHandle bodyA,
            NovaKernel.NovaHandle bodyB,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = bodyA;
            return NovaKernel.NovaResult.Success;
        }

        #endregion

        #region Feature Operations

        public static NovaKernel.NovaResult nova_fillet(
            NovaKernel.NovaHandle body,
            NovaKernel.NovaHandle[] edges,
            uint edgeCount,
            double radius,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = body;
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_chamfer(
            NovaKernel.NovaHandle body,
            NovaKernel.NovaHandle[] edges,
            uint edgeCount,
            double distance1,
            double distance2,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = body;
            return NovaKernel.NovaResult.Success;
        }

        public static NovaKernel.NovaResult nova_shell(
            NovaKernel.NovaHandle body,
            NovaKernel.NovaHandle[] faces,
            uint faceCount,
            double thickness,
            out NovaKernel.NovaHandle outResult)
        {
            outResult = body;
            return NovaKernel.NovaResult.Success;
        }

        #endregion

        #region Tessellation

        public static NovaKernel.NovaResult nova_tessellate_body(
            NovaKernel.NovaHandle body,
            double chordTolerance,
            double angleTolerance,
            out NovaKernel.NovaMesh outMesh)
        {
            // Return an empty mesh for stub
            outMesh = new NovaKernel.NovaMesh
            {
                Vertices = IntPtr.Zero,
                VertexCount = 0,
                Indices = IntPtr.Zero,
                IndexCount = 0
            };
            return NovaKernel.NovaResult.NotImplemented;
        }

        public static NovaKernel.NovaResult nova_mesh_free(ref NovaKernel.NovaMesh mesh)
        {
            return NovaKernel.NovaResult.Success;
        }

        #endregion

        #region File I/O

        public static NovaKernel.NovaResult nova_import_step(
            string filepath,
            out NovaKernel.NovaHandle outHandle)
        {
            outHandle = NovaKernel.NovaHandle.Null;
            return NovaKernel.NovaResult.NotImplemented;
        }

        public static NovaKernel.NovaResult nova_export_step(
            NovaKernel.NovaHandle body,
            string filepath)
        {
            return NovaKernel.NovaResult.NotImplemented;
        }

        public static NovaKernel.NovaResult nova_export_stl(
            NovaKernel.NovaHandle body,
            string filepath)
        {
            return NovaKernel.NovaResult.NotImplemented;
        }

        #endregion

        #region Error Handling

        public static IntPtr nova_last_error()
        {
            return Marshal.StringToHGlobalAnsi("Stub mode - no real kernel");
        }

        public static void nova_clear_error()
        {
        }

        #endregion
    }
}
