# Frustum Culling Implementation

This implementation adds frustum culling capabilities to the Darkmoon Engine, improving rendering performance by only processing objects that are visible within the camera's view frustum.

## Features

### Core Functionality
- **AABB-based culling**: Uses Axis-Aligned Bounding Boxes for accurate visibility testing
- **Sphere-based culling**: Alternative faster but less precise culling method
- **Configurable parameters**: Runtime adjustable settings through GUI
- **Debug logging**: Optional performance statistics logging

### Mathematical Components

#### Aabb (Axis-Aligned Bounding Box)
- Represents object bounds as min/max corners
- Supports transformation to world space
- Methods for intersection and containment testing

#### Frustum
- Extracted from camera's view-projection matrix
- Contains 6 planes (left, right, top, bottom, near, far)
- Supports point, sphere, and AABB visibility testing

## Usage

### Configuration
The frustum culling system can be configured through the GUI or by modifying the `FrustumCullingConfig`:

```rust
pub struct FrustumCullingConfig {
    pub enabled: bool,              // Enable/disable frustum culling
    pub debug_logging: bool,        // Show culling statistics
    pub log_interval_frames: u32,   // How often to log stats
    pub default_object_size: f32,   // Default bounding box size for objects
    pub use_sphere_culling: bool,   // Use sphere instead of AABB culling
}
```

### GUI Controls
Access the frustum culling settings through the "Frustum Culling" section in the debug GUI:

- **Enable frustum culling**: Toggle the entire system on/off
- **Debug logging**: Enable console output of culling statistics
- **Use sphere culling**: Switch between AABB and sphere-based culling
- **Default object size**: Adjust the default bounding volume size
- **Log interval**: Control how frequently statistics are logged

### Performance Impact

#### Benefits
- Reduces number of objects processed by the renderer
- Improves CPU performance in scenes with many objects
- Particularly effective in scenes with objects outside the camera view

#### Overhead
- Additional per-frame calculations for frustum extraction
- Bounding box transformations for visible objects
- Memory overhead for storing bounding boxes

## Implementation Details

### Culling Process
1. Extract view frustum from camera matrices
2. For each scene object:
   - Calculate or retrieve cached bounding box
   - Transform to world space
   - Test against frustum planes
   - Update object visibility state

### Bounding Box Management
- Bounding boxes are cached per scene element
- Default boxes are created for objects without mesh data
- Future improvements could extract actual mesh bounds

### Integration Points
- `update_objects()`: Main culling logic
- `SceneElement`: Extended with bounding box storage
- `PersistedState`: Configuration persistence
- GUI: Runtime configuration interface

## Future Enhancements

### Possible Improvements
1. **Hierarchical culling**: Use spatial data structures (octrees, BVH)
2. **Occlusion culling**: Add visibility testing against scene geometry
3. **Level-of-detail**: Integrate with LOD system based on distance
4. **Multi-threaded culling**: Parallelize visibility tests for large scenes
5. **GPU-based culling**: Move culling computations to compute shaders

### Mesh Integration
Currently uses default bounding boxes. Future versions should:
- Extract actual mesh bounds from vertex data
- Cache bounds in mesh loading pipeline
- Support animated/skinned mesh bounds

## API Reference

### Key Types
- `Frustum`: Camera frustum representation
- `Aabb`: Axis-aligned bounding box
- `IntersectionResult`: Visibility test results (Outside/Intersecting/Inside)
- `FrustumCullingConfig`: Configuration parameters

### Key Methods
- `Frustum::from_view_projection_matrix()`: Create frustum from camera
- `Frustum::is_visible_aabb()`: Test AABB visibility
- `Frustum::is_visible_sphere()`: Test sphere visibility
- `Aabb::transform()`: Transform bounding box to world space

## Performance Notes

### Recommended Settings
- **Small scenes (< 100 objects)**: May not provide significant benefit
- **Medium scenes (100-1000 objects)**: AABB culling recommended
- **Large scenes (1000+ objects)**: Consider sphere culling for better performance
- **Static scenes**: Cache bounding boxes aggressively
- **Dynamic scenes**: Update bounds only when objects move

### Tuning Guidelines
- Start with default settings
- Enable debug logging to monitor effectiveness
- Adjust `default_object_size` based on your scene scale
- Use sphere culling for scenes with many small objects
- Disable if culling overhead exceeds rendering savings
