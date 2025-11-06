//! Catalog of glTF models from Khronos Sample Assets
//!
//! This module provides a curated list of 3D models that can be downloaded
//! and viewed in the ray tracer.

use std::fmt;

/// Metadata for a glTF model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Short identifier (e.g., "cube", "suzanne")
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Brief description
    pub description: &'static str,
    /// Direct URL to the .glb file on GitHub
    pub url: &'static str,
    /// Approximate file size in KB
    pub size_kb: u32,
    /// Complexity level (Simple, Medium, Complex)
    pub complexity: Complexity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
}

impl fmt::Display for Complexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Complexity::Simple => write!(f, "Simple"),
            Complexity::Medium => write!(f, "Medium"),
            Complexity::Complex => write!(f, "Complex"),
        }
    }
}

/// Get the full catalog of available models
pub fn get_catalog() -> Vec<ModelInfo> {
    vec![
        // Simple geometric shapes - great for testing
        ModelInfo {
            id: "cube",
            name: "Simple Cube",
            description: "Basic cube with 12 triangles",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Cube/glTF/Cube.gltf",
            size_kb: 3,
            complexity: Complexity::Simple,
        },
        ModelInfo {
            id: "triangle",
            name: "Triangle Without Indices",
            description: "Single triangle primitive",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/TriangleWithoutIndices/glTF/TriangleWithoutIndices.gltf",
            size_kb: 1,
            complexity: Complexity::Simple,
        },
        ModelInfo {
            id: "box",
            name: "Box",
            description: "Textured box with vertex colors",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Box/glTF/Box.gltf",
            size_kb: 4,
            complexity: Complexity::Simple,
        },
        ModelInfo {
            id: "suzanne",
            name: "Suzanne (Blender Monkey)",
            description: "Classic Blender monkey head",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Suzanne/glTF/Suzanne.gltf",
            size_kb: 100,
            complexity: Complexity::Medium,
        },
        // Medium complexity models
        ModelInfo {
            id: "duck",
            name: "Duck",
            description: "Classic glTF duck model",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Duck/glTF/Duck.gltf",
            size_kb: 100,
            complexity: Complexity::Medium,
        },
        ModelInfo {
            id: "avocado",
            name: "Avocado",
            description: "Detailed avocado model",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Avocado/glTF/Avocado.gltf",
            size_kb: 200,
            complexity: Complexity::Medium,
        },
        ModelInfo {
            id: "brain_stem",
            name: "Brain Stem",
            description: "Anatomical brain stem model",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/BrainStem/glTF/BrainStem.gltf",
            size_kb: 500,
            complexity: Complexity::Medium,
        },
        // Complex models
        ModelInfo {
            id: "damaged_helmet",
            name: "Damaged Helmet",
            description: "Battle-worn sci-fi helmet with PBR materials",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/DamagedHelmet/glTF/DamagedHelmet.gltf",
            size_kb: 3000,
            complexity: Complexity::Complex,
        },
        ModelInfo {
            id: "flight_helmet",
            name: "Flight Helmet",
            description: "Detailed flight helmet",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/FlightHelmet/glTF/FlightHelmet.gltf",
            size_kb: 5000,
            complexity: Complexity::Complex,
        },
        ModelInfo {
            id: "lantern",
            name: "Lantern",
            description: "Vintage lantern with emissive materials",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Lantern/glTF/Lantern.gltf",
            size_kb: 2000,
            complexity: Complexity::Complex,
        },
        ModelInfo {
            id: "sci_fi_helmet",
            name: "Sci-Fi Helmet",
            description: "Futuristic helmet design",
            url: "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/SciFiHelmet/glTF/SciFiHelmet.gltf",
            size_kb: 4000,
            complexity: Complexity::Complex,
        },
    ]
}

/// Find a model by its ID
pub fn find_model(id: &str) -> Option<ModelInfo> {
    get_catalog().into_iter().find(|m| m.id == id)
}

/// Get models filtered by complexity
pub fn get_models_by_complexity(complexity: Complexity) -> Vec<ModelInfo> {
    get_catalog()
        .into_iter()
        .filter(|m| m.complexity == complexity)
        .collect()
}

/// Get a recommended starter model (simple and small)
pub fn get_starter_model() -> ModelInfo {
    find_model("cube").expect("Cube model should always be available")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_not_empty() {
        let catalog = get_catalog();
        assert!(!catalog.is_empty());
    }

    #[test]
    fn test_find_model() {
        let cube = find_model("cube");
        assert!(cube.is_some());
        assert_eq!(cube.unwrap().name, "Simple Cube");
    }

    #[test]
    fn test_all_ids_unique() {
        let catalog = get_catalog();
        let mut ids: Vec<&str> = catalog.iter().map(|m| m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), catalog.len(), "All model IDs should be unique");
    }

    #[test]
    fn test_complexity_filter() {
        let simple = get_models_by_complexity(Complexity::Simple);
        assert!(!simple.is_empty());
        assert!(simple.iter().all(|m| m.complexity == Complexity::Simple));
    }

    #[test]
    fn test_starter_model() {
        let starter = get_starter_model();
        assert_eq!(starter.id, "cube");
        assert_eq!(starter.complexity, Complexity::Simple);
    }
}

