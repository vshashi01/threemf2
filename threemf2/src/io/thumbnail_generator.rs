use crate::core::mesh::Mesh;
use crate::core::model::Model;
use crate::core::resources::Resources;
use crate::core::transform::Transform;
use crate::core::types::ResourceId;
use crate::io::error::Error;
use crate::io::thumbnail_handle::{ImageFormat, ThumbnailHandle};

use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use rusterix::batch::batch3d::Batch3D;
use rusterix::batch::{CullMode, PrimitiveMode};
use rusterix::camera::D3Camera;
use rusterix::camera::d3orbit::D3OrbitCamera;
use rusterix::rasterizer::Rasterizer;
use rusterix::scene::Scene;
use rusterix::{Assets, Material, Shader, VGrayGradientShader};
use vek::mat::repr_c::column_major::mat4::Mat4;
use vek::vec::repr_c::vec2::Vec2;
use vek::vec::repr_c::vec3::Vec3;
use vek::vec::repr_c::vec4::Vec4;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 256;
const DEFAULT_PADDING: f32 = 0.1; // 10% padding
const DEFAULT_CAMERA_FOV: f32 = 75.0;
const DEFAULT_CAMERA_NEAR: f32 = 0.1;
const DEFAULT_CAMERA_FAR: f32 = 1000.0;
const DEFAULT_CAMERA_AZIMUTH: f32 = 45.0_f32.to_radians();
const DEFAULT_CAMERA_ELEVATION: f32 = 30.0_f32.to_radians();
const DEFAULT_TILE_SIZE: usize = 64;

/// Configuration for thumbnail generation
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailConfig {
    /// Width of the thumbnail in pixels
    pub width: u32,
    /// Height of the thumbnail in pixels
    pub height: u32,
    /// Padding around the model as a fraction of the model size (0.0 to 1.0)
    pub padding: f32,
    /// Background color as RGBA
    pub background_color: [u8; 4],
    /// Mesh color as RGBA (used for flat shading)
    pub mesh_color: [u8; 4],
    /// Camera azimuth angle in degrees (horizontal rotation)
    pub camera_azimuth: f32,
    /// Camera elevation angle in degrees (vertical rotation)
    pub camera_elevation: f32,
    /// Camera field of view in degrees
    pub camera_fov: f32,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            padding: DEFAULT_PADDING,
            background_color: [240, 240, 240, 255], // Light gray
            mesh_color: [100, 149, 237, 255],       // Cornflower blue
            camera_azimuth: 45.0,
            camera_elevation: 30.0,
            camera_fov: DEFAULT_CAMERA_FOV,
        }
    }
}

impl ThumbnailConfig {
    /// Creates a new thumbnail configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the thumbnail dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the padding around the model
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding.clamp(0.0, 0.5);
        self
    }

    /// Sets the background color
    pub fn with_background_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.background_color = [r, g, b, a];
        self
    }

    /// Sets the mesh color
    pub fn with_mesh_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.mesh_color = [r, g, b, a];
        self
    }

    /// Sets the camera angles
    pub fn with_camera_angles(mut self, azimuth: f32, elevation: f32) -> Self {
        self.camera_azimuth = azimuth;
        self.camera_elevation = elevation;
        self
    }

    /// Sets the camera field of view
    pub fn with_camera_fov(mut self, fov: f32) -> Self {
        self.camera_fov = fov;
        self
    }
}

/// A generator for creating thumbnails from 3MF models
#[derive(Debug)]
pub struct ThumbnailGenerator {
    config: ThumbnailConfig,
}

impl ThumbnailGenerator {
    /// Creates a new thumbnail generator with the given configuration
    pub fn new(config: ThumbnailConfig) -> Self {
        Self { config }
    }

    /// Creates a new thumbnail generator with default configuration
    pub fn default() -> Self {
        Self::new(ThumbnailConfig::default())
    }

