use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::binary::Binary;
use libvips::ops;
use libvips::VipsImage;
use std::ffi::CString;

fn is_format_supported(saver_name: &str) -> bool {
    let basename = CString::new("VipsOperation").unwrap();
    let nickname = CString::new(saver_name).unwrap();

    unsafe {
        let gtype = libvips::bindings::vips_type_find(
            basename.as_ptr(),
            nickname.as_ptr(),
        );
        gtype != 0
    }
}

/// Image - Fast image processing library
///
/// Example usage:
/// ```php
/// $img = Vips::fromFile("path/to/file");
/// $dimension = $img->getDimension();
/// $newImg = $img->resize(800, 600);
/// $img->saveFile("output.jpg");
/// ```
#[php_class]
#[php(name = "Shopware\\PHPExtension\\Image\\Image")]
pub struct Image {
    image: VipsImage,
}

#[php_impl]
impl Image {
    pub const FORMAT_JPEG: &'static str = "jpeg";
    pub const FORMAT_PNG: &'static str = "png";
    pub const FORMAT_WEBP: &'static str = "webp";
    pub const FORMAT_TIFF: &'static str = "tiff";
    pub const FORMAT_JXL: &'static str = "jxl";
    pub const FORMAT_AVIF: &'static str = "avif";

    /// Get list of supported image formats
    ///
    /// Returns an array with format names as keys and boolean support status as values
    pub fn get_supported_formats() -> Zval {
        let mut arr = ext_php_rs::types::ZendHashTable::new();

        let _ = arr.insert("jpeg", true);
        let _ = arr.insert("png", true);
        let _ = arr.insert("webp", is_format_supported("webpsave"));
        let _ = arr.insert("tiff", is_format_supported("tiffsave"));
        let _ = arr.insert("jxl", is_format_supported("jxlsave"));
        let _ = arr.insert("avif", is_format_supported("heifsave"));

        let mut zval = Zval::new();
        zval.set_hashtable(arr);
        zval
    }

    /// Check if a specific format is supported
    pub fn supports_format(format: &str) -> bool {
        match format.to_lowercase().as_str() {
            "jpeg" | "jpg" => true,
            "png" => true,
            "webp" => is_format_supported("webpsave"),
            "tiff" => is_format_supported("tiffsave"),
            "jxl" | "jpegxl" => is_format_supported("jxlsave"),
            "avif" => is_format_supported("heifsave"),
            _ => false,
        }
    }

    /// Create a new Image instance from a file
    pub fn from_file(path: &str) -> PhpResult<Self> {
        let image = VipsImage::new_from_file(path)
            .map_err(|e| PhpException::default(format!("Failed to load image: {:?}", e)))?;
        Ok(Image { image })
    }

    /// Create a new Image instance from a string (buffer)
    pub fn from_string(data: Binary<u8>) -> PhpResult<Self> {
        let image = VipsImage::new_from_buffer(&data, "")
            .map_err(|e| PhpException::default(format!("Failed to load image from buffer: {:?}", e)))?;
        Ok(Image { image })
    }

    /// Get image dimensions
    ///
    /// Returns array with 'width' and 'height' keys
    pub fn get_dimension(&self) -> PhpResult<Zval> {
        let mut arr = ext_php_rs::types::ZendHashTable::new();
        let _ = arr.insert("width", self.image.get_width() as i64);
        let _ = arr.insert("height", self.image.get_height() as i64);

        let mut zval = Zval::new();
        zval.set_hashtable(arr);
        Ok(zval)
    }

    /// Get image format
    ///
    /// Returns the loader name (e.g., "jpegload", "pngload") or empty if unknown
    pub fn get_format(&self) -> String {
         // Metadata access not currently supported in this binding
         "".to_string()
    }

    /// Save image to file
    ///
    /// If format is provided, it tries to use it as extension/saver hint.
    pub fn save_file(&self, path: &str, format: Option<String>, quality: Option<i32>) -> PhpResult<()> {
        match format {
            Some(fmt) => {
                let fmt_lower = fmt.to_lowercase();
                let mut suffix = match fmt_lower.as_str() {
                    "jpeg" | "jpg" => ".jpg".to_string(),
                    "png" => ".png".to_string(),
                    "webp" => ".webp".to_string(),
                    "tiff" => ".tiff".to_string(),
                    "jxl" | "jpegxl" => ".jxl".to_string(),
                    "avif" => ".avif".to_string(),
                     _ => {
                        let save_path = match quality {
                            Some(q) => format!("{}[Q={}]", path, q),
                            None => path.to_string(),
                        };
                        return self.image.image_write_to_file(&save_path)
                            .map_err(|e| PhpException::default(format!("Failed to save image: {:?}", e)));
                    }
                };

                if let Some(q) = quality {
                    suffix = format!("{}[Q={}]", suffix, q);
                }

                let buffer = self.image.image_write_to_buffer(&suffix)
                     .map_err(|e| PhpException::default(format!("Failed to encode image: {:?}", e)))?;
                std::fs::write(path, buffer)
                     .map_err(|e| PhpException::default(format!("Failed to write file: {:?}", e)))?;
                Ok(())
            },
            None => {
                let save_path = match quality {
                    Some(q) => format!("{}[Q={}]", path, q),
                    None => path.to_string(),
                };
                self.image.image_write_to_file(&save_path)
                   .map_err(|e| PhpException::default(format!("Failed to save image: {:?}", e)))
            }
        }
    }

    /// Save image to string
    ///
    /// Returns the binary string of the image file
    pub fn save_string(&self, format: &str, quality: Option<i32>) -> PhpResult<Zval> {
        let mut suffix = match format.to_lowercase().as_str() {
            "jpeg" | "jpg" => ".jpg".to_string(),
            "png" => ".png".to_string(),
            "webp" => ".webp".to_string(),
            "tiff" => ".tiff".to_string(),
            "jxl" | "jpegxl" => ".jxl".to_string(),
            "avif" => ".avif".to_string(),
             _ => return Err(PhpException::default(format!("Unsupported format: {}", format))),
        };

        if let Some(q) = quality {
            suffix = format!("{}[Q={}]", suffix, q);
        }

        let buffer = self.image.image_write_to_buffer(&suffix)
            .map_err(|e| PhpException::default(format!("Failed to encode image: {:?}", e)))?;
        
        let mut zval = Zval::new();
        zval.set_binary(buffer);
        Ok(zval)
    }

    /// Resize an image
    ///
    /// Returns a NEW Image instance
    pub fn resize(&self, width: i32, height: i32) -> PhpResult<Image> {
        let original_width = self.image.get_width() as f64;
        let original_height = self.image.get_height() as f64;

        let scale_x = width as f64 / original_width;
        let scale_y = height as f64 / original_height;
        let scale = scale_x.min(scale_y); // Maintain aspect ratio

        let resized_image = ops::resize(&self.image, scale)
            .map_err(|e| PhpException::default(format!("Failed to resize image: {:?}", e)))?;
            
        Ok(Image { image: resized_image })
    }
}