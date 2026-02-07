using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using Silk.NET.OpenGL;
using NovaCad.Core.Models;
using NovaCad.Kernel;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Mold design tools for creating mold cavities, cores, and parting lines
    /// </summary>
    public class MoldTools
    {
        private GL _gl;
        
        public MoldTools(GL gl)
        {
            _gl = gl;
        }

        /// <summary>
        /// Create a mold cavity from a body
        /// </summary>
        public MoldResult CreateMoldCavity(
            NovaBodyRef body,
            MoldParameters parameters,
            out string error)
        {
            error = string.Empty;
            
            try
            {
                // Validate parting direction
                if (parameters.PartingDirection.LengthSquared() < 0.001f)
                {
                    error = "Invalid parting direction";
                    return null;
                }

                // Analyze undercuts
                var undercuts = AnalyzeUndercuts(body, parameters.PartingDirection);
                if (undercuts.Count > 0 && !parameters.AllowUndercuts)
                {
                    error = $"Found {undercuts.Count} undercuts. Enable 'Allow Undercuts' or adjust parting direction.";
                    return null;
                }

                // Create bounding box for mold base
                var bbox = GetBodyBoundingBox(body);
                var moldBase = CreateMoldBase(bbox, parameters);

                // Create cavity by subtracting body from mold base
                var cavity = CreateCavityGeometry(body, moldBase, parameters);

                // Create core
                var core = CreateCoreGeometry(body, moldBase, parameters);

                // Generate parting line
                var partingLine = GeneratePartingLine(body, parameters);

                return new MoldResult
                {
                    Cavity = cavity,
                    Core = core,
                    MoldBase = moldBase,
                    PartingLine = partingLine,
                    Undercuts = undercuts,
                    DraftAnalysis = AnalyzeDraft(body, parameters.PartingDirection, parameters.DraftAngle)
                };
            }
            catch (Exception ex)
            {
                error = $"Mold creation failed: {ex.Message}";
                return null;
            }
        }

        /// <summary>
        /// Analyze undercuts in the body
        /// </summary>
        public List<UndercutInfo> AnalyzeUndercuts(NovaBodyRef body, Vector3 partingDirection)
        {
            var undercuts = new List<UndercutInfo>();
            var faces = NovaKernel.GetFaces(body);
            
            Vector3 dir = Vector3.Normalize(partingDirection);
            
            foreach (var face in faces)
            {
                var normal = NovaKernel.GetFaceNormal(face);
                float dot = Vector3.Dot(normal, dir);
                
                // If face normal points opposite to parting direction, it's an undercut
                if (dot < -0.001f)
                {
                    undercuts.Add(new UndercutInfo
                    {
                        FaceId = face.Handle,
                        Normal = normal,
                        Severity = Math.Abs(dot),
                        Area = NovaKernel.GetFaceArea(face)
                    });
                }
            }
            
            return undercuts.OrderByDescending(u => u.Severity).ToList();
        }

        /// <summary>
        /// Analyze draft angles on all faces
        /// </summary>
        public DraftAnalysisResult AnalyzeDraft(NovaBodyRef body, Vector3 partingDirection, float minDraftAngle)
        {
            var result = new DraftAnalysisResult();
            var faces = NovaKernel.GetFaces(body);
            
            Vector3 dir = Vector3.Normalize(partingDirection);
            float minCos = MathF.Cos(minDraftAngle * MathF.PI / 180.0f);
            
            foreach (var face in faces)
            {
                var normal = NovaKernel.GetFaceNormal(face);
                float dot = Vector3.Dot(normal, dir);
                float angle = MathF.Acos(Math.Abs(dot)) * 180.0f / MathF.PI;
                
                var info = new DraftFaceInfo
                {
                    FaceId = face.Handle,
                    DraftAngle = angle,
                    IsPositiveDraft = dot > 0,
                    NeedsDraft = Math.Abs(dot) < minCos
                };
                
                result.Faces.Add(info);
                
                if (info.NeedsDraft)
                {
                    result.UndraftedFaces.Add(info);
                }
            }
            
            return result;
        }

        /// <summary>
        /// Generate parting line for the mold
        /// </summary>
        public PartingLineResult GeneratePartingLine(NovaBodyRef body, MoldParameters parameters)
        {
            var result = new PartingLineResult();
            var edges = NovaKernel.GetEdges(body);
            
            Vector3 dir = Vector3.Normalize(parameters.PartingDirection);
            
            // Find edges where adjacent faces have different draft directions
            foreach (var edge in edges)
            {
                var faces = NovaKernel.GetAdjacentFaces(edge);
                if (faces.Count != 2) continue;
                
                var normal1 = NovaKernel.GetFaceNormal(faces[0]);
                var normal2 = NovaKernel.GetFaceNormal(faces[1]);
                
                float dot1 = Vector3.Dot(normal1, dir);
                float dot2 = Vector3.Dot(normal2, dir);
                
                // If one face is positive draft and other is negative, this edge is on parting line
                if ((dot1 > 0 && dot2 < 0) || (dot1 < 0 && dot2 > 0))
                {
                    result.Edges.Add(edge.Handle);
                    result.Vertices.AddRange(NovaKernel.GetEdgeVertices(edge));
                }
            }
            
            return result;
        }

        /// <summary>
        /// Create split mold with multiple parting lines
        /// </summary>
        public SplitMoldResult CreateSplitMold(
            NovaBodyRef body,
            List<SplitPlane> splitPlanes,
            MoldParameters parameters)
        {
            var result = new SplitMoldResult();
            
            // Sort split planes by position along parting direction
            splitPlanes = splitPlanes.OrderBy(p => p.Position).ToList();
            
            // Create mold pieces
            for (int i = 0; i <= splitPlanes.Count; i++)
            {
                var pieceParams = parameters.Clone();
                
                if (i == 0)
                {
                    // First piece - from negative infinity to first plane
                    pieceParams.SplitRange = (float.NegativeInfinity, splitPlanes[0].Position);
                }
                else if (i == splitPlanes.Count)
                {
                    // Last piece - from last plane to positive infinity
                    pieceParams.SplitRange = (splitPlanes.Last().Position, float.PositiveInfinity);
                }
                else
                {
                    // Middle piece - between two planes
                    pieceParams.SplitRange = (splitPlanes[i - 1].Position, splitPlanes[i].Position);
                }
                
                var piece = CreateMoldPiece(body, pieceParams, i);
                result.Pieces.Add(piece);
            }
            
            return result;
        }

        /// <summary>
        /// Calculate shrinkage compensation
        /// </summary>
        public NovaBodyRef ApplyShrinkage(NovaBodyRef body, float shrinkagePercent)
        {
            float scale = 1.0f + (shrinkagePercent / 100.0f);
            var transform = Matrix4x4.CreateScale(scale);
            return NovaKernel.TransformBody(body, transform);
        }

        /// <summary>
        /// Create cooling channels
        /// </summary>
        public List<CoolingChannel> CreateCoolingChannels(
            BoundingBox moldBounds,
            CoolingChannelParameters parameters)
        {
            var channels = new List<CoolingChannel>();
            
            // Calculate channel layout
            int rows = (int)((moldBounds.Max.Y - moldBounds.Min.Y) / parameters.SpacingY);
            int cols = (int)((moldBounds.Max.X - moldBounds.Min.X) / parameters.SpacingX);
            
            for (int row = 0; row < rows; row++)
            {
                for (int col = 0; col < cols; col++)
                {
                    var channel = new CoolingChannel
                    {
                        Start = new Vector3(
                            moldBounds.Min.X + col * parameters.SpacingX,
                            moldBounds.Min.Y + row * parameters.SpacingY,
                            moldBounds.Min.Z + parameters.DepthFromSurface),
                        End = new Vector3(
                            moldBounds.Min.X + col * parameters.SpacingX,
                            moldBounds.Min.Y + row * parameters.SpacingY,
                            moldBounds.Max.Z - parameters.DepthFromSurface),
                        Diameter = parameters.Diameter
                    };
                    
                    channels.Add(channel);
                }
            }
            
            return channels;
        }

        /// <summary>
        /// Create ejector pins
        /// </summary>
        public List<EjectorPin> CreateEjectorPins(
            NovaBodyRef body,
            EjectorPinParameters parameters)
        {
            var pins = new List<EjectorPin>();
            var faces = NovaKernel.GetFaces(body);
            
            // Find bottom faces (opposite to parting direction)
            Vector3 up = Vector3.UnitZ;
            var bottomFaces = faces.Where(f => 
                Vector3.Dot(NovaKernel.GetFaceNormal(f), up) < -0.9f).ToList();
            
            // Place pins in a grid pattern on bottom faces
            foreach (var face in bottomFaces)
            {
                var bounds = NovaKernel.GetFaceBoundingBox(face);
                var positions = CalculatePinPositions(bounds, parameters);
                
                foreach (var pos in positions)
                {
                    pins.Add(new EjectorPin
                    {
                        Position = pos,
                        Direction = up,
                        Diameter = parameters.Diameter,
                        Length = parameters.Length,
                        FaceId = face.Handle
                    });
                }
            }
            
            return pins;
        }

        /// <summary>
        /// Create venting channels
        /// </summary>
        public List<VentChannel> CreateVentChannels(
            PartingLineResult partingLine,
            VentParameters parameters)
        {
            var vents = new List<VentChannel>();
            
            // Create vents along parting line
            foreach (var edgeId in partingLine.Edges)
            {
                var edge = new NovaEdgeRef { Handle = edgeId };
                var vertices = NovaKernel.GetEdgeVertices(edge);
                
                if (vertices.Count >= 2)
                {
                    var start = NovaKernel.GetVertexPosition(vertices[0]);
                    var end = NovaKernel.GetVertexPosition(vertices[1]);
                    
                    vents.Add(new VentChannel
                    {
                        Start = start,
                        End = end,
                        Width = parameters.Width,
                        Depth = parameters.Depth
                    });
                }
            }
            
            return vents;
        }

        private BoundingBox GetBodyBoundingBox(NovaBodyRef body)
        {
            var bbox = NovaKernel.GetBoundingBox(body);
            return new BoundingBox(
                new Vector3(bbox.MinX, bbox.MinY, bbox.MinZ),
                new Vector3(bbox.MaxX, bbox.MaxY, bbox.MaxZ));
        }

        private NovaBodyRef CreateMoldBase(BoundingBox bbox, MoldParameters parameters)
        {
            // Create box larger than body by wall thickness
            float wall = parameters.WallThickness;
            var min = new Vector3(bbox.Min.X - wall, bbox.Min.Y - wall, bbox.Min.Z - wall);
            var max = new Vector3(bbox.Max.X + wall, bbox.Max.Y + wall, bbox.Max.Z + wall);
            
            return NovaKernel.CreateBox(
                max.X - min.X,
                max.Y - min.Y,
                max.Z - min.Z);
        }

        private NovaBodyRef CreateCavityGeometry(NovaBodyRef body, NovaBodyRef moldBase, MoldParameters parameters)
        {
            // Boolean subtract body from mold base
            return NovaKernel.BooleanSubtract(moldBase, body);
        }

        private NovaBodyRef CreateCoreGeometry(NovaBodyRef body, NovaBodyRef moldBase, MoldParameters parameters)
        {
            // Create core on opposite side of parting line
            // This is a simplified version
            return NovaKernel.BooleanSubtract(moldBase, body);
        }

        private MoldPiece CreateMoldPiece(NovaBodyRef body, MoldParameters parameters, int index)
        {
            return new MoldPiece
            {
                Index = index,
                Name = $"MoldPiece_{index}",
                // Actual geometry creation would go here
            };
        }

        private List<Vector3> CalculatePinPositions(BoundingBox faceBounds, EjectorPinParameters parameters)
        {
            var positions = new List<Vector3>();
            
            int rows = Math.Max(2, (int)((faceBounds.Max.Y - faceBounds.Min.Y) / parameters.Spacing));
            int cols = Math.Max(2, (int)((faceBounds.Max.X - faceBounds.Min.X) / parameters.Spacing));
            
            for (int i = 0; i < rows; i++)
            {
                for (int j = 0; j < cols; j++)
                {
                    float x = faceBounds.Min.X + (faceBounds.Max.X - faceBounds.Min.X) * (j + 1) / (cols + 1);
                    float y = faceBounds.Min.Y + (faceBounds.Max.Y - faceBounds.Min.Y) * (i + 1) / (rows + 1);
                    float z = faceBounds.Min.Z;
                    
                    positions.Add(new Vector3(x, y, z));
                }
            }
            
            return positions;
        }
    }

    public class MoldParameters
    {
        public Vector3 PartingDirection { get; set; } = Vector3.UnitZ;
        public float WallThickness { get; set; } = 10.0f;
        public float DraftAngle { get; set; } = 1.0f;
        public float ShrinkagePercent { get; set; } = 0.5f;
        public bool AllowUndercuts { get; set; } = false;
        public (float Min, float Max) SplitRange { get; set; }

        public MoldParameters Clone()
        {
            return (MoldParameters)MemberwiseClone();
        }
    }

    public class MoldResult
    {
        public NovaBodyRef Cavity { get; set; }
        public NovaBodyRef Core { get; set; }
        public NovaBodyRef MoldBase { get; set; }
        public PartingLineResult PartingLine { get; set; }
        public List<UndercutInfo> Undercuts { get; set; }
        public DraftAnalysisResult DraftAnalysis { get; set; }
    }

    public class PartingLineResult
    {
        public List<uint> Edges { get; set; } = new();
        public List<uint> Vertices { get; set; } = new();
    }

    public class UndercutInfo
    {
        public uint FaceId { get; set; }
        public Vector3 Normal { get; set; }
        public float Severity { get; set; }
        public float Area { get; set; }
    }

    public class DraftAnalysisResult
    {
        public List<DraftFaceInfo> Faces { get; set; } = new();
        public List<DraftFaceInfo> UndraftedFaces => Faces.Where(f => f.NeedsDraft).ToList();
    }

    public class DraftFaceInfo
    {
        public uint FaceId { get; set; }
        public float DraftAngle { get; set; }
        public bool IsPositiveDraft { get; set; }
        public bool NeedsDraft { get; set; }
    }

    public class SplitMoldResult
    {
        public List<MoldPiece> Pieces { get; set; } = new();
    }

    public class MoldPiece
    {
        public int Index { get; set; }
        public string Name { get; set; }
        public NovaBodyRef Geometry { get; set; }
    }

    public class SplitPlane
    {
        public Vector3 Normal { get; set; }
        public float Position { get; set; }
    }

    public class CoolingChannelParameters
    {
        public float Diameter { get; set; } = 8.0f;
        public float SpacingX { get; set; } = 40.0f;
        public float SpacingY { get; set; } = 40.0f;
        public float DepthFromSurface { get; set; } = 20.0f;
    }

    public class CoolingChannel
    {
        public Vector3 Start { get; set; }
        public Vector3 End { get; set; }
        public float Diameter { get; set; }
    }

    public class EjectorPinParameters
    {
        public float Diameter { get; set; } = 3.0f;
        public float Length { get; set; } = 50.0f;
        public float Spacing { get; set; } = 20.0f;
    }

    public class EjectorPin
    {
        public Vector3 Position { get; set; }
        public Vector3 Direction { get; set; }
        public float Diameter { get; set; }
        public float Length { get; set; }
        public uint FaceId { get; set; }
    }

    public class VentParameters
    {
        public float Width { get; set; } = 2.0f;
        public float Depth { get; set; } = 0.5f;
    }

    public class VentChannel
    {
        public Vector3 Start { get; set; }
        public Vector3 End { get; set; }
        public float Width { get; set; }
        public float Depth { get; set; }
    }
}