    /// Generates a thumbnail from a 3MF model
    ///
    /// # Arguments
    /// * `model` - The 3MF model to render
    ///
    /// # Returns
    /// A `ThumbnailHandle` containing the PNG-encoded thumbnail image
    pub fn generate(&self, model: &Model) -> Result<ThumbnailHandle, Error> {
        // Collect all meshes with their transforms from the model
        let meshes_with_transforms = self.collect_meshes(model)?;

        if meshes_with_transforms.is_empty() {
            return Err(Error::ThumbnailError(
                "Model contains no renderable geometry".to_string(),
            ));
        }

        // Calculate bounding box of all transformed vertices
        let bounding_box = self.calculate_bounding_box(&meshes_with_transforms);
        let (center, size) = self.get_bounding_box_info(&bounding_box);

        // Create rusterix batches from the meshes
        let batches = self.create_batches(&meshes_with_transforms)?;

        // Setup camera with auto-fit
        let camera = self.setup_camera(center, size);

        // Create scene
        let mut scene =
            Scene::from_static(vec![], batches).background(Box::new(VGrayGradientShader::new()));

        // Render the scene
        let mut pixels = vec![0u8; (self.config.width * self.config.height * 4) as usize];
        let view_matrix = camera.view_matrix();
        let projection_matrix =
            camera.projection_matrix(self.config.width as f32, self.config.height as f32);

        let mut rasterizer = Rasterizer::setup(None, view_matrix, projection_matrix);
        rasterizer.rasterize(
            &mut scene,
            &mut pixels,
            self.config.width as usize,
            self.config.height as usize,
            DEFAULT_TILE_SIZE,
            &Assets::default(),
        );

        // Encode as PNG
        let png_data = self.encode_png(&pixels)?;

        Ok(ThumbnailHandle {
            data: png_data,
            format: ImageFormat::Png,
        })
    }

    /// Collects all meshes from the model, applying transforms
    fn collect_meshes(&self, model: &Model) -> Result<Vec<(Mesh, Mat4<f32>)>, Error> {
        let mut meshes = Vec::new();

        // Process each build item
        for item in &model.build.item {
            let object_id = item.objectid;
            let base_transform = item
                .transform
                .as_ref()
                .map(|t| self.transform_to_mat4(t))
                .unwrap_or_else(Mat4::identity);

            self.collect_object_meshes(&model.resources, object_id, base_transform, &mut meshes)?;
        }

        Ok(meshes)
    }

    /// Recursively collects meshes from an object and its components
    fn collect_object_meshes(
        &self,
        resources: &Resources,
        object_id: ResourceId,
        transform: Mat4<f32>,
        meshes: &mut Vec<(Mesh, Mat4<f32>)>,
    ) -> Result<(), Error> {
        let object = resources
            .object
            .iter()
            .find(|o| o.id == object_id)
            .ok_or_else(|| {
                Error::ResourceNotFound(format!("Object with ID {} not found", object_id))
            })?;

        // If the object has a mesh, add it
        if let Some(mesh) = &object.mesh {
            meshes.push((mesh.clone(), transform));
        }

        // If the object has components, recursively process them
        if let Some(components) = &object.components {
            for component in &components.component {
                let component_transform = component
                    .transform
                    .as_ref()
                    .map(|t| self.transform_to_mat4(t))
                    .unwrap_or_else(Mat4::identity);
                let combined_transform = transform * component_transform;

                self.collect_object_meshes(
                    resources,
                    component.objectid,
                    combined_transform,
                    meshes,
                )?;
            }
        }

        Ok(())
    }

    /// Converts a 3MF Transform to a rusterix Mat4
    fn transform_to_mat4(&self, transform: &Transform) -> Mat4<f32> {
        let m = &transform.0;
        Mat4::new(
            m[0] as f32,
            m[1] as f32,
            m[2] as f32,
            0.0,
            m[3] as f32,
            m[4] as f32,
            m[5] as f32,
            0.0,
            m[6] as f32,
            m[7] as f32,
            m[8] as f32,
            0.0,
            m[9] as f32,
            m[10] as f32,
            m[11] as f32,
            1.0,
        )
    }

    /// Creates rusterix batches from meshes
    fn create_batches(
        &self,
        meshes_with_transforms: &[(Mesh, Mat4<f32>)],
    ) -> Result<Vec<Batch3D>, Error> {
        let mut all_batches = Vec::new();

        for (mesh, transform) in meshes_with_transforms {
            let batch = self.mesh_to_batch(mesh, *transform)?;
            all_batches.push(batch);
        }

        Ok(all_batches)
    }

