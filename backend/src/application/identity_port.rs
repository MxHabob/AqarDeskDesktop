use crate::{
    application::ApplicationError,
    domain::{Organization, OrganizationId, RolePermissionSet, UserAccount, UserId},
};

pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub role_id: &'a str,
}

pub struct UserUpdate<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: Option<&'a str>,
    pub role_id: &'a str,
}

pub struct OrganizationBootstrap<'a> {
    pub organization_name: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[async_trait::async_trait]
pub trait IdentityRepository: Send + Sync {
    async fn list_role_permissions(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<RolePermissionSet>, ApplicationError>;

    async fn update_role_permissions(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        role_id: &str,
        permissions: Vec<crate::domain::Permission>,
    ) -> Result<RolePermissionSet, ApplicationError>;

    async fn create_role(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        name: &str,
        permissions: Vec<crate::domain::Permission>,
    ) -> Result<RolePermissionSet, ApplicationError>;

    async fn delete_role(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        role_id: &str,
    ) -> Result<(), ApplicationError>;

    async fn has_users(&self) -> Result<bool, ApplicationError>;

    async fn organization_name(
        &self,
        organization_id: OrganizationId,
    ) -> Result<String, ApplicationError>;

    async fn bootstrap_admin(
        &self,
        input: OrganizationBootstrap<'_>,
    ) -> Result<(Organization, UserAccount), ApplicationError>;

    async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<UserAccount, ApplicationError>;

    async fn get_active_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserAccount>, ApplicationError>;

    async fn list_users(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<UserAccount>, ApplicationError>;

    async fn create_user(
        &self,
        organization_id: OrganizationId,
        actor_id: UserId,
        input: NewUser<'_>,
    ) -> Result<UserAccount, ApplicationError>;

    async fn update_user(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
        actor_id: UserId,
        input: UserUpdate<'_>,
    ) -> Result<UserAccount, ApplicationError>;

    async fn set_user_active(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
        actor_id: UserId,
        is_active: bool,
    ) -> Result<UserAccount, ApplicationError>;
}
