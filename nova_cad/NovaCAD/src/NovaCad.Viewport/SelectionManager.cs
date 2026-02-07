using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using Silk.NET.OpenGL;
using NovaCad.Core.Models;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Manages 3D selection state and highlighting
    /// </summary>
    public class SelectionManager
    {
        private HashSet<uint> _selectedIds = new();
        private HashSet<uint> _highlightedIds = new();
        private uint? _primarySelection;
        
        // Selection appearance
        public Color SelectionColor { get; set; } = new Color(1.0f, 0.5f, 0.0f, 1.0f);
        public Color HighlightColor { get; set; } = new Color(1.0f, 1.0f, 0.0f, 0.5f);
        public Color PreselectionColor { get; set; } = new Color(0.0f, 1.0f, 1.0f, 0.5f);
        
        // Selection mode
        public SelectionMode Mode { get; set; } = SelectionMode.Single;
        public SelectionFilter Filter { get; set; } = SelectionFilter.All;
        
        // Events
        public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;
        public event EventHandler<uint>? EntitySelected;
        public event EventHandler<uint>? EntityDeselected;

        /// <summary>
        /// Get selected entity IDs
        /// </summary>
        public IReadOnlyCollection<uint> SelectedIds => _selectedIds;

        /// <summary>
        /// Get primary selection
        /// </summary>
        public uint? PrimarySelection => _primarySelection;

        /// <summary>
        /// Check if entity is selected
        /// </summary>
        public bool IsSelected(uint entityId) => _selectedIds.Contains(entityId);

        /// <summary>
        /// Select a single entity
        /// </summary>
        public void Select(uint entityId, bool clearOthers = true)
        {
            if (clearOthers)
            {
                ClearSelection();
            }
            
            if (_selectedIds.Add(entityId))
            {
                _primarySelection = entityId;
                EntitySelected?.Invoke(this, entityId);
                OnSelectionChanged();
            }
        }

        /// <summary>
        /// Select multiple entities
        /// </summary>
        public void SelectRange(IEnumerable<uint> entityIds)
        {
            bool changed = false;
            
            foreach (var id in entityIds)
            {
                if (_selectedIds.Add(id))
                {
                    changed = true;
                    EntitySelected?.Invoke(this, id);
                }
            }
            
            if (changed)
            {
                _primarySelection = _selectedIds.FirstOrDefault();
                OnSelectionChanged();
            }
        }

        /// <summary>
        /// Toggle selection of an entity
        /// </summary>
        public void ToggleSelection(uint entityId)
        {
            if (_selectedIds.Contains(entityId))
            {
                Deselect(entityId);
            }
            else
            {
                Select(entityId, false);
            }
        }

        /// <summary>
        /// Deselect an entity
        /// </summary>
        public void Deselect(uint entityId)
        {
            if (_selectedIds.Remove(entityId))
            {
                EntityDeselected?.Invoke(this, entityId);
                
                if (_primarySelection == entityId)
                {
                    _primarySelection = _selectedIds.FirstOrDefault();
                }
                
                OnSelectionChanged();
            }
        }

        /// <summary>
        /// Clear all selection
        /// </summary>
        public void ClearSelection()
        {
            if (_selectedIds.Count == 0) return;
            
            var oldSelection = _selectedIds.ToList();
            _selectedIds.Clear();
            _primarySelection = null;
            
            foreach (var id in oldSelection)
            {
                EntityDeselected?.Invoke(this, id);
            }
            
            OnSelectionChanged();
        }

        /// <summary>
        /// Invert selection
        /// </summary>
        public void InvertSelection(IEnumerable<uint> allEntities)
        {
            var all = allEntities.ToList();
            var newSelection = all.Where(id => !_selectedIds.Contains(id)).ToList();
            
            ClearSelection();
            SelectRange(newSelection);
        }

        /// <summary>
        /// Set preselection (hover)
        /// </summary>
        public void SetPreselection(uint? entityId)
        {
            _highlightedIds.Clear();
            if (entityId.HasValue)
            {
                _highlightedIds.Add(entityId.Value);
            }
        }

        /// <summary>
        /// Get selection color for an entity
        /// </summary>
        public Color GetEntityColor(uint entityId, Color originalColor)
        {
            if (_selectedIds.Contains(entityId))
            {
                return BlendColors(originalColor, SelectionColor, 0.7f);
            }
            else if (_highlightedIds.Contains(entityId))
            {
                return BlendColors(originalColor, PreselectionColor, 0.5f);
            }
            
            return originalColor;
        }

        /// <summary>
        /// Apply selection state to meshes
        /// </summary>
        public void ApplyToMeshes(IEnumerable<Mesh> meshes)
        {
            foreach (var mesh in meshes)
            {
                mesh.IsSelected = _selectedIds.Contains(mesh.EntityId);
            }
        }

        /// <summary>
        /// Get bounding box of selection
        /// </summary>
        public BoundingBox? GetSelectionBounds(IEnumerable<Mesh> meshes)
        {
            var selectedMeshes = meshes.Where(m => _selectedIds.Contains(m.EntityId)).ToList();
            
            if (selectedMeshes.Count == 0) return null;
            
            var bbox = selectedMeshes[0].BoundingBox;
            foreach (var mesh in selectedMeshes.Skip(1))
            {
                bbox.Expand(mesh.BoundingBox);
            }
            
            return bbox;
        }

        /// <summary>
        /// Select all visible entities
        /// </summary>
        public void SelectAll(IEnumerable<uint> visibleEntities)
        {
            SelectRange(visibleEntities);
        }

        /// <summary>
        /// Delete selected entities
        /// </summary>
        public IReadOnlyCollection<uint> DeleteSelection()
        {
            var deleted = _selectedIds.ToList();
            ClearSelection();
            return deleted;
        }

        /// <summary>
        /// Hide selected entities
        /// </summary>
        public void HideSelection(IEnumerable<Mesh> meshes)
        {
            foreach (var mesh in meshes)
            {
                if (_selectedIds.Contains(mesh.EntityId))
                {
                    mesh.Visible = false;
                }
            }
        }

        /// <summary>
        /// Isolate selected entities (hide others)
        /// </summary>
        public void IsolateSelection(IEnumerable<Mesh> meshes)
        {
            foreach (var mesh in meshes)
            {
                mesh.Visible = _selectedIds.Contains(mesh.EntityId);
            }
        }

        /// <summary>
        /// Show all entities
        /// </summary>
        public void ShowAll(IEnumerable<Mesh> meshes)
        {
            foreach (var mesh in meshes)
            {
                mesh.Visible = true;
            }
        }

        private void OnSelectionChanged()
        {
            SelectionChanged?.Invoke(this, new SelectionChangedEventArgs
            {
                SelectedIds = _selectedIds.ToList(),
                PrimarySelection = _primarySelection,
                Count = _selectedIds.Count
            });
        }

        private Color BlendColors(Color c1, Color c2, float t)
        {
            return new Color(
                c1.R * (1 - t) + c2.R * t,
                c1.G * (1 - t) + c2.G * t,
                c1.B * (1 - t) + c2.B * t,
                c1.A * (1 - t) + c2.A * t
            );
        }
    }

    public enum SelectionMode
    {
        Single,
        Add,
        Remove,
        Toggle
    }

    public enum SelectionFilter
    {
        All,
        Bodies,
        Faces,
        Edges,
        Vertices
    }

    public class SelectionChangedEventArgs : EventArgs
    {
        public IReadOnlyCollection<uint> SelectedIds { get; set; }
        public uint? PrimarySelection { get; set; }
        public int Count { get; set; }
    }
}
