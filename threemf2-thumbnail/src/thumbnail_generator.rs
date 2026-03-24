use threemf2::core::mesh::Mesh;
use threemf2::core::model::Model;
use threemf2::core::resources::Resources;
use threemf2::core::transform::Transform;
use threemf2::core::types::ResourceId;
use threemf2::io::Error;
use threemf2::io::thumbnail_handle::{ImageFormat, ThumbnailHandle};

use crate::bbox::BoundingBox;
use crate::camera::OrthographicCamera;
use crate::euc::buffer::Buffer2d;
use crate::euc::pipeline::Pipeline;
use crate::mesh_pipeline::{ColoredMesh, Rgba, VertexIn, WireframeMesh};

use image::ImageEncoder;
use image::codecs::png::PngEncoder;

const DEFAULT_WIDTH: u32 = 256;
const DEFAULT_HEIGHT: u32 = 256;
const DEFAULT_PADDING: f32 = 0.001; // 1% padding

/// Configuration for thumbnail generation
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailConfig {
    /// Width of the thumbnail in pixels
    pub width: u32,
    /// Height of the thumbnail in pixels
    pub height: u32,
    /// Padding around the model as a fraction of the model size (0.0 to 0.1)
    pub padding: f32,
    /// Background color as RGBA
    pub background_color: [u8; 4],
    /// Mesh color as RGBA (used for flat shading)
    pub mesh_color: [u8; 4],
    /// Camera yaw angle in degrees (about the Z axis)
    pub yaw_angle: f32,
    /// Camera patch angle in degrees (about the XY plane)
    pub pitch_angle: f32,
    /// Aspect ratio of the camera
    pub aspect_ratio: f32,
    /// Draws the wireframe of the mesh
    pub enable_wireframe: bool,
    /// Draws the surface of the mesh
    pub enable_surface: bool,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            padding: DEFAULT_PADDING,
            background_color: [255, 0, 0, 255], // Red
            mesh_color: [100, 149, 237, 255],   // Cornflower blue
            yaw_angle: 0.0,
            pitch_angle: 0.0,
            aspect_ratio: 1.0,
            enable_wireframe: false,
            enable_surface: true,
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
    pub fn with_camera_angles(mut self, yaw_deg: f32, pitch_deg: f32) -> Self {
        self.yaw_angle = yaw_deg;
        self.pitch_angle = pitch_deg;
        self
    }

    /// Sets the aspect ratio of the thumbnail
    pub fn with_aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = ratio;
        self
    }

    /// Enable or disable the wireframe rendering for supported entities
    pub fn with_wireframe(mut self, enable: bool) -> Self {
        self.enable_wireframe = enable;
        self
    }

    /// Enable or disable Filled rendering for supported entities
    pub fn with_surface(mut self, enable: bool) -> Self {
        self.enable_surface = enable;
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
        let mut camera = OrthographicCamera::looking_at(center)
            .with_angles(self.config.yaw_angle, self.config.pitch_angle)
            .with_aspect_ratio(self.config.aspect_ratio);

        camera.fit_to_bounds(size, self.config.padding);
        let camera_matrix = camera.view_projection_matrix();

        // Create render buffers
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let mut color_buffer = Buffer2d::fill([width, height], Rgba(self.config.background_color));
        let mut depth_buffer = Buffer2d::fill([width, height], 1.0);

        // Collect all vertices and indices from all meshes
        let mut all_vertices: Vec<[f32; 3]> = Vec::new();
        let mut all_indices: Vec<[usize; 3]> = Vec::new();
        let mut vertex_offset = 0;

        for (mesh, transform) in meshes_with_transforms {
            all_vertices.reserve_exact(mesh.vertices.vertex.len());
            // Add vertices
            for vertex in &mesh.vertices.vertex {
                let pos = glam::Vec3::new(
                    vertex.x.value() as f32,
                    vertex.y.value() as f32,
                    vertex.z.value() as f32,
                );

                let transformed = transform.transform_point3(pos);
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
        let mut triangle_vertices: Vec<VertexIn> = Vec::with_capacity(all_indices.len() * 3);
        for triangle_indices in &all_indices {
            let p0 = glam::Vec3::from_array(vertices[triangle_indices[0]]);
            let p1 = glam::Vec3::from_array(vertices[triangle_indices[1]]);
            let p2 = glam::Vec3::from_array(vertices[triangle_indices[2]]);

            //all threemf Mesh is expected to have CCW winding order (Right hand rule)
            let normal = (p1 - p0).cross(p2 - p0).normalize();

            triangle_vertices.push(VertexIn {
                pos: p0,
                normal: normal,
            });
            triangle_vertices.push(VertexIn { pos: p1, normal });
            triangle_vertices.push(VertexIn { pos: p2, normal });
        }

        if self.config.enable_surface {
            ColoredMesh {
                mesh_color: Rgba(self.config.mesh_color),
                model_matrix: camera_matrix,
                light_matrix: glam::Mat4::IDENTITY,
                camera_pos: camera.position(),
                light_pos: camera.position(),
            }
            .render(&triangle_vertices, &mut color_buffer, &mut depth_buffer);
        }

        if self.config.enable_wireframe {
            WireframeMesh {
                wireframe_color: Rgba([0, 0, 0, 255]),
                mvp_matrix: camera_matrix,
            }
            .render(&triangle_vertices, &mut color_buffer, &mut depth_buffer);
        }

        // Encode as PNG
        let png_data = self.encode_png(color_buffer.raw())?;

        Ok(ThumbnailHandle {
            data: png_data,
            format: ImageFormat::Png,
        })
    }

    /// Collects all meshes from the model, applying transforms
    fn collect_meshes<'b>(&self, model: &'b Model) -> Result<Vec<(&'b Mesh, glam::Mat4)>, Error> {
        let mut meshes = Vec::new();

        // Process each build item
        for item in &model.build.item {
            let object_id = item.objectid;
            let base_transform = item
                .transform
                .as_ref()
                .map(|t| self.transform_to_mat4(t))
                .unwrap_or_else(|| glam::Mat4::IDENTITY);

            self.collect_object_meshes(&model.resources, object_id, base_transform, &mut meshes)?;
        }

        Ok(meshes)
    }

    /// Recursively collects meshes from an object and its components
    fn collect_object_meshes<'b>(
        &self,
        resources: &'b Resources,
        object_id: ResourceId,
        transform: glam::Mat4,
        meshes: &mut Vec<(&'b Mesh, glam::Mat4)>,
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
                    .unwrap_or_else(|| glam::Mat4::IDENTITY);
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
    fn transform_to_mat4(&self, transform: &Transform) -> glam::Mat4 {
        glam::Mat4::from_cols_array_2d(&[
            [
                transform.0[0] as f32,
                transform.0[1] as f32,
                transform.0[2] as f32,
                0.0,
            ],
            [
                transform.0[3] as f32,
                transform.0[4] as f32,
                transform.0[5] as f32,
                0.0,
            ],
            [
                transform.0[6] as f32,
                transform.0[7] as f32,
                transform.0[8] as f32,
                0.0,
            ],
            [
                transform.0[9] as f32,
                transform.0[10] as f32,
                transform.0[11] as f32,
                1.0,
            ],
        ])
    }

    /// Calculates the bounding box of all vertices
    fn calculate_bounding_box(&self, meshes: &[(&Mesh, glam::Mat4)]) -> BoundingBox {
        let mut total_bbox = BoundingBox::default();

        for (mesh, transform) in meshes {
            for vertex in &mesh.vertices.vertex {
                let pos = glam::Vec3::new(
                    vertex.x.value() as f32,
                    vertex.y.value() as f32,
                    vertex.z.value() as f32,
                );

                let transformed = transform.transform_point3(pos);
                total_bbox.expand_to_include(&transformed);
            }
        }

        total_bbox
    }

    /// Gets the center and size from a bounding box
    fn get_bounding_box_size(&self, bbox: &BoundingBox) -> (glam::Vec3, glam::Vec3) {
        (bbox.center(), bbox.delta())
    }

    /// Encodes the pixel buffer as a PNG image
    fn encode_png(&self, pixels: &[Rgba]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        let encoder = PngEncoder::new(&mut output);

        // Flatten the pixel array
        let flattened: Vec<u8> = pixels.iter().flat_map(|p| p.0.iter().copied()).collect();

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use threemf2::core::build::Build;
    use threemf2::core::resources::Resources;
    use threemf2::io::ThreemfPackage;

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
        let path = PathBuf::from("../threemf2/tests/data/mesh-composedpart.3mf");
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
        let path = PathBuf::from("../threemf2/tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let generator = ThumbnailGenerator::default();
        let meshes = generator.collect_meshes(&package.root).unwrap();
        let bbox = generator.calculate_bounding_box(&meshes);

        let (center, size) = generator.get_bounding_box_size(&bbox);

        assert_eq!(glam::Vec3::new(125.000046, 133.29489, 36.7928), center);
        assert_eq!(glam::Vec3::new(88.251495, 69.66521, 53.5856), size);
    }

    #[test]
    fn test_generate_thumbnail_surface_only() {
        let path = PathBuf::from("../threemf2/tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let config = ThumbnailConfig::default()
            .with_wireframe(false)
            .with_surface(true)
            .with_camera_angles(-135.0, 30.0);
        let generator = ThumbnailGenerator::new(config);
        let thumbnail = generator.generate(&package.root).unwrap();

        // Decode the generated PNG thumbnail to raw RGB data
        let generated_image =
            image::load_from_memory_with_format(&thumbnail.data, image::ImageFormat::Png)
                .expect("Failed to decode generated PNG");

        generated_image
            .save("tests/data/golden_files/components-object_new.png")
            .unwrap();
        let generated_image = nv_flip::FlipImageRgb8::with_data(
            generator.config.width,
            generator.config.height,
            &generated_image.to_rgb8(),
        );

        let ref_image = image::open("tests/data/golden_files/components-object.png").unwrap();
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
    fn test_generate_thumbnail_wireframe_only() {
        let path = PathBuf::from("../threemf2/tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false).unwrap();
        let config = ThumbnailConfig::default()
            .with_wireframe(true)
            .with_surface(false)
            .with_camera_angles(-135.0, 30.0);
        let generator = ThumbnailGenerator::new(config);
        let thumbnail = generator.generate(&package.root).unwrap();

        // Decode the generated PNG thumbnail to raw RGB data
        let generated_image =
            image::load_from_memory_with_format(&thumbnail.data, image::ImageFormat::Png)
                .expect("Failed to decode generated PNG");

        // generated_image
        //     .save("tests/data/golden_files/components-object_new_wireframe_only.png")
        //     .unwrap();
        let generated_image = nv_flip::FlipImageRgb8::with_data(
            generator.config.width,
            generator.config.height,
            &generated_image.to_rgb8(),
        );

        let ref_image =
            image::open("tests/data/golden_files/components-object_wireframe_only.png").unwrap();
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
