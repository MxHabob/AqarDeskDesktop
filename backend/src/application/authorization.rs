use crate::{
    application::ApplicationError,
    domain::{ActorContext, Permission},
};

pub(crate) fn require_permission(
    actor: ActorContext,
    permission: Permission,
) -> Result<(), ApplicationError> {
    if actor.has_permission(permission) {
        Ok(())
    } else {
        Err(ApplicationError::Forbidden)
    }
}

pub(crate) fn require_admin(actor: ActorContext) -> Result<(), ApplicationError> {
    require_permission(actor, Permission::ManageAdministration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccessRole, OrganizationId, Permission, UserId};

    #[test]
    fn readonly_actors_cannot_write() {
        let actor = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Readonly);
        assert!(matches!(
            require_permission(actor, Permission::ViewProperties),
            Err(ApplicationError::Forbidden)
        ));
    }

    #[test]
    fn only_admins_may_manage_administration() {
        let admin = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Admin);
        let staff = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Staff);
        assert!(require_admin(admin).is_ok());
        assert!(matches!(
            require_admin(staff),
            Err(ApplicationError::Forbidden)
        ));
    }

    #[test]
    fn staff_can_create_properties_but_not_manage_administration() {
        let staff = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Staff);
        assert!(require_permission(staff, Permission::CreateProperties).is_ok());
        assert!(matches!(
            require_permission(staff, Permission::ManageAdministration),
            Err(ApplicationError::Forbidden)
        ));
    }

    #[test]
    fn configured_permissions_are_enforced_instead_of_role_defaults() {
        let staff = ActorContext::with_permissions(
            UserId::new(),
            OrganizationId::new(),
            AccessRole::Staff,
            vec![Permission::ViewProperties],
        );
        assert!(require_permission(staff, Permission::ViewProperties).is_ok());
        assert!(matches!(
            require_permission(staff, Permission::RecordPayments),
            Err(ApplicationError::Forbidden)
        ));
        assert!(matches!(
            require_permission(staff, Permission::ManageAdministration),
            Err(ApplicationError::Forbidden)
        ));
    }
}
