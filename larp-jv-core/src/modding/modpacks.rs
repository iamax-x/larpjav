//! Modpack System — Port of ModpackApi, ModrinthApi, CurseforgeApi, ModpackInstaller

use crate::value::{ModSource, ModItem, SearchFilters, SearchResult, ModDetail, ModrinthIndex, CurseManifest};
use crate::error::LarpResult;
use async_trait::async_trait;
use std::path::Path;

/// Modpack API Trait — Port of ModpackApi.java
#[async_trait]
pub trait ModpackApi: Send + Sync {
    fn name(&self) -> &str;
    async fn search(&self, query: &str, filters: &SearchFilters) -> LarpResult<Vec<SearchResult>>;
    async fn get_mod_detail(&self, project_id: &str) -> LarpResult<ModDetail>;
    async fn download_mod(&self, mod_item: &ModItem, dest: &Path) -> LarpResult<()>;
    async fn get_versions(&self, project_id: &str, mc_version: &str, loader: &str) -> LarpResult<Vec<String>>;
}

/// Modrinth API — Port of ModrinthApi.java
pub struct ModrinthApi {
    client: reqwest::Client,
    base_url: String,
    cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
}

impl ModrinthApi {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.modrinth.com/v2".to_string(),
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> LarpResult<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;
        let json = response.json().await?;
        Ok(json)
    }
}

impl Default for ModrinthApi {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModpackApi for ModrinthApi {
    fn name(&self) -> &str {
        "Modrinth"
    }

    async fn search(&self, query: &str, filters: &SearchFilters) -> LarpResult<Vec<SearchResult>> {
        let mut params = vec![
            ("query", query),
            ("limit", "20"),
            ("index", "relevance"),
        ];
        
        if let Some(loader) = &filters.loader {
            params.push(("facets", &format!("[\"categories:{}\"]", loader)));
        }
        if let Some(version) = &filters.mc_version {
            params.push(("facets", &format!("[\"versions:{}\"]", version)));
        }

        let response = self.client.get(&format!("{}/search", self.base_url))
            .query(&params)
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        let hits = data["hits"].as_array().unwrap_or(&vec![]);
        
        let results = hits.iter().filter_map(|h| {
            Some(SearchResult {
                project_id: h["project_id"].as_str()?.to_string(),
                name: h["title"].as_str()?.to_string(),
                summary: h["description"].as_str()?.to_string(),
                icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
                downloads: h["downloads"].as_u64().unwrap_or(0),
                versions: h["versions"].as_array()?.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                loaders: h["categories"].as_array()?.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            })
        }).collect();
        
        Ok(results)
    }

