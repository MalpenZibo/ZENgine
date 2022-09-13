use std::path::Path;

pub async fn load(asset_path: &Path) -> Vec<u8> {
    let data = async {
        std::fs::read(asset_path)
            .unwrap_or_else(|e| panic!("Could not load file {:?}: {}", asset_path, e))
    };

    data.await
}
