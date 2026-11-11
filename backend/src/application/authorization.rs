use crate::{
    application::ApplicationError,
    domain::{AccessRole, ActorContext},
};

pub(crate) fn require_write(actor: ActorContext) -> Result<(), ApplicationError> {
    if actor.role().can_write_business_data() {
        Ok(())
    } else {
        Err(ApplicationError::Forbidden)
    }
}

pub(crate) fn require_admin(actor: ActorContext) -> Result<(), ApplicationError> {
    if actor.role() == AccessRole::Admin {
        Ok(())
    } else {
        Err(ApplicationError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccessRole, OrganizationId, UserId};

    #[test]
    fn readonly_actors_cannot_write() {
        let actor = ActorContext::new(UserId::new(), OrganizationId::new(), AccessRole::Readonly);
        assert!(matches!(
            require_write(actor),
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
}
