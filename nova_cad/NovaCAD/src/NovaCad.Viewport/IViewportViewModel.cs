using System;
using System.Collections.ObjectModel;
using NovaCad.Core.Models;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Event args for when a new visual object is created
    /// </summary>
    public class VisualObjectCreatedEventArgs : EventArgs
    {
        public string Name { get; }
        public Mesh Mesh { get; }
        public object? BodyRef { get; }

        public VisualObjectCreatedEventArgs(string name, Mesh mesh, object? bodyRef = null)
        {
            Name = name;
            Mesh = mesh;
            BodyRef = bodyRef;
        }
    }

    /// <summary>
    /// Represents a visual object in the viewport
    /// </summary>
    public interface IVisualObject
    {
        string Name { get; }
        Mesh Mesh { get; }
        object? BodyRef { get; }
        bool IsVisible { get; set; }
        bool IsSelected { get; set; }
    }

    /// <summary>
    /// Viewport view model interface
    /// </summary>
    public interface IViewportViewModel
    {
        RenderMode RenderMode { get; }
        bool ShowGrid { get; }
        bool ShowAxes { get; }
        ObservableCollection<IVisualObject> VisualObjects { get; }

        event EventHandler<VisualObjectCreatedEventArgs>? VisualObjectCreated;
        event EventHandler? VisualObjectsCleared;
        event EventHandler<RenderMode>? RenderModeChanged;
        event EventHandler<string>? ViewPresetChanged;
        event EventHandler? FitAllRequested;
    }


}
