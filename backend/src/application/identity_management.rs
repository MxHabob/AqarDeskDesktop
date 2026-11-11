use crate::{
    application::{
        ApplicationError, IdentityRepository, NewUser, OrganizationBootstrap, UserUpdate,
    },
    domain::{AccessRole, ActorContext, Organization, UserAccount},
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

fn require_admin(actor: ActorContext) -> Result<(), ApplicationError> {
    if actor.role() == AccessRole::Admin {
        Ok(())
    } else {
        Err(ApplicationError::Forbidden)
    }
}

fn valid_email(value: &str) -> bool {
    let value = value.trim();
    value.len() <= 254
        && value.contains('@')
        && !value.bytes().any(|byte| byte.is_ascii_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OrganizationId, UserId};

    struct UnreachableRepository;

    #[async_trait::async_trait]
    impl IdentityRepository for UnreachableRepository {
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
                        role: AccessRole::Staff,
                    },
                )
                .await,
            Err(ApplicationError::Forbidden)
        ));
    }
}
