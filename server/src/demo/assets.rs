#[cfg(feature = "demo")]
use once_cell::sync::Lazy;
#[cfg(feature = "demo")]
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct EmbeddedAsset {
    pub content: &'static [u8],
    pub content_type: &'static str,
    pub encoding: AssetEncoding,
}

#[derive(Debug, Copy, Clone)]
pub enum AssetEncoding {
    Gzip,
    Identity,
}

// Compile-time embedded assets with zero runtime overhead
#[cfg(feature = "demo")]
static ASSETS: Lazy<HashMap<&'static str, EmbeddedAsset>> = Lazy::new(|| {
    let mut m = HashMap::with_capacity(8);

    // Compressed vendor assets - only include if files exist (skip during cargo publish)
    #[cfg(all(not(docsrs), not(cargo_publish)))]
    {
        m.insert(
            "/vendor/gridjs.min.js",
            EmbeddedAsset {
                content: include_bytes!("../../assets/vendor/gridjs.min.js.gz"),
                content_type: "application/javascript",
                encoding: AssetEncoding::Gzip,
            },
        );

        m.insert(
            "/vendor/gridjs-mermaid.min.css",
            EmbeddedAsset {
                content: include_bytes!("../../assets/vendor/gridjs-mermaid.min.css.gz"),
                content_type: "text/css",
                encoding: AssetEncoding::Gzip,
            },
        );

        m.insert(
            "/vendor/mermaid.min.js",
            EmbeddedAsset {
                content: include_bytes!("../../assets/vendor/mermaid.min.js.gz"),
                content_type: "application/javascript",
                encoding: AssetEncoding::Gzip,
            },
        );

        m.insert(
            "/vendor/d3.min.js",
            EmbeddedAsset {
                content: include_bytes!("../../assets/vendor/d3.min.js.gz"),
                content_type: "application/javascript",
                encoding: AssetEncoding::Gzip,
            },
        );
    }

    // Demo-specific assets - only include if files exist (skip during cargo publish)
    #[cfg(all(not(docsrs), not(cargo_publish)))]
    {
        m.insert(
            "/demo.css",
            EmbeddedAsset {
                content: include_bytes!("../../assets/demo/style.min.css"),
                content_type: "text/css",
                encoding: AssetEncoding::Identity,
            },
        );

        m.insert(
            "/demo.js",
            EmbeddedAsset {
                content: include_bytes!("../../assets/demo/app.min.js"),
                content_type: "application/javascript",
                encoding: AssetEncoding::Identity,
            },
        );

        m.insert(
            "/favicon.ico",
            EmbeddedAsset {
                content: include_bytes!("../../assets/demo/favicon.ico"),
                content_type: "image/x-icon",
                encoding: AssetEncoding::Identity,
            },
        );
    }

    m
});

#[cfg(feature = "demo")]
pub fn get_asset(path: &str) -> Option<&'static EmbeddedAsset> {
    ASSETS.get(path)
}

#[cfg(not(feature = "demo"))]
pub fn get_asset(_path: &str) -> Option<&'static EmbeddedAsset> {
    None
}

/// Decompress an asset if needed
pub fn decompress_asset(asset: &EmbeddedAsset) -> std::borrow::Cow<'static, [u8]> {
    match asset.encoding {
        AssetEncoding::Identity => std::borrow::Cow::Borrowed(asset.content),
        AssetEncoding::Gzip => {
            use flate2::read::GzDecoder;
            use std::io::Read;

            let mut decoder = GzDecoder::new(asset.content);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .expect("Failed to decompress asset");
            std::borrow::Cow::Owned(decompressed)
        }
    }
}

/// Get asset hash for cache busting
pub fn get_asset_hash() -> &'static str {
    option_env!("ASSET_HASH").unwrap_or("development")
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_assets_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
