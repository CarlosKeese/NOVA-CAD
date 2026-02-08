using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using NovaCad.Core.Models;
using NovaCad.Viewport;
using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using System.Windows.Input;
using static NovaCad.Viewport.ViewportDiagnostics;

namespace NovaCad.App.ViewModels
{
    /// <summary>
    /// Represents a visual object in the viewport
    /// </summary>
    public class VisualObject : IVisualObject
    {
        public string Name { get; set; }
        public Mesh Mesh { get; set; }
        public object? BodyRef { get; set; }
        public bool IsVisible { get; set; } = true;
        public bool IsSelected { get; set; }

        public VisualObject(string name, Mesh mesh, object? bodyRef = null)
        {
            Name = name;
            Mesh = mesh;
            BodyRef = bodyRef;
        }
    }

    /// <summary>
    /// Main window view model
    /// </summary>
    public partial class MainWindowViewModel : ObservableObject
    {
        [ObservableProperty]
        private string _title = "Nova CAD";

        [ObservableProperty]
        private NovaDocument? _currentDocument;

        [ObservableProperty]
        private bool _isBusy;

        [ObservableProperty]
        private string _statusMessage = "Ready";

        [ObservableProperty]
        private ViewportViewModel _viewportViewModel;

        [ObservableProperty]
        private ModelTreeViewModel _modelTreeViewModel;

        [ObservableProperty]
        private PropertyPanelViewModel _propertyPanelViewModel;

        [ObservableProperty]
        private RibbonViewModel _ribbonViewModel;

        // Recent files
        public ObservableCollection<string> RecentFiles { get; } = new();

        public MainWindowViewModel()
        {
            _viewportViewModel = new ViewportViewModel();
            _modelTreeViewModel = new ModelTreeViewModel();
            _propertyPanelViewModel = new PropertyPanelViewModel();
            _ribbonViewModel = new RibbonViewModel();

            // Create a new document on startup
            NewDocument();
        }

        [RelayCommand]
        private void NewDocument()
        {
            CurrentDocument?.Dispose();
            CurrentDocument = NovaDocument.CreateNew();
            Title = "Nova CAD - Untitled";
            StatusMessage = "New document created";
            ViewportViewModel.ClearVisualObjects();
            ModelTreeViewModel.Bodies.Clear();
        }

        [RelayCommand]
        private async Task OpenDocument()
        {
            // TODO: Implement file dialog
            await Task.Delay(100);
            StatusMessage = "Open document not implemented";
        }

        [RelayCommand]
        private async Task SaveDocument()
        {
            if (CurrentDocument == null) return;

            if (string.IsNullOrEmpty(CurrentDocument.FilePath))
            {
                await SaveDocumentAs();
            }
            else
            {
                // TODO: Implement save
                await Task.Delay(100);
                CurrentDocument.MarkAsSaved();
                StatusMessage = "Document saved";
            }
        }

        [RelayCommand]
        private async Task SaveDocumentAs()
        {
            // TODO: Implement save as dialog
            await Task.Delay(100);
            StatusMessage = "Save as not implemented";
        }

        [RelayCommand]
        private void Exit()
        {
            // This will be handled by the application
            Environment.Exit(0);
        }

        [RelayCommand]
        private void Undo()
        {
            StatusMessage = "Undo not implemented";
        }

        [RelayCommand]
        private void Redo()
        {
            StatusMessage = "Redo not implemented";
        }

