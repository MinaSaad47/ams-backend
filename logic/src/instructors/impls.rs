use std::sync::Arc;

use sea_orm::Set;

use super::*;

pub struct InstructorsRepo(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for InstructorsRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl InstructorsRepoTrait for InstructorsRepo {
    async fn create(&self, instructor: CreateInstructor) -> Result<Instructor, RepoError> {
        Ok(instructors::ActiveModel {
            name: Set(instructor.name),
            email: Set(instructor.email),
            password: Set(instructor.password),
            number: Set(instructor.number),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?
        .into())
    }
    async fn update(
        &self,
        id: Uuid,
        UpdateInstructor {
            name,
            email,
            password,
            number,
        }: UpdateInstructor,
    ) -> Result<Instructor, RepoError> {
        let mut instructor: instructors::ActiveModel = instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into();

        if let Some(name) = name {
            instructor.name = Set(name);
        }
        if let Some(email) = email {
            instructor.email = Set(email);
        }
        if let Some(password) = password {
            instructor.password = Set(password);
        }
        if let Some(number) = number {
            instructor.number = Set(number);
        }

        let instructor: instructors::Model = instructor.update(self.as_ref()).await?;

        Ok(instructor.into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find()
            .filter(instructors::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Instructor>, RepoError> {
        Ok(instructors::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(Instructor::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        instructors::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}
