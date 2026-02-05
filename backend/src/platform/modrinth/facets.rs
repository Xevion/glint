use serde::Serialize;

#[derive(Debug, Clone)]
pub enum Facet {
    ProjectType(ProjectType),
    Categories(String),
    Versions(String),
    License(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectType {
    Mod,
    Modpack,
    ResourcePack,
    Shader,
}

impl ProjectType {
    fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Mod => "mod",
            ProjectType::Modpack => "modpack",
            ProjectType::ResourcePack => "resourcepack",
            ProjectType::Shader => "shader",
        }
    }
}

impl Serialize for Facet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Facet::ProjectType(pt) => format!("project_type:{}", pt.as_str()),
            Facet::Categories(c) => format!("categories:{c}"),
            Facet::Versions(v) => format!("versions:{v}"),
            Facet::License(l) => format!("license:{l}"),
        };
        serializer.collect_str(&s)
    }
}