        [RelayCommand]
        private void CreateBox()
        {
            ViewportDiagnostics.Log("CreateBox command executed", LogLevel.Info);
            try
            {
                var result = Kernel.NovaKernel.nova_make_box(100, 50, 30, out var handle);
                ViewportDiagnostics.Log($"nova_make_box result: {result}", LogLevel.Debug);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    ViewportDiagnostics.Log("Box created in kernel, creating body ref", LogLevel.Debug);
                    var body = new NovaBodyRef(handle.Value, "Box");
                    CurrentDocument?.AddBody(body);
                    ModelTreeViewModel.Bodies.Add(body);
                    ViewportDiagnostics.Log($"Body added to document and model tree. Current bodies: {CurrentDocument?.Bodies.Count}", LogLevel.Debug);
                    
                    // Create mesh for the box
                    ViewportDiagnostics.Log("Creating mesh for box...", LogLevel.Debug);
                    var mesh = MeshFactory.CreateBox(100, 50, 30);
                    mesh.Name = "Box";
                    ViewportDiagnostics.Log($"Mesh created: {mesh.Vertices.Count} vertices, {mesh.Indices.Count} indices", LogLevel.Debug);
                    
                    ViewportDiagnostics.Log("Adding visual object to ViewportViewModel...", LogLevel.Debug);
                    ViewportViewModel.AddVisualObject("Box", mesh, body);
                    ViewportDiagnostics.Log($"Visual objects count: {ViewportViewModel.VisualObjects.Count}", LogLevel.Debug);
                    
                    StatusMessage = "Box created";
                }
                else
                {
                    StatusMessage = $"Failed to create box: {result}";
                }
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error: {ex.Message}";
            }
        }

