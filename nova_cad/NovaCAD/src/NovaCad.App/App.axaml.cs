using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using NovaCad.App.ViewModels;
using NovaCad.App.Views;
using Serilog;
using System;

namespace NovaCad.App
{
    public partial class App : Application
    {
        public static IServiceProvider Services { get; private set; } = null!;

        public override void Initialize()
        {
            AvaloniaXamlLoader.Load(this);
        }

        public override void OnFrameworkInitializationCompleted()
        {
            // Configure services
            var serviceCollection = new ServiceCollection();
            ConfigureServices(serviceCollection);
            Services = serviceCollection.BuildServiceProvider();

            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                desktop.MainWindow = new MainWindow
                {
                    DataContext = Services.GetRequiredService<MainWindowViewModel>()
                };

                desktop.Startup += OnStartup;
                desktop.Exit += OnExit;
            }

            base.OnFrameworkInitializationCompleted();
        }

        private void ConfigureServices(IServiceCollection services)
        {
            // Logging
            services.AddLogging(builder =>
            {
                builder.AddSerilog();
            });

            // ViewModels
            services.AddSingleton<MainWindowViewModel>();
            services.AddTransient<ViewportViewModel>();
            services.AddTransient<ModelTreeViewModel>();
            services.AddTransient<PropertyPanelViewModel>();
            services.AddTransient<RibbonViewModel>();
        }

        private void OnStartup(object? sender, ControlledApplicationLifetimeStartupEventArgs e)
        {
            var logger = Services.GetRequiredService<ILogger<App>>();
            logger.LogInformation("Nova CAD application started");

            // Initialize the Nova kernel
            try
            {
                var result = Kernel.NovaKernel.nova_init();
                if (result != Kernel.NovaKernel.NovaResult.Success)
                {
                    logger.LogError("Failed to initialize Nova kernel: {Result}", result);
                }
                else
                {
                    var version = Kernel.NovaKernel.GetVersion();
                    logger.LogInformation("Nova kernel initialized. Version: {Version}", version);
                }
            }
            catch (Exception ex)
            {
                logger.LogError(ex, "Exception during Nova kernel initialization");
            }
        }

        private void OnExit(object? sender, ControlledApplicationLifetimeExitEventArgs e)
        {
            var logger = Services.GetRequiredService<ILogger<App>>();
            logger.LogInformation("Nova CAD application exiting");

            // Shutdown the Nova kernel
            try
            {
                Kernel.NovaKernel.nova_shutdown();
                logger.LogInformation("Nova kernel shut down successfully");
            }
            catch (Exception ex)
            {
                logger.LogError(ex, "Exception during Nova kernel shutdown");
            }
        }
    }
}
