use crate::{
    application::{ApplicationError, OwnerRepository, require_permission},
    domain::{ActorContext, Owner, OwnerId, OwnerInput, Permission},
};

pub struct OwnerDirectory<R> {
    owners: R,
}

impl<R: OwnerRepository> OwnerDirectory<R> {
    pub fn new(owners: R) -> Self {
        Self { owners }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Owner>, ApplicationError> {
        require_permission(actor, Permission::ViewOwners)?;
        self.owners.list(actor.organization_id()).await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: OwnerInput,
    ) -> Result<Owner, ApplicationError> {
        require_permission(actor, Permission::CreateOwners)?;
        self.owners
            .insert(
                Owner::create(OwnerId::new(), actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        owner_id: OwnerId,
        input: OwnerInput,
    ) -> Result<Owner, ApplicationError> {
        require_permission(actor, Permission::UpdateOwners)?;
        self.owners
            .update(
                Owner::create(owner_id, actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }
}