        [RelayCommand]
        private void CreateCylinder()
        {
            ViewportDiagnostics.Log("CreateCylinder command executed", LogLevel.Info);
            try
            {
                var result = Kernel.NovaKernel.nova_make_cylinder(25, 100, out var handle);
                ViewportDiagnostics.Log($"nova_make_cylinder result: {result}", LogLevel.Debug);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    var body = new NovaBodyRef(handle.Value, "Cylinder");
                    CurrentDocument?.AddBody(body);
                    ModelTreeViewModel.Bodies.Add(body);
                    
                    // Create mesh for the cylinder
                    var mesh = MeshFactory.CreateCylinder(25, 100, 32);
                    mesh.Name = "Cylinder";
                    ViewportViewModel.AddVisualObject("Cylinder", mesh, body);
                    
                    StatusMessage = "Cylinder created";
                }
                else
                {
                    StatusMessage = $"Failed to create cylinder: {result}";
                }
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error: {ex.Message}";
            }
        }

        [RelayCommand]
        private void CreateSphere()
        {
            ViewportDiagnostics.Log("CreateSphere command executed", LogLevel.Info);
            try
            {
                var result = Kernel.NovaKernel.nova_make_sphere(30, out var handle);
                ViewportDiagnostics.Log($"nova_make_sphere result: {result}", LogLevel.Debug);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    var body = new NovaBodyRef(handle.Value, "Sphere");
                    CurrentDocument?.AddBody(body);
                    ModelTreeViewModel.Bodies.Add(body);
                    
                    // Create mesh for the sphere
                    var mesh = MeshFactory.CreateSphere(30, 32, 16);
                    mesh.Name = "Sphere";
                    ViewportViewModel.AddVisualObject("Sphere", mesh, body);
                    
                    StatusMessage = "Sphere created";
                }
                else
                {
                    StatusMessage = $"Failed to create sphere: {result}";
                }
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error: {ex.Message}";
            }
        }

        [RelayCommand]
        private void FitAll()
        {
            ViewportViewModel.FitAll();
            StatusMessage = "Fit all";
        }

        [RelayCommand]
        private void SetViewFront()
        {
            ViewportViewModel.SetViewFront();
            StatusMessage = "Front view";
        }

        [RelayCommand]
        private void SetViewTop()
        {
            ViewportViewModel.SetViewTop();
            StatusMessage = "Top view";
        }

        [RelayCommand]
        private void SetViewRight()
        {
            ViewportViewModel.SetViewRight();
            StatusMessage = "Right view";
        }

        [RelayCommand]
        private void SetViewIso()
        {
            ViewportViewModel.SetViewIso();
            StatusMessage = "Isometric view";
        }

        [RelayCommand]
        private void ToggleShadedMode()
        {
            ViewportViewModel.RenderMode = RenderMode.Shaded;
            StatusMessage = "Shaded mode";
        }

        [RelayCommand]
        private void ToggleWireframeMode()
        {
            ViewportViewModel.RenderMode = RenderMode.Wireframe;
            StatusMessage = "Wireframe mode";
        }

        [RelayCommand]
        private void ToggleShadedWithEdgesMode()
        {
            ViewportViewModel.RenderMode = RenderMode.ShadedWithEdges;
            StatusMessage = "Shaded with edges mode";
        }

        [RelayCommand]
        private void ShowAbout()
        {
            var version = Kernel.NovaKernel.GetVersion();
            StatusMessage = $"Nova CAD - {version}";
        }
    }

    /// <summary>
    /// Viewport view model
    /// </summary>
    public partial class ViewportViewModel : ObservableObject, IViewportViewModel
    {
        [ObservableProperty]
        private RenderMode _renderMode = RenderMode.ShadedWithEdges;

        [ObservableProperty]
        private bool _showGrid = true;

        [ObservableProperty]
        private bool _showAxes = true;

        // ObservableCollection of VisualObject (which implements IVisualObject)
        public ObservableCollection<VisualObject> VisualObjects { get; } = new();

        // Explicit interface implementation for IViewportViewModel
        ObservableCollection<IVisualObject> IViewportViewModel.VisualObjects
        {
            get
            {
                var collection = new ObservableCollection<IVisualObject>();
                foreach (var obj in VisualObjects)
                    collection.Add(obj);
                return collection;
            }
        }

        public event EventHandler<VisualObjectCreatedEventArgs>? VisualObjectCreated;
        public event EventHandler? VisualObjectsCleared;
        public event EventHandler<RenderMode>? RenderModeChanged;
        public event EventHandler<string>? ViewPresetChanged;
        public event EventHandler? FitAllRequested;

        partial void OnRenderModeChanged(RenderMode value)
        {
            RenderModeChanged?.Invoke(this, value);
        }

        public void AddVisualObject(string name, Mesh mesh, object? bodyRef = null)
        {
            ViewportDiagnostics.Log($"AddVisualObject called: {name}", LogLevel.Debug);
            var visualObject = new VisualObject(name, mesh, bodyRef);
            VisualObjects.Add(visualObject);
            ViewportDiagnostics.Log($"VisualObject added to collection. Firing event...", LogLevel.Debug);
            VisualObjectCreated?.Invoke(this, new VisualObjectCreatedEventArgs(name, mesh, bodyRef));
            ViewportDiagnostics.Log($"VisualObjectCreated event fired", LogLevel.Debug);
        }

        public void ClearVisualObjects()
        {
            VisualObjects.Clear();
            VisualObjectsCleared?.Invoke(this, EventArgs.Empty);
        }

        public void FitAll()
        {
            FitAllRequested?.Invoke(this, EventArgs.Empty);
        }

        public void SetViewFront()
        {
            ViewPresetChanged?.Invoke(this, "Front");
        }

        public void SetViewTop()
        {
            ViewPresetChanged?.Invoke(this, "Top");
        }

        public void SetViewRight()
        {
            ViewPresetChanged?.Invoke(this, "Right");
        }

        public void SetViewIso()
        {
            ViewPresetChanged?.Invoke(this, "Iso");
        }
    }

    /// <summary>
    /// Model tree view model
    /// </summary>
    public partial class ModelTreeViewModel : ObservableObject
    {
        [ObservableProperty]
        private ObservableCollection<NovaBodyRef> _bodies = new();
    }

    /// <summary>
    /// Property panel view model
    /// </summary>
    public partial class PropertyPanelViewModel : ObservableObject
    {
        [ObservableProperty]
        private object? _selectedObject;
    }

    /// <summary>
    /// Ribbon view model
    /// </summary>
    public partial class RibbonViewModel : ObservableObject
    {
        [ObservableProperty]
        private string _activeTab = "Home";

        public string[] Tabs { get; } = new[]
        {
            "Home",
            "Insert",
            "Modify",
            "Inspect",
            "View",
            "Mold Tools"
        };
    }
}
