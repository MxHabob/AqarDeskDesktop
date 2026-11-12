use crate::{
    application::{
        ApplicationError, IdentityRepository, NewUser, OrganizationBootstrap, UserUpdate,
        require_admin,
    },
    domain::{AccessRole, ActorContext, Organization, Permission, RolePermissionSet, UserAccount},
};

pub struct IdentityService<R> {
    identities: R,
}

impl<R: IdentityRepository> IdentityService<R> {
    pub fn new(identities: R) -> Self {
        Self { identities }
    }

    pub async fn setup_status(&self) -> Result<bool, ApplicationError> {
        Ok(!self.identities.has_users().await?)
    }

    pub async fn organization_name(&self, actor: ActorContext) -> Result<String, ApplicationError> {
        self.identities
            .organization_name(actor.organization_id())
            .await
    }

    pub async fn bootstrap_admin(
        &self,
        input: OrganizationBootstrap<'_>,
    ) -> Result<(Organization, UserAccount), ApplicationError> {
        if input.organization_name.trim().is_empty()
            || input.name.trim().is_empty()
            || input.password.len() < 12
            || !valid_email(input.email)
        {
            return Err(crate::domain::DomainError::Validation(
                "Organization, administrator, valid email, and a 12-character password are required.",
            )
            .into());
        }
        self.identities.bootstrap_admin(input).await
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<UserAccount, ApplicationError> {
        self.identities.authenticate(email.trim(), password).await
    }

    pub async fn current_user(
        &self,
        user_id: crate::domain::UserId,
    ) -> Result<UserAccount, ApplicationError> {
        self.identities
            .get_active_user(user_id)
            .await?
            .ok_or(ApplicationError::Authentication)
    }

    pub async fn list_users(
        &self,
        actor: ActorContext,
    ) -> Result<Vec<UserAccount>, ApplicationError> {
        require_admin(actor)?;
        self.identities.list_users(actor.organization_id()).await
    }

    pub async fn list_role_permissions(
        &self,
        actor: ActorContext,
    ) -> Result<Vec<RolePermissionSet>, ApplicationError> {
        require_admin(actor)?;
        self.identities
            .list_role_permissions(actor.organization_id())
            .await
    }

    pub async fn update_role_permissions(
        &self,
        actor: ActorContext,
        role_id: &str,
        permissions: Vec<Permission>,
    ) -> Result<RolePermissionSet, ApplicationError> {
        require_admin(actor)?;
        if role_id == AccessRole::Admin.as_str() {
            return Err(crate::domain::DomainError::Validation(
                "Administrator permissions are fixed and cannot be changed.",
            )
            .into());
        }

        validate_role_permissions(&permissions)?;

        self.identities
            .update_role_permissions(
                actor.organization_id(),
                actor.user_id(),
                role_id,
                permissions,
            )
            .await
    }

    pub async fn create_role(
        &self,
        actor: ActorContext,
        name: &str,
        permissions: Vec<Permission>,
    ) -> Result<RolePermissionSet, ApplicationError> {
        require_admin(actor)?;
        validate_role_permissions(&permissions)?;
        let name = name.trim();
        let normalized_name = name.to_lowercase();
        if name.is_empty()
            || name.chars().count() > 80
            || ["admin", "staff", "readonly"].contains(&normalized_name.as_str())
        {
            return Err(crate::domain::DomainError::Validation(
                "Enter a unique custom role name of at most 80 characters.",
            )
            .into());
        }
        self.identities
            .create_role(actor.organization_id(), actor.user_id(), name, permissions)
            .await
    }

    pub async fn delete_role(
        &self,
        actor: ActorContext,
        role_id: &str,
    ) -> Result<(), ApplicationError> {
        require_admin(actor)?;
        if ["admin", "staff", "readonly"].contains(&role_id) {
            return Err(
                crate::domain::DomainError::Validation("System roles cannot be deleted.").into(),
            );
        }
        self.identities
            .delete_role(actor.organization_id(), actor.user_id(), role_id)
            .await
    }

    pub async fn create_user(
        &self,
        actor: ActorContext,
        input: NewUser<'_>,
    ) -> Result<UserAccount, ApplicationError> {
        require_admin(actor)?;
        if input.name.trim().is_empty() || !valid_email(input.email) || input.password.len() < 12 {
            return Err(crate::domain::DomainError::Validation(
                "Name, valid email, and a 12-character password are required.",
            )
            .into());
        }
        self.identities
            .create_user(actor.organization_id(), actor.user_id(), input)
            .await
    }

    pub async fn set_user_active(
        &self,
        actor: ActorContext,
        user_id: crate::domain::UserId,
        is_active: bool,
    ) -> Result<UserAccount, ApplicationError> {
        require_admin(actor)?;
        if actor.user_id() == user_id && !is_active {
            return Err(crate::domain::DomainError::Validation(
                "You cannot deactivate the account currently in use.",
            )
            .into());
        }
        self.identities
            .set_user_active(actor.organization_id(), user_id, actor.user_id(), is_active)
            .await
    }

    pub async fn update_user(
        &self,
        actor: ActorContext,
        user_id: crate::domain::UserId,
        input: UserUpdate<'_>,
    ) -> Result<UserAccount, ApplicationError> {
        require_admin(actor)?;
        if actor.user_id() == user_id && input.role_id != AccessRole::Admin.as_str() {
            return Err(crate::domain::DomainError::Validation(
                "You cannot change the role of the account currently in use.",
            )
            .into());
        }
        if input.name.trim().is_empty()
            || !valid_email(input.email)
            || input.password.is_some_and(|password| password.len() < 12)
        {
            return Err(crate::domain::DomainError::Validation(
                "Name, valid email, and an optional 12-character password are required.",
            )
            .into());
        }
        self.identities
            .update_user(actor.organization_id(), user_id, actor.user_id(), input)
            .await
    }
}

fn valid_email(value: &str) -> bool {
    let value = value.trim();
    value.len() <= 254
        && value.contains('@')
        && !value.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn validate_role_permissions(permissions: &[Permission]) -> Result<(), ApplicationError> {
    let mut unique = std::collections::HashSet::new();
    if permissions.iter().any(|permission| {
        *permission == Permission::ManageAdministration || !unique.insert(*permission)
    }) {
        return Err(crate::domain::DomainError::Validation(
            "The role permission list contains an unsupported or duplicate permission.",
        )
        .into());
    }
    let dependencies = [
        (Permission::CreateProperties, Permission::ViewProperties),
        (Permission::UpdateProperties, Permission::ViewProperties),
        (Permission::ArchiveProperties, Permission::ViewProperties),
        (Permission::CreateOwners, Permission::ViewOwners),
        (Permission::UpdateOwners, Permission::ViewOwners),
        (Permission::CreateUnits, Permission::ViewUnits),
        (Permission::CreateUnits, Permission::ViewProperties),
        (Permission::UpdateUnits, Permission::ViewUnits),
        (Permission::UpdateUnits, Permission::ViewProperties),
        (Permission::ArchiveUnits, Permission::ViewUnits),
        (Permission::CreateTenants, Permission::ViewTenants),
        (Permission::UpdateTenants, Permission::ViewTenants),
        (Permission::ArchiveTenants, Permission::ViewTenants),
        (Permission::RecordPayments, Permission::ViewPayments),
        (Permission::RecordPayments, Permission::ViewTenants),
        (Permission::VoidPayments, Permission::ViewPayments),
    ];
    if dependencies.iter().any(|(operation, prerequisite)| {
        permissions.contains(operation) && !permissions.contains(prerequisite)
    }) {
        return Err(crate::domain::DomainError::Validation(
            "View permissions are required before granting write permissions.",
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OrganizationId, UserId};

    struct UnreachableRepository;

    #[async_trait::async_trait]
    impl IdentityRepository for UnreachableRepository {
        async fn list_role_permissions(
            &self,
            _: OrganizationId,
        ) -> Result<Vec<RolePermissionSet>, ApplicationError> {
            unreachable!()
        }
        async fn update_role_permissions(
            &self,
            _: OrganizationId,
            _: UserId,
            _: &str,
            _: Vec<Permission>,
        ) -> Result<RolePermissionSet, ApplicationError> {
            unreachable!()
        }
        async fn create_role(
            &self,
            _: OrganizationId,
            _: UserId,
            _: &str,
            _: Vec<Permission>,
        ) -> Result<RolePermissionSet, ApplicationError> {
            unreachable!()
        }
        async fn delete_role(
            &self,
            _: OrganizationId,
            _: UserId,
            _: &str,
        ) -> Result<(), ApplicationError> {
            unreachable!()
        }
        async fn has_users(&self) -> Result<bool, ApplicationError> {
            Ok(false)
        }
        async fn organization_name(&self, _: OrganizationId) -> Result<String, ApplicationError> {
            unreachable!()
        }
        async fn bootstrap_admin(
            &self,
            _: OrganizationBootstrap<'_>,
        ) -> Result<(Organization, UserAccount), ApplicationError> {
            unreachable!()
        }
        async fn authenticate(&self, _: &str, _: &str) -> Result<UserAccount, ApplicationError> {
            unreachable!()
        }
        async fn get_active_user(
            &self,
            _: UserId,
        ) -> Result<Option<UserAccount>, ApplicationError> {
            unreachable!()
        }
        async fn list_users(
            &self,
            _: OrganizationId,
        ) -> Result<Vec<UserAccount>, ApplicationError> {
            unreachable!()
        }
        async fn create_user(
            &self,
            _: OrganizationId,
            _: UserId,
            _: NewUser<'_>,
        ) -> Result<UserAccount, ApplicationError> {
            unreachable!()
        }
        async fn update_user(
            &self,
            _: OrganizationId,
            _: UserId,
            _: UserId,
            _: UserUpdate<'_>,
        ) -> Result<UserAccount, ApplicationError> {
            unreachable!()
        }
        async fn set_user_active(
            &self,
            _: OrganizationId,
            _: UserId,
            _: UserId,
            _: bool,
        ) -> Result<UserAccount, ApplicationError> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn bootstrap_validates_inputs_before_storage() {
        let service = IdentityService::new(UnreachableRepository);
        let result = service
            .bootstrap_admin(OrganizationBootstrap {
                organization_name: "AqarDesk",
                name: "Admin",
                email: "admin@example.com",
                password: "short",
            })
            .await;
        assert!(matches!(result, Err(ApplicationError::Domain(_))));
    }

    #[tokio::test]
    async fn only_admins_may_manage_users() {
        let service = IdentityService::new(UnreachableRepository);
        let actor = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Staff);
        assert!(matches!(
            service.list_users(actor).await,
            Err(ApplicationError::Forbidden)
        ));
        assert!(matches!(
            service
                .update_user(
                    actor,
                    UserId::new(),
                    UserUpdate {
                        name: "Staff",
                        email: "staff@example.com",
                        password: None,
                        role_id: "staff",
                    },
                )
                .await,
            Err(ApplicationError::Forbidden)
        ));
        assert!(matches!(
            service
                .update_role_permissions(actor, "staff", vec![Permission::ViewProperties])
                .await,
            Err(ApplicationError::Forbidden)
        ));
    }

    #[tokio::test]
    async fn administrator_permissions_cannot_be_reconfigured() {
        let service = IdentityService::new(UnreachableRepository);
        let actor = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Admin);
        assert!(matches!(
            service
                .update_role_permissions(actor, "admin", Vec::new())
                .await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
        assert!(matches!(
            service
                .update_role_permissions(
                    actor,
                    "staff",
                    vec![Permission::ViewProperties, Permission::ViewProperties]
                )
                .await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
        assert!(matches!(
            service
                .update_role_permissions(actor, "staff", vec![Permission::ManageAdministration])
                .await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
        assert!(matches!(
            service.create_role(actor, " ADMIN ", Vec::new()).await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
        assert!(matches!(
            service
                .create_role(actor, "Sales", vec![Permission::ManageAdministration])
                .await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
    }

    #[test]
    fn write_permissions_require_their_read_dependencies() {
        assert!(validate_role_permissions(&[Permission::CreateProperties]).is_err());
        assert!(
            validate_role_permissions(&[Permission::ViewPayments, Permission::RecordPayments,])
                .is_err()
        );
        assert!(
            validate_role_permissions(&[
                Permission::ViewPayments,
                Permission::ViewTenants,
                Permission::RecordPayments,
            ])
            .is_ok()
        );
    }

    #[tokio::test]
    async fn administrator_cannot_demote_the_active_account() {
        let service = IdentityService::new(UnreachableRepository);
        let user_id = UserId::new();
        let actor = ActorContext::new(user_id, OrganizationId::new(), AccessRole::Admin);
        assert!(matches!(
            service
                .update_user(
                    actor,
                    user_id,
                    UserUpdate {
                        name: "Admin",
                        email: "admin@example.com",
                        password: None,
                        role_id: "staff",
                    }
                )
                .await,
            Err(ApplicationError::Domain(
                crate::domain::DomainError::Validation(_)
            ))
        ));
    }
}
