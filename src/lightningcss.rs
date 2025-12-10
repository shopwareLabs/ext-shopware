use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::{Browsers, Targets};

/// LightningCSS - A fast CSS parser, transformer, and minifier
///
/// Example usage:
/// ```php
/// $css = new LightningCSS();
/// $result = $css->minify('body { color: red; }');
/// ```
#[php_class]
#[php(name = "Shopware\\PHPExtension\\LightningCSS\\LightningCSS")]
pub struct LightningCSS {
    targets: Targets,
}

#[php_impl]
impl LightningCSS {
    /// Create a new LightningCSS instance
    pub fn __construct() -> Self {
        LightningCSS { 
            targets: Targets::default(),
        }
    }

    /// Set browser targets for compatibility transformations
    ///
    /// Example:
    /// ```php
    /// $css->setBrowserTargets([
    ///     'chrome' => 95,
    ///     'firefox' => 90,
    ///     'safari' => 14,
    /// ]);
    /// ```
    pub fn set_browser_targets(&mut self, browsers: &ext_php_rs::types::ZendHashTable) -> PhpResult<()> {
        let mut browser_targets = Browsers::default();

        for (key, value) in browsers.iter() {
            let browser_name = match key {
                ext_php_rs::types::ArrayKey::String(s) => s.to_lowercase(),
                ext_php_rs::types::ArrayKey::Str(s) => s.to_lowercase(),
                _ => continue,
            };

            let version = value.long().unwrap_or(0) as u32;
            // Convert version number to lightningcss format (major << 16 | minor << 8 | patch)
            let version_bits = (version << 16) | (0 << 8) | 0;

            match browser_name.as_str() {
                "chrome" => browser_targets.chrome = Some(version_bits),
                "firefox" => browser_targets.firefox = Some(version_bits),
                "safari" => browser_targets.safari = Some(version_bits),
                "edge" => browser_targets.edge = Some(version_bits),
                "ie" => browser_targets.ie = Some(version_bits),
                "opera" => browser_targets.opera = Some(version_bits),
                "ios_safari" | "ios" => browser_targets.ios_saf = Some(version_bits),
                "android" => browser_targets.android = Some(version_bits),
                "samsung" => browser_targets.samsung = Some(version_bits),
                _ => {}
            }
        }

        self.targets = Targets { 
            browsers: Some(browser_targets), 
            ..Default::default() 
        };
        Ok(())
    }

    /// Minify CSS code
    ///
    /// Returns minified CSS string
    pub fn minify(&self, css: &str) -> PhpResult<String> {
        let mut stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| PhpException::default(format!("CSS parse error: {:?}", e)))?;

        let minify_options = MinifyOptions {
            targets: self.targets.clone(),
            ..Default::default()
        };

        stylesheet
            .minify(minify_options)
            .map_err(|e| PhpException::default(format!("CSS minify error: {:?}", e)))?;

        let printer_options = PrinterOptions {
            minify: true,
            targets: self.targets.clone(),
            ..Default::default()
        };

        let result = stylesheet
            .to_css(printer_options)
            .map_err(|e| PhpException::default(format!("CSS print error: {:?}", e)))?;

        Ok(result.code)
    }

    /// Transform CSS for browser compatibility without minification
    ///
    /// Adds vendor prefixes and transforms modern syntax for older browsers
    pub fn transform(&self, css: &str) -> PhpResult<String> {
        let mut stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| PhpException::default(format!("CSS parse error: {:?}", e)))?;

        let minify_options = MinifyOptions {
            targets: self.targets.clone(),
            ..Default::default()
        };

        stylesheet
            .minify(minify_options)
            .map_err(|e| PhpException::default(format!("CSS transform error: {:?}", e)))?;

        let printer_options = PrinterOptions {
            minify: false,
            targets: self.targets.clone(),
            ..Default::default()
        };

        let result = stylesheet
            .to_css(printer_options)
            .map_err(|e| PhpException::default(format!("CSS print error: {:?}", e)))?;

        Ok(result.code)
    }

    /// Parse and pretty-print CSS (formats the CSS)
    pub fn format(&self, css: &str) -> PhpResult<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| PhpException::default(format!("CSS parse error: {:?}", e)))?;

        let printer_options = PrinterOptions {
            minify: false,
            ..Default::default()
        };

        let result = stylesheet
            .to_css(printer_options)
            .map_err(|e| PhpException::default(format!("CSS print error: {:?}", e)))?;

        Ok(result.code)
    }

    /// Validate CSS syntax
    ///
    /// Returns true if CSS is valid, throws exception with details if invalid
    pub fn validate(&self, css: &str) -> PhpResult<bool> {
        StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| PhpException::default(format!("CSS validation error: {:?}", e)))?;

        Ok(true)
    }

    /// Parse CSS and return analysis information
    ///
    /// Returns an array with information about the stylesheet
    pub fn analyze(&self, css: &str) -> PhpResult<Zval> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| PhpException::default(format!("CSS parse error: {:?}", e)))?;

        let mut arr = ext_php_rs::types::ZendHashTable::new();
        let _ = arr.insert("rules_count", stylesheet.rules.0.len() as i64);

        let mut zval = Zval::new();
        zval.set_hashtable(arr);
        Ok(zval)
    }
}
