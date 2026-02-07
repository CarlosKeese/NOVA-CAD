using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using CommunityToolkit.Mvvm.ComponentModel;

namespace NovaCad.Core.Models
{
    /// <summary>
    /// Represents a Nova CAD document containing bodies and design data
    /// </summary>
    public partial class NovaDocument : ObservableObject, IDisposable
    {
        [ObservableProperty]
        private string _filePath = string.Empty;

        [ObservableProperty]
        private string _documentName = "Untitled";

        [ObservableProperty]
        private bool _isModified;

        [ObservableProperty]
        private DateTime _createdDate;

        [ObservableProperty]
        private DateTime _lastModifiedDate;

        [ObservableProperty]
        private string _author = string.Empty;

        [ObservableProperty]
        private string _units = "mm";

        [ObservableProperty]
        private double _tolerance = 1e-6;

        /// <summary>
        /// Collection of bodies in the document
        /// </summary>
        public ObservableCollection<NovaBodyRef> Bodies { get; } = new();

        /// <summary>
        /// Currently active body for editing
        /// </summary>
        [ObservableProperty]
        private NovaBodyRef? _activeBody;

        /// <summary>
        /// Current selection set
        /// </summary>
        public SelectionSet Selection { get; } = new();

        /// <summary>
        /// Document metadata
        /// </summary>
        public DocumentMetadata Metadata { get; set; } = new();

        /// <summary>
        /// View state (camera positions, etc.)
        /// </summary>
        public ViewState ViewState { get; set; } = new();

        /// <summary>
        /// Material library for this document
        /// </summary>
        public MaterialLibrary Materials { get; set; } = new();

        public NovaDocument()
        {
            CreatedDate = DateTime.Now;
            LastModifiedDate = DateTime.Now;
            
            Bodies.CollectionChanged += (s, e) =>
            {
                IsModified = true;
                LastModifiedDate = DateTime.Now;
            };
        }

        /// <summary>
        /// Create a new document
        /// </summary>
        public static NovaDocument CreateNew(string name = "Untitled")
        {
            return new NovaDocument
            {
                DocumentName = name,
                CreatedDate = DateTime.Now,
                LastModifiedDate = DateTime.Now
            };
        }

        /// <summary>
        /// Add a body to the document
        /// </summary>
        public void AddBody(NovaBodyRef body)
        {
            Bodies.Add(body);
            if (ActiveBody == null)
            {
                ActiveBody = body;
            }
            IsModified = true;
        }

        /// <summary>
        /// Remove a body from the document
        /// </summary>
        public bool RemoveBody(NovaBodyRef body)
        {
            var result = Bodies.Remove(body);
            if (result)
            {
                if (ActiveBody == body)
                {
                    ActiveBody = Bodies.Count > 0 ? Bodies[0] : null;
                }
                IsModified = true;
            }
            return result;
        }

        /// <summary>
        /// Mark the document as saved
        /// </summary>
        public void MarkAsSaved(string? filePath = null)
        {
            if (filePath != null)
            {
                FilePath = filePath;
                DocumentName = System.IO.Path.GetFileNameWithoutExtension(filePath);
            }
            IsModified = false;
        }

        public void Dispose()
        {
            foreach (var body in Bodies)
            {
                body.Dispose();
            }
            Bodies.Clear();
        }
    }

    /// <summary>
    /// Reference to a body in the document
    /// </summary>
    public partial class NovaBodyRef : ObservableObject, IDisposable
    {
        [ObservableProperty]
        private ulong _kernelHandle;

        [ObservableProperty]
        private string _name = "Body";

        [ObservableProperty]
        private bool _isVisible = true;

        [ObservableProperty]
        private bool _isSelected;

        [ObservableProperty]
        private NovaColor _color = new(0.7, 0.7, 0.7, 1.0);

        [ObservableProperty]
        private double _transparency;

        [ObservableProperty]
        private DateTime _createdDate;

        [ObservableProperty]
        private BoundingBox3 _boundingBox;

        /// <summary>
        /// Parent document
        /// </summary>
        public NovaDocument? Document { get; set; }

        /// <summary>
        /// User-defined tags
        /// </summary>
        public ObservableCollection<string> Tags { get; } = new();

        public NovaBodyRef(ulong kernelHandle, string name)
        {
            KernelHandle = kernelHandle;
            Name = name;
            CreatedDate = DateTime.Now;
        }

        public void Dispose()
        {
            if (KernelHandle != 0)
            {
                // Release the kernel handle
                // NovaKernel.nova_body_release(new NovaKernel.NovaHandle(KernelHandle));
                KernelHandle = 0;
            }
        }
    }

    /// <summary>
    /// Selection set for the document
    /// </summary>
    public partial class SelectionSet : ObservableObject
    {
        [ObservableProperty]
        private ObservableCollection<SelectedEntity> _entities = new();

        [ObservableProperty]
        private SelectionMode _mode = SelectionMode.Face;

        [ObservableProperty]
        private bool _isEmpty = true;

        public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;

        public void Clear()
        {
            Entities.Clear();
            IsEmpty = true;
            SelectionChanged?.Invoke(this, new SelectionChangedEventArgs(null, false));
        }

        public void Add(SelectedEntity entity)
        {
            Entities.Add(entity);
            IsEmpty = false;
            SelectionChanged?.Invoke(this, new SelectionChangedEventArgs(entity, true));
        }

