use chrono::*;
use uuid::Uuid;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use tonic::{Request, Response, Status};

use tokio_postgres::tls::NoTls;

use crate::user_crud::{
    user_crud_server::UserCrud, CreateUserReply, CreateUserRequest, DeleteUserReply, Empty,
    UpdateUserReply, UpdateUserRequest, UserReply, UserRequest, Users,
};

#[derive(Debug)]
pub struct MyUserCrud {
    pub pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[tonic::async_trait]
impl UserCrud for MyUserCrud {
    async fn get_user(&self, request: Request<UserRequest>) -> Result<Response<UserReply>, Status> {
        println!("Got a request: {:#?}", &request);
        let UserRequest { id } = &request.into_inner();
        let connection = self.pool.get().await.unwrap();
        let stmt = connection
            .prepare("SELECT * FROM users WHERE id = $1")
            .await
            .unwrap();
        let rows = &connection.query(&stmt, &[&id]).await.unwrap();
        let row = rows.get(0).unwrap();
        let date_of_birth: NaiveDate = row.get(3);
        let reply = UserReply {
            id: row.get(0),
            first_name: row.get(1),
            last_name: row.get(2),
            date_of_birth: date_of_birth.to_string(),
        };

        Ok(Response::new(reply))
    }

    async fn list_users(&self, request: Request<Empty>) -> Result<Response<Users>, Status> {
        println!("Got a request: {:#?}", &request);

        let connection = self.pool.get().await.unwrap();
        let stmt = connection.prepare("SELECT * FROM users").await.unwrap();
        let rows = &connection.query(&stmt, &[]).await.unwrap();
        let mut v: Vec<UserReply> = Vec::new();

        for row in rows {
            let date_of_birth: NaiveDate = row.get(3);
            let user = UserReply {
                id: row.get(0),
                first_name: row.get(1),
                last_name: row.get(2),
                date_of_birth: date_of_birth.to_string(),
            };

            v.push(user);
        }

        let reply = Users { users: v };

        Ok(Response::new(reply))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        println!("Got a request: {:#?}", &request);

        let user_id = Uuid::new_v4().to_hyphenated().to_string();
        let CreateUserRequest {
            first_name,
            last_name,
            date_of_birth,
        } = &request.into_inner();
        let serialize_date_of_birth = NaiveDate::parse_from_str(date_of_birth, "%Y-%m-%d").unwrap();
        let connection = self.pool.get().await.unwrap();
        let stmt = connection.prepare("INSERT INTO users (id, first_name, last_name, date_of_birth) VALUES ($1, $2, $3, $4)").await.unwrap();
        let number_of_rows_affected = &connection
            .execute(
                &stmt,
                &[&user_id, &first_name, &last_name, &serialize_date_of_birth],
            )
            .await
            .unwrap();
        let reply = if number_of_rows_affected == &(0 as u64) {
            CreateUserReply {
                message: format!("Fail to create user with id {}.", &user_id),
            }
        } else {
            CreateUserReply {
                message: format!(
                    "Create {} user with id {}.",
                    &number_of_rows_affected, &user_id
                ),
            }
        };

        Ok(Response::new(reply))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserReply>, Status> {
        println!("Got a request: {:#?}", &request);

        let UpdateUserRequest {
            id,
            first_name,
            last_name,
            date_of_birth,
        } = &request.into_inner();
        let serialize_date_of_birth = NaiveDate::parse_from_str(date_of_birth, "%Y-%m-%d").unwrap();
        let connection = self.pool.get().await.unwrap();
        let stmt = connection.prepare("UPDATE users SET first_name = $2, last_name = $3, date_of_birth = $4 WHERE id = $1").await.unwrap();
        let number_of_rows_affected = &connection
            .execute(
                &stmt,
                &[&id, &first_name, &last_name, &serialize_date_of_birth],
            )
            .await
            .unwrap();
        let reply = if number_of_rows_affected == &(0 as u64) {
            UpdateUserReply {
                message: format!("Fail to update the user with id {}.", id),
            }
        } else {
            UpdateUserReply {
                message: format!("Update {} user with id {}", &number_of_rows_affected, &id),
            }
        };

        Ok(Response::new(reply))
    }

    async fn delete_user(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        println!("Got a request: {:#?}", &request);

        let UserRequest { id } = &request.into_inner();
        let connection = self.pool.get().await.unwrap();
        let stmt = connection
            .prepare("DELETE FROM users WHERE id = $1")
            .await
            .unwrap();
        let number_of_rows_affected = &connection.execute(&stmt, &[&id]).await.unwrap();
        let reply = if number_of_rows_affected == &(0 as u64) {
            DeleteUserReply {
                message: format!("Fail to delete the user with id {}.", id),
            }
        } else {
            DeleteUserReply {
                message: format!("Remove the user with id {}.", id),
            }
        };

        Ok(Response::new(reply))
    }

    async fn delete_users(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        println!("Got a request: {:#?}", &request);

        let connection = self.pool.get().await.unwrap();
        let stmt = connection.prepare("DELETE FROM users").await.unwrap();
        let number_of_rows_affected = &connection.execute(&stmt, &[]).await.unwrap();
        let reply = DeleteUserReply {
            message: format!(
                "Remove {} user data from the database.",
                number_of_rows_affected
            ),
        };

        Ok(Response::new(reply))
    }
}
