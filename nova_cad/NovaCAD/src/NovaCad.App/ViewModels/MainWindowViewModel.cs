using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using NovaCad.Core.Models;
using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using System.Windows.Input;

namespace NovaCad.App.ViewModels
{
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
            try
            {
                var result = Kernel.NovaKernel.nova_make_box(100, 50, 30, out var handle);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    var body = new NovaBodyRef(handle.Value, "Box");
                    CurrentDocument?.AddBody(body);
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
            try
            {
                var result = Kernel.NovaKernel.nova_make_cylinder(25, 100, out var handle);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    var body = new NovaBodyRef(handle.Value, "Cylinder");
                    CurrentDocument?.AddBody(body);
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
            try
            {
                var result = Kernel.NovaKernel.nova_make_sphere(30, out var handle);
                if (result == Kernel.NovaKernel.NovaResult.Success)
                {
                    var body = new NovaBodyRef(handle.Value, "Sphere");
                    CurrentDocument?.AddBody(body);
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
    public partial class ViewportViewModel : ObservableObject
    {
        [ObservableProperty]
        private RenderMode _renderMode = RenderMode.ShadedWithEdges;

        [ObservableProperty]
        private bool _showGrid = true;

        [ObservableProperty]
        private bool _showAxes = true;

        public void FitAll()
        {
            // TODO: Implement fit all
        }

        public void SetViewFront()
        {
            // TODO: Implement set view
        }

        public void SetViewTop()
        {
            // TODO: Implement set view
        }

        public void SetViewRight()
        {
            // TODO: Implement set view
        }

        public void SetViewIso()
        {
            // TODO: Implement set view
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
