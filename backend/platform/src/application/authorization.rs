use crate::{application::ApplicationError, domain::ActorContext};

pub(crate) fn require_write(actor: ActorContext) -> Result<(), ApplicationError> {
    if actor.role().can_write_business_data() {
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
}
