using Avalonia.Controls;
using Avalonia.ReactiveUI;
using NovaCad.App.ViewModels;
using NovaCad.Viewport;

namespace NovaCad.App.Views
{
    public partial class MainWindow : ReactiveWindow<MainWindowViewModel>
    {
        public MainWindow()
        {
            InitializeComponent();

            // Connect ViewModel to ViewportControl after initialization
            this.DataContextChanged += (s, e) =>
            {
                ViewportDiagnostics.Log($"DataContextChanged: {(DataContext != null ? "not null" : "NULL")}", LogLevel.Info);
                
                if (DataContext is MainWindowViewModel vm)
                {
                    var viewport = this.FindControl<ViewportControl>("MainViewport");
                    ViewportDiagnostics.Log($"Found ViewportControl: {(viewport != null ? "yes" : "NO")}", LogLevel.Info);
                    
                    if (viewport != null)
                    {
                        viewport.ViewModel = vm.ViewportViewModel;
                        ViewportDiagnostics.Log("ViewModel connected to ViewportControl", LogLevel.Info);
                    }
                    else
                    {
                        ViewportDiagnostics.Log("ERROR: ViewportControl not found!", LogLevel.Error);
                    }
                }
            };
        }
    }
}