        public void Remove(SelectedEntity entity)
        {
            Entities.Remove(entity);
            IsEmpty = Entities.Count == 0;
            SelectionChanged?.Invoke(this, new SelectionChangedEventArgs(entity, false));
        }

        public bool Contains(SelectedEntity entity)
        {
            return Entities.Contains(entity);
        }
    }

    /// <summary>
    /// Selected entity information
    /// </summary>
    public record SelectedEntity
    {
        public required ulong BodyHandle { get; init; }
        public required EntityType Type { get; init; }
        public required ulong EntityId { get; init; }
        public string? Name { get; init; }
    }

    /// <summary>
    /// Entity types for selection
    /// </summary>
    public enum EntityType
    {
        Body,
        Face,
        Edge,
        Vertex,
        Feature
    }

    /// <summary>
    /// Selection modes
    /// </summary>
    public enum SelectionMode
    {
        Face,
        Edge,
        Vertex,
        Body,
        Feature
    }

    /// <summary>
    /// Selection changed event arguments
    /// </summary>
    public class SelectionChangedEventArgs : EventArgs
    {
        public SelectedEntity? Entity { get; }
        public bool Added { get; }

        public SelectionChangedEventArgs(SelectedEntity? entity, bool added)
        {
            Entity = entity;
            Added = added;
        }
    }

    /// <summary>
    /// Document metadata
    /// </summary>
    public class DocumentMetadata
    {
        public string Title { get; set; } = string.Empty;
        public string Subject { get; set; } = string.Empty;
        public string Author { get; set; } = string.Empty;
        public string Keywords { get; set; } = string.Empty;
        public string Comments { get; set; } = string.Empty;
        public string Company { get; set; } = string.Empty;
        public string Category { get; set; } = string.Empty;
    }

    /// <summary>
    /// View state (camera positions, etc.)
    /// </summary>
    public class ViewState
    {
        public CameraState Camera { get; set; } = new();
        public RenderMode RenderMode { get; set; } = RenderMode.ShadedWithEdges;
        public bool ShowGrid { get; set; } = true;
        public bool ShowAxes { get; set; } = true;
    }

    /// <summary>
    /// Camera state
    /// </summary>
    public class CameraState
    {
        public NovaPoint3 Position { get; set; } = new(100, 100, 100);
        public NovaPoint3 Target { get; set; } = new(0, 0, 0);
        public NovaVec3 Up { get; set; } = new(0, 0, 1);
        public double FieldOfView { get; set; } = 45.0;
        public bool IsOrthographic { get; set; } = false;
        public double OrthographicScale { get; set; } = 100.0;
    }

    /// <summary>
    /// Render modes
    /// </summary>
    public enum RenderMode
    {
        Shaded,
        ShadedWithEdges,
        Wireframe,
        HiddenLine,
        XRay,
        DraftAnalysis,
        CurvatureMap,
        ZebraStripes
    }

    /// <summary>
    /// Material library
    /// </summary>
    public class MaterialLibrary
    {
        public ObservableCollection<NovaMaterial> Materials { get; } = new();
    }

    /// <summary>
    /// Material definition
    /// </summary>
    public class NovaMaterial
    {
        public string Name { get; set; } = "Default";
        public NovaColor DiffuseColor { get; set; } = new(0.7, 0.7, 0.7, 1.0);
        public NovaColor SpecularColor { get; set; } = new(1.0, 1.0, 1.0, 1.0);
        public double Shininess { get; set; } = 32.0;
        public double Transparency { get; set; } = 0.0;
    }

    /// <summary>
    /// Color structure
    /// </summary>
    public record NovaColor(double R, double G, double B, double A)
    {
        public static readonly NovaColor White = new(1, 1, 1, 1);
        public static readonly NovaColor Black = new(0, 0, 0, 1);
        public static readonly NovaColor Red = new(1, 0, 0, 1);
        public static readonly NovaColor Green = new(0, 1, 0, 1);
        public static readonly NovaColor Blue = new(0, 0, 1, 1);
        public static readonly NovaColor Gray = new(0.5, 0.5, 0.5, 1);
        public static readonly NovaColor LightGray = new(0.8, 0.8, 0.8, 1);
        public static readonly NovaColor DarkGray = new(0.3, 0.3, 0.3, 1);
    }

    /// <summary>
    /// 3D point structure
    /// </summary>
    public record NovaPoint3(double X, double Y, double Z);

    /// <summary>
    /// 3D vector structure
    /// </summary>
    public record NovaVec3(double X, double Y, double Z);

    /// <summary>
    /// 3D bounding box
    /// </summary>
    public record BoundingBox3(NovaPoint3 Min, NovaPoint3 Max)
    {
        public static readonly BoundingBox3 Empty = new(
            new NovaPoint3(double.PositiveInfinity, double.PositiveInfinity, double.PositiveInfinity),
            new NovaPoint3(double.NegativeInfinity, double.NegativeInfinity, double.NegativeInfinity)
        );

        public bool IsEmpty => Min.X > Max.X || Min.Y > Max.Y || Min.Z > Max.Z;

        public NovaPoint3 Center => new(
            (Min.X + Max.X) * 0.5,
            (Min.Y + Max.Y) * 0.5,
            (Min.Z + Max.Z) * 0.5
        );

        public double Diagonal => Math.Sqrt(
            Math.Pow(Max.X - Min.X, 2) +
            Math.Pow(Max.Y - Min.Y, 2) +
            Math.Pow(Max.Z - Min.Z, 2)
        );
    }
}