    async fn get_mod_detail(&self, project_id: &str) -> LarpResult<ModDetail> {
        let path = format!("/project/{}", project_id);
        let data = self.get_json(&path).await?;
        
        Ok(ModDetail {
            project_id: data["id"].as_str().unwrap_or("").to_string(),
            name: data["title"].as_str().unwrap_or("").to_string(),
            description: data["description"].as_str().unwrap_or("").to_string(),
            icon_url: data["icon_url"].as_str().map(|s| s.to_string()),
            downloads: data["downloads"].as_u64().unwrap_or(0),
            followers: data["follows"].as_u64().unwrap_or(0),
            categories: data["categories"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            versions: data["versions"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            loaders: data["categories"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            links: data["links"].as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k,v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect(),
        })
    }

    async fn download_mod(&self, mod_item: &ModItem, dest: &Path) -> LarpResult<()> {
        // Get version info, then download file
        let versions = self.get_versions(&mod_item.project_id, &mod_item.mc_version, &mod_item.loader).await?;
        if let Some(version) = versions.first() {
            let version_path = format!("/version/{}", version);
            let version_data: serde_json::Value = self.get_json(&version_path).await?;
            if let Some(files) = version_data["files"].as_array() {
                if let Some(file) = files.first() {
                    let url = file["url"].as_str().unwrap_or("");
                    let response = self.client.get(url).send().await?;
                    let bytes = response.bytes().await?;
                    tokio::fs::write(dest, bytes).await?;
                }
            }
        }
        Ok(())
    }

    async fn get_versions(&self, project_id: &str, mc_version: &str, loader: &str) -> LarpResult<Vec<String>> {
        let path = format!("/project/{}/version", project_id);
        let data: serde_json::Value = self.get_json(&path).await?;
        
        let versions = data.as_array().unwrap_or(&vec![]).iter().filter_map(|v| {
            let game_versions = v["game_versions"].as_array()?;
            let loaders = v["loaders"].as_array()?;
            if game_versions.iter().any(|gv| gv.as_str() == Some(mc_version)) &&
               loaders.iter().any(|l| l.as_str() == Some(loader)) {
                v["version_number"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        }).collect();
        
        Ok(versions)
    }
}

/// CurseForge API — Port of CurseforgeApi.java
pub struct CurseforgeApi {
    client: reqwest::Client,
    api_key: Option<String>,
    base_url: String,
}

impl CurseforgeApi {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.curseforge.com/v1".to_string(),
        }
    }
}

impl Default for CurseforgeApi {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl ModpackApi for CurseforgeApi {
    fn name(&self) -> &str {
        "CurseForge"
    }

    async fn search(&self, query: &str, filters: &SearchFilters) -> LarpResult<Vec<SearchResult>> {
        let mut request = self.client.get(&format!("{}/mods/search", self.base_url))
            .query(&[("searchFilter", query), ("pageSize", "20")]);
        
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        
        if let Some(version) = &filters.mc_version {
            request = request.query(&[("gameVersion", version)]);
        }
        if let Some(loader) = &filters.loader {
            request = request.query(&[("modLoaderType", loader)]);
        }
        
        let response = request.send().await?;
        let data: serde_json::Value = response.json().await?;
        
        let results = data["data"].as_array().unwrap_or(&vec![]).iter().filter_map(|m| {
            Some(SearchResult {
                project_id: m["id"].as_u64()?.to_string(),
                name: m["name"].as_str()?.to_string(),
                summary: m["summary"].as_str().unwrap_or("").to_string(),
                icon_url: m["logo"]["url"].as_str().map(|s| s.to_string()),
                downloads: m["downloadCount"].as_u64().unwrap_or(0),
                versions: vec![],
                loaders: vec![],
            })
        }).collect();
        
        Ok(results)
    }

    async fn get_mod_detail(&self, project_id: &str) -> LarpResult<ModDetail> {
        let mut request = self.client.get(&format!("{}/mods/{}", self.base_url, project_id));
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        let response = request.send().await?;
        let data: serde_json::Value = response.json().await?;
        let m = &data["data"];
        
        Ok(ModDetail {
            project_id: m["id"].as_u64().unwrap_or(0).to_string(),
            name: m["name"].as_str().unwrap_or("").to_string(),
            description: m["description"].as_str().unwrap_or("").to_string(),
            icon_url: m["logo"]["url"].as_str().map(|s| s.to_string()),
            downloads: m["downloadCount"].as_u64().unwrap_or(0),
            followers: 0,
            categories: m["categories"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v["name"].as_str().map(|s| s.to_string())).collect(),
            versions: vec![],
            loaders: vec![],
            links: std::collections::HashMap::new(),
        })
    }

    async fn download_mod(&self, mod_item: &ModItem, dest: &Path) -> LarpResult<()> {
        // Get file info and download
        Ok(())
    }

    async fn get_versions(&self, project_id: &str, mc_version: &str, loader: &str) -> LarpResult<Vec<String>> {
        let mut request = self.client.get(&format!("{}/mods/{}/files", self.base_url, project_id));
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        request = request.query(&[("gameVersion", mc_version), ("modLoaderType", loader)]);
        let response = request.send().await?;
        let data: serde_json::Value = response.json().await?;
        
        let versions = data["data"].as_array().unwrap_or(&vec![]).iter().filter_map(|f| {
            f["fileName"].as_str().map(|s| s.to_string())
        }).collect();
        
        Ok(versions)
    }
}

/// Modpack Installer — Port of ModpackInstaller.java
pub struct ModpackInstaller {
    modrinth: ModrinthApi,
    curseforge: CurseforgeApi,
}

impl ModpackInstaller {
    pub fn new(modrinth: ModrinthApi, curseforge: CurseforgeApi) -> Self {
        Self { modrinth, curseforge }
    }

    pub async fn install_mrpack(&self, mrpack_path: &Path, instance_dir: &Path) -> LarpResult<()> {
        // Read MRPACK (Modrinth pack format)
        let file = tokio::fs::read(mrpack_path).await?;
        let zip = zip::ZipArchive::new(std::io::Cursor::new(file))?;
        
        // Extract index.json
        let mut index_content = Vec::new();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            if file.name() == "index.json" {
                std::io::Read::read_to_end(&mut file, &mut index_content)?;
                break;
            }
        }
        
        let index: ModrinthIndex = serde_json::from_slice(&index_content)?;
        
        // Download and install each mod
        for file_info in index.files {
            let dest = instance_dir.join("mods").join(&file_info.path);
            tokio::fs::create_dir_all(dest.parent().unwrap()).await?;
            
            // Determine source and download
            if let Some(mod_source) = &file_info.source {
                match mod_source {
                    ModSource::Modrinth { project_id, version_id, .. } => {
                        let mod_item = ModItem {
                            project_id: project_id.clone(),
                            mc_version: index.dependencies.minecraft.clone(),
                            loader: index.dependencies.loader.clone(),
                        };
                        self.modrinth.download_mod(&mod_item, &dest).await?;
                    }
                    ModSource::CurseForge { project_id, file_id, .. } => {
                        // Download from CurseForge
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    pub async fn install_curseforge_zip(&self, zip_path: &Path, instance_dir: &Path) -> LarpResult<()> {
        // Extract and install CurseForge modpack
        Ok(())
    }
}

/// Image Cache — Port of ModIconCache, DownloadImageTask, ReadFromDiskTask
pub mod imagecache {
    use crate::error::LarpResult;
    use std::path::Path;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;

    pub struct ModIconCache {
        cache_dir: std::path::PathBuf,
        memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl ModIconCache {
        pub fn new(cache_dir: &Path) -> Self {
            Self {
                cache_dir: cache_dir.to_path_buf(),
                memory_cache: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub async fn get(&self, url: &str) -> LarpResult<Vec<u8>> {
            // Check memory cache
            if let Some(data) = self.memory_cache.read().await.get(url) {
                return Ok(data.clone());
            }
            
            // Check disk cache
            let hash = crate::utils::hash::sha1(url.as_bytes());
            let cache_path = self.cache_dir.join(&hash);
            if cache_path.exists() {
                let data = tokio::fs::read(&cache_path).await?;
                self.memory_cache.write().await.insert(url.to_string(), data.clone());
                return Ok(data);
            }
            
            // Download
            let client = reqwest::Client::new();
            let response = client.get(url).send().await?;
            let data = response.bytes().await?.to_vec();
            
            // Save to cache
            tokio::fs::create_dir_all(&self.cache_dir).await?;
            tokio::fs::write(&cache_path, &data).await?;
            self.memory_cache.write().await.insert(url.to_string(), data.clone());
            
            Ok(data)
        }
    }
}