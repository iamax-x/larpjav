//! Mod search (Modrinth, CurseForge)

use crate::error::{LarpError, LarpResult};
use crate::value::ModInfo;
use crate::utils::download::create_client;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mod search result
#[derive(Debug, Clone)]
pub struct ModSearchResult {
    pub mods: Vec<ModInfo>,
    pub total_hits: u32,
}

/// Mod search client
pub struct ModSearchClient {
    client: Client,
}

impl ModSearchClient {
    pub fn new() -> LarpResult<Self> {
        Ok(Self {
            client: create_client()?,
        })
    }

    /// Search Modrinth
    pub async fn search_modrinth(
        &self,
        query: &str,
        loader: crate::value::ModLoaderType,
        mc_version: &str,
        limit: usize,
    ) -> LarpResult<ModSearchResult> {
        let mut url = "https://api.modrinth.com/v2/search".to_string();
        let mut params = Vec::new();

        params.push(format!("query={}", urlencoding::encode(query)));
        params.push(format!("limit={}", limit));
        params.push(format!("facets=[[\"categories:{}\"]]", loader_name(loader)));
        params.push(format!("versions=[\"{}\"]", mc_version));

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let resp = self.client.get(&url).send().await?;
        let json: ModrinthSearchResponse = resp.json().await?;

        let mut mods = Vec::new();
        for hit in json.hits {
            let mut mod_info = ModInfo {
                id: hit.project_id,
                version: hit.version_number.unwrap_or_default(),
                loader,
                enabled: true,
                url: Some(format!("https://modrinth.com/mod/{}", hit.slug)),
                sha1: None,
            };

            // Get latest file for version
            if let Some(files) = hit.files {
                for file in files {
                    if file.version_type == "release" && file.game_versions.iter().any(|v| v == mc_version) {
                        mod_info.url = Some(file.url);
                        break;
                    }
                }
            }

            mods.push(mod_info);
        }

        Ok(ModSearchResult {
            mods,
            total_hits: json.total_hits,
        })
    }

    /// Search CurseForge (requires API key)
    pub async fn search_curseforge(
        &self,
        query: &str,
        loader: crate::value::ModLoaderType,
        mc_version: &str,
        limit: usize,
        api_key: &str,
    ) -> LarpResult<ModSearchResult> {
        let url = "https://api.curseforge.com/v1/mods/search";
        let resp = self.client
            .get(url)
            .header("x-api-key", api_key)
            .query(&[
                ("gameId", "432"), // Minecraft
                ("searchFilter", query),
                ("modLoaderType", &curseforge_loader_id(loader).to_string()),
                ("gameVersion", mc_version),
                ("pageSize", &limit.to_string()),
                ("sortField", "2"), // Popularity
                ("sortOrder", "desc"),
            ])
            .send()
            .await?;

        let json: CurseForgeSearchResponse = resp.json().await?;

        let mut mods = Vec::new();
        for hit in json.data {
            let mod_info = ModInfo {
                id: hit.id.to_string(),
                version: hit.latestFiles.first().map(|f| f.displayName.clone()).unwrap_or_default(),
                loader,
                enabled: true,
                url: Some(format!("https://www.curseforge.com/minecraft/mc-mods/{}", hit.slug)),
                sha1: None,
            };
            mods.push(mod_info);
        }

        Ok(ModSearchResult {
            mods,
            total_hits: json.pagination.totalCount as u32,
        })
    }
}

fn loader_name(loader: crate::value::ModLoaderType) -> &'static str {
    match loader {
        crate::value::ModLoaderType::Forge => "forge",
        crate::value::ModLoaderType::Fabric => "fabric",
        crate::value::ModLoaderType::Quilt => "quilt",
        crate::value::ModLoaderType::NeoForge => "neoforge",
        _ => "fabric",
    }
}

fn curseforge_loader_id(loader: crate::value::ModLoaderType) -> i32 {
    match loader {
        crate::value::ModLoaderType::Forge => 1,
        crate::value::ModLoaderType::Fabric => 4,
        crate::value::ModLoaderType::Quilt => 5,
        crate::value::ModLoaderType::NeoForge => 6,
        _ => 4,
    }
}

/// Modrinth API response
#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
    total_hits: u32,
}

#[derive(Debug, Deserialize)]
struct ModrinthHit {
    project_id: String,
    slug: String,
    version_number: Option<String>,
    files: Option<Vec<ModrinthFile>>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    version_type: String,
    game_versions: Vec<String>,
    url: String,
}

/// CurseForge API response
#[derive(Debug, Deserialize)]
struct CurseForgeSearchResponse {
    data: Vec<CurseForgeMod>,
    pagination: CurseForgePagination,
}

#[derive(Debug, Deserialize)]
struct CurseForgeMod {
    id: i32,
    slug: String,
    latestFiles: Vec<CurseForgeFile>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFile {
    displayName: String,
}

#[derive(Debug, Deserialize)]
struct CurseForgePagination {
    totalCount: i32,
}

impl Default for ModSearchClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ModSearchClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_name() {
        assert_eq!(loader_name(crate::value::ModLoaderType::Fabric), "fabric");
        assert_eq!(loader_name(crate::value::ModLoaderType::Forge), "forge");
    }
}