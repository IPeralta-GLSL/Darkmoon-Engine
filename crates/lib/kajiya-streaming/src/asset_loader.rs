use crate::{ResourceId, level_of_detail::LodLevel};
use anyhow::Result;
use std::path::Path;
use std::fs;
use log::{debug, info, warn};
use std::sync::Arc;
use parking_lot::Mutex;

/// Prioridad de carga de un asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Solicitud de carga de asset
#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub resource_id: ResourceId,
    pub path: String,
    pub priority: LoadPriority,
    pub lod_level: LodLevel,
}

/// Tipo de asset detectado
#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Audio,
    Scene,
    Unknown,
}

/// Datos de un asset cargado
#[derive(Debug, Clone)]
pub struct AssetData {
    pub asset_type: AssetType,
    pub data: Vec<u8>,
    pub metadata: AssetMetadata,
}

impl AssetData {
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Metadatos de un asset
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub original_size: u64,
    pub compressed_size: u64,
    pub format: String,
    pub creation_time: std::time::SystemTime,
    pub lod_level: LodLevel,
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            original_size: 0,
            compressed_size: 0,
            format: String::new(),
            creation_time: std::time::UNIX_EPOCH,
            lod_level: LodLevel::Medium,
        }
    }
}

/// Cargador síncrono de assets
#[derive(Clone)]
pub struct AssetLoader {
    base_path: String,
    max_concurrent: usize,
    current_loads: Arc<Mutex<usize>>,
}