    /// Converts a single mesh to a rusterix batch
    fn mesh_to_batch(&self, mesh: &Mesh, transform: Mat4<f32>) -> Result<Batch3D, Error> {
        // Convert vertices to [f32; 4] and apply transform
        let vertices: Vec<[f32; 4]> = mesh
            .vertices
            .vertex
            .iter()
            .map(|v| {
                let pos = Vec4::new(
                    v.x.value() as f32,
                    v.y.value() as f32,
                    v.z.value() as f32,
                    1.0,
                );
                let transformed = transform * pos;
                [transformed.x, transformed.y, transformed.z, transformed.w]
            })
            .collect();

        // Create triangle indices
        let indices: Vec<(usize, usize, usize)> = mesh
            .triangles
            .triangle
            .iter()
            .map(|t| (t.v1 as usize, t.v2 as usize, t.v3 as usize))
            .collect();

        // Create UVs (simple default UVs since we don't have texture info)
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; vertices.len()];

        // Create the batch
        let batch = Batch3D::new(vertices, indices, uvs)
            .mode(PrimitiveMode::Triangles)
            .cull_mode(CullMode::Off)
            .material(Material::new(
                rusterix::MaterialRole::Metallic,
                rusterix::MaterialModifier::None,
                0.6,
                0.0,
            ))
            .with_computed_normals();

