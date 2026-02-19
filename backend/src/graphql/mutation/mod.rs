pub mod shaders;

use async_graphql::MergedObject;

use shaders::ShaderMutation;

/// Root mutation type — merges all domain mutation types.
#[derive(MergedObject, Default)]
pub struct MutationRoot(ShaderMutation);