impl AssetLoader {
    /// Crea un nuevo cargador de assets
    pub fn new(max_concurrent_loads: usize, base_path: &str) -> Result<Self> {
        info!("Inicializando cargador de assets con {} workers concurrentes", max_concurrent_loads);
        info!("Directorio base: {}", base_path);
        
        // Verificar que el directorio base existe
        let path = Path::new(base_path);
        if !path.exists() {
            warn!("Directorio base no existe, creándolo: {}", base_path);
            fs::create_dir_all(path)?;
        }
        
        Ok(Self {
            base_path: base_path.to_string(),
            max_concurrent: max_concurrent_loads,
            current_loads: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Carga un asset de forma asíncrona
    pub async fn load_asset(&self, request: &LoadRequest) -> Result<AssetData> {
        // Esperar hasta que podamos cargar (control de concurrencia simple)
        loop {
            {
                let mut current = self.current_loads.lock();
                if *current < self.max_concurrent {
                    *current += 1;
                    break;
                }
            }
            // Esperar un poco antes de intentar de nuevo
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // Asegurar que decrementemos el contador al final
        let _guard = scopeguard::guard((), |_| {
            let mut current = self.current_loads.lock();
            *current -= 1;
        });
        
        debug!("Cargando asset: {} con prioridad {:?}", request.resource_id, request.priority);
        
        let full_path = Path::new(&self.base_path).join(&request.path);
        
        // Verificar que el archivo existe
        if !full_path.exists() {
            return Err(anyhow::anyhow!("Archivo no encontrado: {}", full_path.display()));
        }
        
        // Detectar tipo de asset por extensión
        let asset_type = self.detect_asset_type(&full_path);
        
        // Cargar el archivo
        let data = fs::read(&full_path)?;
        let original_size = data.len() as u64;
        
        // Procesar según el nivel de detalle solicitado
        let processed_data = self.process_lod_data(data, &asset_type, request.lod_level)?;
        
        let metadata = AssetMetadata {
            original_size,
            compressed_size: processed_data.len() as u64,
            format: self.get_format_string(&full_path),
            creation_time: fs::metadata(&full_path)?.created().unwrap_or(std::time::SystemTime::now()),
            lod_level: request.lod_level,
        };
        
        let asset_data = AssetData {
            asset_type,
            data: processed_data,
            metadata,
        };
        
        info!("Asset cargado: {} ({} bytes -> {} bytes)", 
              request.resource_id, 
              original_size, 
              asset_data.data.len());
        
        Ok(asset_data)
    }
    
    /// Carga múltiples assets en paralelo
    pub async fn load_multiple_assets(&self, requests: Vec<LoadRequest>) -> Vec<Result<AssetData>> {
        let futures = requests.iter().map(|request| self.load_asset(request));
        futures::future::join_all(futures).await
    }
    
    /// Precarga assets basándose en patrones predictivos
    pub async fn preload_assets(&self, patterns: &[String]) -> Result<Vec<AssetData>> {
        let mut load_requests = Vec::new();
        
        for pattern in patterns {
            let matching_files = self.find_matching_files(pattern).await?;
            for file_path in matching_files {
                let request = LoadRequest {
                    resource_id: file_path.clone(),
                    path: file_path,
                    priority: LoadPriority::Low,
                    lod_level: LodLevel::Low, // Precarga con baja calidad
                };
                load_requests.push(request);
            }
        }
        
        info!("Precargando {} assets", load_requests.len());
        
        let results = self.load_multiple_assets(load_requests).await;
        let successful_assets: Vec<AssetData> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();
            
        info!("Precargados {} assets exitosamente", successful_assets.len());
        Ok(successful_assets)
    }
    
    /// Detecta el tipo de asset basándose en la extensión del archivo
    fn detect_asset_type(&self, path: &Path) -> AssetType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("gltf") | Some("glb") | Some("obj") | Some("fbx") => AssetType::Mesh,
            Some("png") | Some("jpg") | Some("jpeg") | Some("tga") | Some("dds") | Some("hdr") | Some("exr") => AssetType::Texture,
            Some("mtl") | Some("mat") => AssetType::Material,
            Some("wav") | Some("mp3") | Some("ogg") => AssetType::Audio,
            Some("dmoon") | Some("scene") => AssetType::Scene,
            _ => AssetType::Unknown,
        }
    }
    
    /// Obtiene la cadena de formato del archivo
    fn get_format_string(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_uppercase()
    }
    
    /// Procesa los datos según el nivel de detalle solicitado
    fn process_lod_data(&self, data: Vec<u8>, asset_type: &AssetType, lod_level: LodLevel) -> Result<Vec<u8>> {
        match asset_type {
            AssetType::Texture => self.process_texture_lod(data, lod_level),
            AssetType::Mesh => self.process_mesh_lod(data, lod_level),
            _ => Ok(data), // Para otros tipos, devolver datos originales
        }
    }
    
    /// Procesa texturas según el nivel de detalle
    fn process_texture_lod(&self, data: Vec<u8>, lod_level: LodLevel) -> Result<Vec<u8>> {
        match lod_level {
            LodLevel::High => Ok(data), // Calidad completa
            LodLevel::Medium => {
                // En una implementación real, aquí reduciríamos la resolución al 50%
                debug!("Procesando textura a calidad media");
                Ok(data)
            }
            LodLevel::Low => {
                // En una implementación real, aquí reduciríamos la resolución al 25%
                debug!("Procesando textura a calidad baja");
                Ok(data)
            }
        }
    }
    
    /// Procesa meshes según el nivel de detalle
    fn process_mesh_lod(&self, data: Vec<u8>, lod_level: LodLevel) -> Result<Vec<u8>> {
        match lod_level {
            LodLevel::High => Ok(data), // Malla completa
            LodLevel::Medium => {
                // En una implementación real, aquí simplificaríamos la geometría
                debug!("Procesando malla a calidad media");
                Ok(data)
            }
            LodLevel::Low => {
                // En una implementación real, aquí simplificaríamos más la geometría
                debug!("Procesando malla a calidad baja");
                Ok(data)
            }
        }
    }
    
    /// Encuentra archivos que coinciden con un patrón
    async fn find_matching_files(&self, pattern: &str) -> Result<Vec<String>> {
        let base_path = Path::new(&self.base_path);
        let mut matching_files = Vec::new();
        
        // Implementación simple - en producción usarías un sistema de patrones más sofisticado
        let entries = fs::read_dir(base_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(pattern) {
                    if let Some(relative_path) = path.strip_prefix(base_path).ok() {
                        matching_files.push(relative_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(matching_files)
    }
}