        Ok(batch)
    }

    /// Calculates the bounding box of all vertices
    fn calculate_bounding_box(&self, meshes: &[(Mesh, Mat4<f32>)]) -> (Vec3<f32>, Vec3<f32>) {
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for (mesh, transform) in meshes {
            for vertex in &mesh.vertices.vertex {
                let pos = Vec4::new(
                    vertex.x.value() as f32,
                    vertex.y.value() as f32,
                    vertex.z.value() as f32,
                    1.0,
                );
                let transformed = *transform * pos;

                min.x = min.x.min(transformed.x);
                min.y = min.y.min(transformed.y);
                min.z = min.z.min(transformed.z);

                max.x = max.x.max(transformed.x);
                max.y = max.y.max(transformed.y);
                max.z = max.z.max(transformed.z);
            }
        }

        (min, max)
    }

    /// Gets the center and size from a bounding box
    fn get_bounding_box_info(&self, bbox: &(Vec3<f32>, Vec3<f32>)) -> (Vec3<f32>, Vec3<f32>) {
        let min = bbox.0;
        let max = bbox.1;

        let center = Vec3::new(
            (min.x + max.x) / 2.0,
            (min.y + max.y) / 2.0,
            (min.z + max.z) / 2.0,
        );

        let size = Vec3::new(max.x - min.x, max.y - min.y, max.z - min.z);

        (center, size)
    }

    /// Sets up the camera with auto-fit to the model
    fn setup_camera(&self, target: Vec3<f32>, size: Vec3<f32>) -> D3OrbitCamera {
        let max_size = size.x.max(size.y).max(size.z);
        let padding_factor = 1.0 + self.config.padding;
        let distance =
            max_size * padding_factor / (self.config.camera_fov.to_radians() / 2.0).tan();

        D3OrbitCamera {
            center: target,
            distance: distance.max(1.0), // Ensure minimum distance
            azimuth: self.config.camera_azimuth.to_radians(),
            elevation: self.config.camera_elevation.to_radians(),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: self.config.camera_fov,
            near: DEFAULT_CAMERA_NEAR,
            far: DEFAULT_CAMERA_FAR,
        }
    }

    /// Encodes the pixel buffer as a PNG image
    fn encode_png(&self, pixels: &[u8]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        let encoder = PngEncoder::new(&mut output);

        encoder
            .write_image(
                pixels,
                self.config.width,
                self.config.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| Error::ThumbnailError(format!("PNG encoding failed: {}", e)))?;

        Ok(output)
    }
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self::new(ThumbnailConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use super::*;
    use crate::core::build::{Build, Item};
    use crate::core::mesh::{Mesh, Triangle, Triangles, Vertex, Vertices};
    use crate::core::object::Object;
    use crate::core::resources::Resources;
    use crate::core::types::{OptionalResourceId, OptionalResourceIndex};
    use crate::io::ThreemfPackage;

    fn create_simple_cube_model() -> Model {
        let vertices = Vertices {
            vertex: vec![
                Vertex::new(-1.0, -1.0, -1.0), // 0
                Vertex::new(1.0, -1.0, -1.0),  // 1
                Vertex::new(1.0, 1.0, -1.0),   // 2
                Vertex::new(-1.0, 1.0, -1.0),  // 3
                Vertex::new(-1.0, -1.0, 1.0),  // 4
                Vertex::new(1.0, -1.0, 1.0),   // 5
                Vertex::new(1.0, 1.0, 1.0),    // 6
                Vertex::new(-1.0, 1.0, 1.0),   // 7
            ],
        };

        let triangles = Triangles {
            triangle: vec![
                // Front face
                Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 0,
                    v2: 2,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                // Back face
                Triangle {
                    v1: 5,
                    v2: 4,
                    v3: 7,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 5,
                    v2: 7,
                    v3: 6,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        };

        let mesh = Mesh {
            vertices,
            triangles,
            trianglesets: None,
            beamlattice: None,
        };

        let object = Object {
            id: 1,
            mesh: Some(mesh),
            components: None,
            name: Some("Cube".to_string()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            thumbnail: None,
            partnumber: None,
            uuid: None,
            objecttype: None,
        };

        let resources = Resources {
            object: vec![object],
            basematerials: vec![],
        };

        let build = Build {
            uuid: None,
            item: vec![Item {
                objectid: 1,
                transform: None,
                partnumber: None,
                uuid: None,
                path: None,
            }],
        };

        Model {
            unit: None,
            metadata: vec![],
            resources,
            build,
            recommendedextensions: None,
            requiredextensions: None,
        }
    }

    #[test]
    fn test_thumbnail_config_default() {
        let config = ThumbnailConfig::default();
        assert_eq!(config.width, DEFAULT_WIDTH);
        assert_eq!(config.height, DEFAULT_HEIGHT);
        assert_eq!(config.padding, DEFAULT_PADDING);
    }

    #[test]
    fn test_thumbnail_config_builder() {
        let config = ThumbnailConfig::new()
            .with_dimensions(512, 512)
            .with_padding(0.2)
            .with_background_color(255, 0, 0, 255);

        assert_eq!(config.width, 512);
        assert_eq!(config.height, 512);
        assert_eq!(config.padding, 0.2);
        assert_eq!(config.background_color, [255, 0, 0, 255]);
    }

    #[test]
    fn test_collect_meshes() {
        let model = create_simple_cube_model();
        let generator = ThumbnailGenerator::default();
        let meshes = generator.collect_meshes(&model).unwrap();

        assert_eq!(meshes.len(), 1);
    }

    #[test]
    fn test_calculate_bounding_box() {
        let model = create_simple_cube_model();
        let generator = ThumbnailGenerator::default();
        let meshes = generator.collect_meshes(&model).unwrap();
        let bbox = generator.calculate_bounding_box(&meshes);

        let (center, size) = generator.get_bounding_box_info(&bbox);

        // Cube from -1 to 1 in all axes, so center should be at origin
        assert!((center.x).abs() < 0.01);
        assert!((center.y).abs() < 0.01);
        assert!((center.z).abs() < 0.01);

        // Size should be approximately 2 in each dimension
        assert!((size.x - 2.0).abs() < 0.01);
        assert!((size.y - 2.0).abs() < 0.01);
        assert!((size.z - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_thumbnail() {
        // let model = create_simple_cube_model();
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let generator = ThumbnailGenerator::default();
        let thumbnail = generator.generate(&package.root).unwrap();

        let mut file = File::create("output.png").unwrap();
        file.write_all(&thumbnail.data).unwrap();

        assert_eq!(thumbnail.format, ImageFormat::Png);
        assert!(!thumbnail.data.is_empty());

        // Verify it's a valid PNG by checking the PNG magic bytes
        assert_eq!(&thumbnail.data[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_empty_model_error() {
        let model = Model {
            unit: None,
            metadata: vec![],
            resources: Resources {
                object: vec![],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![],
            },
            recommendedextensions: None,
            requiredextensions: None,
        };

        let generator = ThumbnailGenerator::default();
        let result = generator.generate(&model);

        assert!(result.is_err());
        assert!(matches!(result, Err(Error::ThumbnailError(_))));
    }
}
