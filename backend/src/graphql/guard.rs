use async_graphql::{Context, ErrorExtensions, Guard, Result};

use crate::auth::MaybeAuthUser;
use crate::error::AppError;
use crate::models::User;

/// Guard that requires the current user to be an authenticated admin.
///
/// Attach to queries or mutations via `#[graphql(guard = "AdminGuard")]`.
pub struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<MaybeAuthUser>()?;
        match &auth.0 {
            Some(auth_user) if auth_user.user.role.is_admin() => Ok(()),
            Some(_) => Err(AppError::Forbidden("Admin access required".into()).extend()),
            None => Err(AppError::Unauthorized("Authentication required".into()).extend()),
        }
    }
}

/// Extract the authenticated admin user from the GraphQL context.
///
/// Use inside mutation resolvers that need the `User` object (e.g. for audit logging).
pub fn require_admin<'a>(ctx: &'a Context<'a>) -> Result<&'a User> {
    let auth = ctx.data::<MaybeAuthUser>()?;
    match &auth.0 {
        Some(auth_user) if auth_user.user.role.is_admin() => Ok(&auth_user.user),
        Some(_) => Err(AppError::Forbidden("Admin access required".into()).extend()),
        None => Err(AppError::Unauthorized("Authentication required".into()).extend()),
    }
}
