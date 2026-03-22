use crate::core::mesh::Mesh;
use crate::core::model::Model;
use crate::core::resources::Resources;
use crate::core::transform::Transform;
use crate::core::types::ResourceId;
use crate::io::error::Error;
use crate::io::thumbnail_handle::{ImageFormat, ThumbnailHandle};

use euc::buffer::Buffer2d;
use euc::rasterizer::{BackfaceCullingDisabled, Triangles};
use euc::{Pipeline, Rasterizer};
use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use vek_old::mat::repr_c::column_major::mat4::Mat4;
use vek_old::vec::repr_c::vec3::Vec3;
use vek_old::vec::repr_c::vec4::Vec4;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 256;
const DEFAULT_PADDING: f32 = 0.1; // 10% padding
const DEFAULT_CAMERA_FOV: f32 = 75.0;
const DEFAULT_CAMERA_NEAR: f32 = 0.1;
const DEFAULT_CAMERA_FAR: f32 = 1000.0;

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
            background_color: [255, 0, 0, 255], // Red
            mesh_color: [100, 149, 237, 255],   // Cornflower blue
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
        let (center, size) = self.get_bounding_box_size(&bounding_box);

        // Setup camera with auto-fit
        let camera_matrix = self.setup_camera_matrix(center, size);

        // Create render buffers
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let mut color_buffer = Buffer2d::new([width, height], self.config.background_color);
        let mut depth_buffer = Buffer2d::new([width, height], 1.0f32);

        // Create rendering pipeline
        let pipeline = ThumbnailPipeline {
            mvp_matrix: camera_matrix,
            mesh_color: self.config.mesh_color,
        };

        // Collect all vertices and indices from all meshes
        let mut all_vertices: Vec<[f32; 3]> = Vec::new();
        let mut all_indices: Vec<[usize; 3]> = Vec::new();
        let mut vertex_offset = 0;

        for (mesh, transform) in meshes_with_transforms {
            all_vertices.reserve_exact(mesh.vertices.vertex.len());
            // Add vertices
            for vertex in &mesh.vertices.vertex {
                let pos = Vec4::new(
                    vertex.x.value() as f32,
                    vertex.y.value() as f32,
                    vertex.z.value() as f32,
                    1.0,
                );
                let transformed = transform * pos;
                all_vertices.push([transformed.x, transformed.y, transformed.z]);
            }

            // Add indices (with offset)
            all_indices.reserve_exact(mesh.triangles.triangle.len());
            for triangle in &mesh.triangles.triangle {
                all_indices.push([
                    triangle.v1 as usize + vertex_offset,
                    triangle.v2 as usize + vertex_offset,
                    triangle.v3 as usize + vertex_offset,
                ]);
            }

            vertex_offset += mesh.vertices.vertex.len();
        }

        // Render each triangle using euc
        // Each triangle is 3 consecutive vertices
        let vertices: Vec<[f32; 3]> = all_vertices;

        // euc expects vertices in a flat array where each group of 3 is a triangle
        // We need to expand our indexed triangles into a flat vertex array
        let mut triangle_vertices: Vec<[f32; 3]> = Vec::with_capacity(all_indices.len() * 3);
        for triangle_indices in &all_indices {
            triangle_vertices.push(vertices[triangle_indices[0]]);
            triangle_vertices.push(vertices[triangle_indices[1]]);
            triangle_vertices.push(vertices[triangle_indices[2]]);
        }

        // Use the Triangles rasterizer to draw
        Triangles::<_, BackfaceCullingDisabled>::draw(
            &pipeline,
            &triangle_vertices,
            &mut color_buffer,
            Some(&mut depth_buffer),
        );

        // Encode as PNG
        let png_data = self.encode_png(color_buffer.as_ref())?;

        Ok(ThumbnailHandle {
            data: png_data,
            format: ImageFormat::Png,
        })
    }

    /// Collects all meshes from the model, applying transforms
    fn collect_meshes<'b>(&self, model: &'b Model) -> Result<Vec<(&'b Mesh, Mat4<f32>)>, Error> {
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
    fn collect_object_meshes<'b>(
        &self,
        resources: &'b Resources,
        object_id: ResourceId,
        transform: Mat4<f32>,
        meshes: &mut Vec<(&'b Mesh, Mat4<f32>)>,
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
            meshes.push((mesh, transform));
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

    /// Converts a 3MF Transform to a Mat4
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

    /// Calculates the bounding box of all vertices
    fn calculate_bounding_box(&self, meshes: &[(&Mesh, Mat4<f32>)]) -> (Vec3<f32>, Vec3<f32>) {
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
    fn get_bounding_box_size(&self, bbox: &(Vec3<f32>, Vec3<f32>)) -> (Vec3<f32>, Vec3<f32>) {
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

    /// Sets up the camera matrix with auto-fit to the model
    fn setup_camera_matrix(&self, target: Vec3<f32>, size: Vec3<f32>) -> Mat4<f32> {
        let max_size = size.x.max(size.y).max(size.z);
        let padding_factor = 1.0 + self.config.padding;
        let distance =
            (max_size / 1.75) * padding_factor / (self.config.camera_fov.to_radians() / 2.0).tan();

        // Calculate camera position using spherical coordinates
        let azimuth = self.config.camera_azimuth.to_radians();
        let elevation = self.config.camera_elevation.to_radians();
        let distance = distance.max(1.0);

        let cam_x = target.x + distance * azimuth.cos() * elevation.cos();
        let cam_y = target.y + distance * elevation.sin();
        let cam_z = target.z + distance * azimuth.sin() * elevation.cos();
        let camera_pos = Vec3::new(cam_x, cam_y, cam_z);

        // View matrix
        let view_matrix = Mat4::look_at_rh(camera_pos, target, Vec3::new(0.0, 1.0, 0.0));

        // Projection matrix
        let aspect_ratio = self.config.width as f32 / self.config.height as f32;
        let projection_matrix = Mat4::perspective_rh_zo(
            self.config.camera_fov.to_radians(),
            aspect_ratio,
            DEFAULT_CAMERA_NEAR,
            DEFAULT_CAMERA_FAR,
        );

        // Return MVP matrix
        projection_matrix * view_matrix
    }

    /// Encodes the pixel buffer as a PNG image
    fn encode_png(&self, pixels: &[[u8; 4]]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        let encoder = PngEncoder::new(&mut output);

        // Flatten the pixel array
        let flattened: Vec<u8> = pixels.iter().flat_map(|p| p.iter().copied()).collect();

        encoder
            .write_image(
                &flattened,
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

/// euc Pipeline for rendering thumbnails
struct ThumbnailPipeline {
    mvp_matrix: Mat4<f32>,
    mesh_color: [u8; 4],
}

impl Pipeline for ThumbnailPipeline {
    type Vertex = [f32; 3]; // x, y, z
    type VsOut = (); // No data passed from vertex to fragment
    type Pixel = [u8; 4]; // RGBA

    // Vertex shader: transform vertices to clip space
    fn vert(&self, pos: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let pos_vec = Vec4::new(pos[0], pos[1], pos[2], 1.0);
        let transformed = self.mvp_matrix * pos_vec;
        (
            [transformed.x, transformed.y, transformed.z, transformed.w],
            (),
        )
    }

    // Fragment shader: output flat color
    fn frag(&self, _: &Self::VsOut) -> Self::Pixel {
        self.mesh_color
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::build::Build;
    use crate::core::resources::Resources;
    use crate::io::ThreemfPackage;

    use std::cmp::Ordering;
    use std::fs::File;
    use std::path::PathBuf;

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
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let generator = ThumbnailGenerator::default();
        let meshes = generator.collect_meshes(&package.root).unwrap();

        // 2 torus and 1 pyramid
        assert_eq!(meshes.len(), 3);
    }

    #[test]
    fn test_calculate_bounding_box() {
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let generator = ThumbnailGenerator::default();
        let meshes = generator.collect_meshes(&package.root).unwrap();
        let bbox = generator.calculate_bounding_box(&meshes);

        let (center, size) = generator.get_bounding_box_size(&bbox);

        // Cube from -1 to 1 in all axes, so center should be at origin
        assert_eq!(Vec3::new(0.0, 0.0, 0.0), center);
        assert_eq!(Vec3::new(88.2515, 69.67, 53.5856), size);
    }

    #[test]
    fn test_generate_thumbnail() {
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let generator = ThumbnailGenerator::default();
        let thumbnail = generator.generate(&package.root).unwrap();

        // Decode the generated PNG thumbnail to raw RGB data
        let generated_image =
            image::load_from_memory_with_format(&thumbnail.data, image::ImageFormat::Png)
                .expect("Failed to decode generated PNG");

        // generated_image
        //     .save("tests/data/golden_files/thumbnails/components-object_new.png")
        //     .unwrap();
        let generated_image = nv_flip::FlipImageRgb8::with_data(
            generator.config.width,
            generator.config.height,
            &generated_image.to_rgb8(),
        );

        let ref_image =
            image::open("tests/data/golden_files/thumbnails/components-object.png").unwrap();
        let ref_image = nv_flip::FlipImageRgb8::with_data(
            generator.config.width,
            generator.config.height,
            &ref_image.to_rgb8(),
        );

        let flip_result = nv_flip::flip(ref_image, generated_image, 0.01);
        let pool = nv_flip::FlipPool::from_image(&flip_result);
        if let Some(Ordering::Greater) = pool.mean().partial_cmp(&0.01) {
            println!("Mean error {}", pool.mean());
            panic!("Something is wrong with thumbnail")
        }
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
            requiredextensions: None,
            recommendedextensions: None,
        };

        let generator = ThumbnailGenerator::default();
        let result = generator.generate(&model);

        assert!(result.is_err());
        assert!(matches!(result, Err(Error::ThumbnailError(_))));
    }
}
