using Avalonia.Controls;
using Avalonia.ReactiveUI;
using NovaCad.App.ViewModels;

namespace NovaCad.App.Views
{
    public partial class MainWindow : ReactiveWindow<MainWindowViewModel>
    {
        public MainWindow()
        {
            InitializeComponent();
        }
    }
}
