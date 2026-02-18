use async_graphql::SimpleObject;

use crate::models::taxonomy::{Category, Feature, Tag};

#[derive(SimpleObject, Debug, Clone)]
pub struct CategoryNode {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<Category> for CategoryNode {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            slug: c.slug,
            name: c.name,
            description: c.description,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct FeatureNode {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<Feature> for FeatureNode {
    fn from(f: Feature) -> Self {
        Self {
            id: f.id,
            slug: f.slug,
            name: f.name,
            description: f.description,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct TagNode {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<Tag> for TagNode {
    fn from(t: Tag) -> Self {
        Self {
            id: t.id,
            slug: t.slug,
            name: t.name,
            description: t.description,
        }
    }
}
